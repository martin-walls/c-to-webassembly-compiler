use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

use log::debug;

use crate::fmt_indented::{FmtIndented, IndentLevel};
use crate::middle_end::ids::VarId;

type IntervalBound = u32;

/// Data stored in an interval tree should be 'mergeable'. If we try to insert data
/// with an interval that already exists in the tree, the data will be merged
pub trait Mergeable {
    fn merge(&mut self, other: &Self);
}

#[derive(PartialEq, Clone)]
pub struct Interval {
    pub start: IntervalBound,
    pub end: IntervalBound,
}

pub struct IntervalTree<T: Mergeable> {
    root: *mut Node<T>,
}

struct Node<T: Mergeable> {
    // Null pointer = child is a leaf
    left: *mut Node<T>,
    right: *mut Node<T>,
    parent: *mut Node<T>,
    colour: NodeColour,
    interval: Interval,
    max: IntervalBound,
    data: T,
}

#[derive(Clone, PartialEq)]
enum NodeColour {
    Red,
    Black,
}

impl<T: Mergeable> Node<T> {
    fn new(colour: NodeColour, interval: Interval, max: u32, data: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            colour,
            interval,
            max,
            data,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
        }))
    }

    fn key(&self) -> IntervalBound {
        self.interval.start
    }

    fn merge_data(&mut self, other: *const Node<T>) {
        unsafe {
            self.data.merge(&(*other).data);
        }
    }
}

impl<T: Mergeable> IntervalTree<T> {
    pub fn new() -> Self {
        Self {
            root: std::ptr::null_mut(),
        }
    }

    /// Insert data to the given interval in the tree. If the interval isn't in the tree, a new
    /// node is created. If the interval already exists, data is merged to the existing node.
    pub fn insert_or_merge(&mut self, interval: Interval, data: T) {
        debug!("inserting");
        let new_node = Node::new(NodeColour::Red, interval.to_owned(), interval.end, data);

        let mut y = std::ptr::null_mut();
        let mut x = self.root;

        unsafe {
            // walk down the tree until we get to a leaf, or we find a matching node
            while !x.is_null() {
                y = x;

                if (*new_node).interval == (*x).interval {
                    // merge new node with x
                    (*x).merge_data(new_node);
                    return;
                }

                // smaller keys or equal to the left, bigger keys to the right
                if (*new_node).key() <= (*x).key() {
                    x = (*x).left;
                } else {
                    x = (*x).right;
                }
            }

            // replace the leaf with the new node
            (*new_node).parent = y;

            // check if the tree is empty (we didn't walk down it), and so
            // set the new node as the root
            // otherwise set the new node as the correct left/right child depending on its key
            if y.is_null() {
                self.root = new_node;
            } else if (*new_node).key() <= (*y).key() {
                (*y).left = new_node;
            } else {
                (*y).right = new_node;
            }

            self.insert_fixup(new_node);
        }
    }

    /// ```plaintext
    ///       X                 Y
    ///      / \               / \
    ///     a   Y     ==>     X   c
    ///        / \           / \
    ///       b   c         a   b
    /// ```
    ///
    /// Returns true if the rotation was completed successfully
    fn left_rotate(&mut self, x: *mut Node<T>) -> bool {
        unsafe {
            // can't left-rotate if no right child
            if (*x).right.is_null() {
                return false;
            }

            // get X's right subtree
            let y = (*x).right;

            // make Y's left subtree into X's right subtree
            (*x).right = (*y).left;
            // point X's new right subtree upwards to X
            if !(*x).right.is_null() {
                (*(*x).right).parent = x;
            }

            // point Y upwards to X's parent
            (*y).parent = (*x).parent;

            // attach Y as the correct child of X's parent
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == (*(*x).parent).left {
                (*(*x).parent).left = y;
            } else {
                (*(*x).parent).right = y;
            }

            // put X as Y's left subtree
            (*y).left = x;
            // point X upwards to Y
            (*x).parent = y;
        }
        true
    }

    /// ```plaintext
    ///       X                 Y
    ///      / \               / \
    ///     Y   c     ==>     a   X
    ///    / \                   / \
    ///   a   b                 b   c
    /// ```
    ///
    /// Returns true if the rotation was completed successfully
    fn right_rotate(&mut self, x: *mut Node<T>) {
        unsafe {
            // can't right-rotate if no left child
            if (*x).left.is_null() {
                return;
            }

            // get X's left subtree
            let y = (*x).left;

            // make Y's right subtree into X's left subtree
            (*x).left = (*y).right;
            // point X's new left subtree upwards to X
            if !(*x).left.is_null() {
                (*(*x).left).parent = x;
            }

            // point Y upwards to X's parent
            (*y).parent = (*x).parent;

            // attach Y as the correct child of X's parent
            if (*x).parent.is_null() {
                self.root = y;
            } else if x == (*(*x).parent).left {
                (*(*x).parent).left = y;
            } else {
                (*(*x).parent).right = y;
            }

            // put X as Y's right subtree
            (*y).right = x;
            // point X upwards to Y
            (*x).parent = y;
        }
    }

    /// After inserting a new node, maintain the red-black tree invariants
    fn insert_fixup(&mut self, mut node: *mut Node<T>) {
        debug!("fixup");
        unsafe {
            // the node we've inserted is red, so if parent is also red we need to fixup
            while !(*node).parent.is_null() && (*(*node).parent).colour == NodeColour::Red {
                // if parent is a left child
                if (*node).parent == (*(*(*node).parent).parent).left {
                    let y = (*(*(*node).parent).parent).right;
                    if (*y).colour == NodeColour::Red {
                        (*(*node).parent).colour = NodeColour::Black;
                        (*y).colour = NodeColour::Black;
                        (*(*(*node).parent).parent).colour = NodeColour::Red;
                        node = (*(*node).parent).parent;
                    } else {
                        //      o            o
                        //     /            /
                        //    o     =>     o
                        //     \          /
                        //      o        o
                        if node == (*(*node).parent).right {
                            node = (*node).parent;
                            self.left_rotate(node);
                        }
                        (*(*node).parent).colour = NodeColour::Black;
                        (*(*(*node).parent).parent).colour = NodeColour::Red;
                        self.right_rotate((*(*node).parent).parent);
                    }
                } else {
                    // symmetric for if parent is a right child
                    let y = (*(*(*node).parent).parent).left;
                    if (*y).colour == NodeColour::Red {
                        (*(*node).parent).colour = NodeColour::Black;
                        (*y).colour = NodeColour::Black;
                        (*(*(*node).parent).parent).colour = NodeColour::Red;
                        node = (*(*node).parent).parent;
                    } else {
                        //    o          o
                        //     \          \
                        //      o   =>     o
                        //     /            \
                        //    o              o
                        if node == (*(*node).parent).left {
                            node = (*node).parent;
                            self.right_rotate(node);
                        }
                        (*(*node).parent).colour = NodeColour::Black;
                        (*(*(*node).parent).parent).colour = NodeColour::Red;
                        self.left_rotate((*(*node).parent).parent);
                    }
                }
            }
            // the root should always be black
            (*self.root).colour = NodeColour::Black;
        }
        debug!("done");
    }

    /// Deallocate all the nodes from the given node downwards
    fn deallocate_from(&mut self, node: *mut Node<T>) {
        if node.is_null() {
            return;
        }
        unsafe {
            // deallocate both children before we can deallocate the node itself
            self.deallocate_from((*node).left);
            self.deallocate_from((*node).right);

            // cos the box now owns node, node will be deallocated
            drop(Box::from_raw(node));
        }
    }
}

impl<T: Mergeable> Drop for IntervalTree<T> {
    fn drop(&mut self) {
        self.deallocate_from(self.root);
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}

impl<T: Mergeable> FmtIndented for Node<T> {
    fn fmt_indented(&self, f: &mut Formatter<'_>, indent_level: &mut IndentLevel) -> fmt::Result {
        indent_level.write(f)?;
        writeln!(f, "{} (max = {})", self.interval, self.max)?;
        indent_level.increment_marked();
        unsafe {
            if !self.left.is_null() {
                (*self.left).fmt_indented(f, indent_level)?;
            } else {
                indent_level.write(f)?;
                writeln!(f, "NULL")?;
            }
            if !self.right.is_null() {
                (*self.right).fmt_indented(f, indent_level)?;
            } else {
                indent_level.write(f)?;
                writeln!(f, "NULL")?;
            }
        }
        indent_level.decrement();
        Ok(())
    }
}

impl<T: Mergeable> fmt::Display for IntervalTree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unsafe {
            if !self.root.is_null() {
                (*self.root).fmt_indented(f, &mut IndentLevel::zero())
            } else {
                write!(f, "NULL")
            }
        }
    }
}
