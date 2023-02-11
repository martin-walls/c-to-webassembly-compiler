use std::collections::HashSet;

use crate::middle_end::ids::VarId;

type IntervalBound = u32;

#[derive(Clone)]
pub struct ClashInterval {
    start: IntervalBound,
    end: IntervalBound,
    clashes: HashSet<VarId>,
}

impl PartialEq for ClashInterval {
    /// equality comparison just based on interval bounds
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

pub struct ClashesIntervalTree {
    root: *mut Node,
}

struct Node {
    // Null pointer = child is a leaf
    left: *mut Node,
    right: *mut Node,
    parent: *mut Node,
    colour: NodeColour,
    interval: ClashInterval,
    max: IntervalBound,
}

#[derive(Clone, PartialEq)]
enum NodeColour {
    Red,
    Black,
}

impl Node {
    fn new(colour: NodeColour, interval: ClashInterval, max: u32) -> *mut Self {
        Box::into_raw(Box::new(Self {
            colour,
            interval,
            max,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
        }))
    }

    fn key(&self) -> IntervalBound {
        self.interval.start
    }

    fn merge_clashes(&mut self, other: *const Node) {
        unsafe {
            self.interval
                .clashes
                .extend((*other).interval.clashes.to_owned())
        }
    }
}

impl ClashesIntervalTree {
    /// ```plaintext
    ///       X                 Y
    ///      / \               / \
    ///     a   Y     ==>     X   c
    ///        / \           / \
    ///       b   c         a   b
    /// ```
    ///
    /// Returns true if the rotation was completed successfully
    fn left_rotate(&mut self, x: *mut Node) -> bool {
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
    fn right_rotate(&mut self, x: *mut Node) {
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

    fn insert_or_merge(&mut self, interval: ClashInterval) {
        let new_node = Node::new(NodeColour::Red, interval.to_owned(), interval.end);

        let mut y = std::ptr::null_mut();
        let mut x = self.root;

        unsafe {
            // walk down the tree until we get to a leaf, or we find a matching node
            while !x.is_null() {
                y = x;

                if (*new_node).interval == (*x).interval {
                    // merge new node with x
                    (*x).merge_clashes(new_node);
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

    fn insert_fixup(&mut self, mut node: *mut Node) {
        unsafe {
            // the node we've inserted is red, so if parent is also red we need to fixup
            while (*(*node).parent).colour == NodeColour::Red {
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
    }

    fn deallocate_from(&mut self, node: *mut Node) {
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

impl Drop for ClashesIntervalTree {
    fn drop(&mut self) {
        self.deallocate_from(self.root);
    }
}
