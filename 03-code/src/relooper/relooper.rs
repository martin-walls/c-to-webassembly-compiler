use crate::middle_end::ids::LabelId;
use crate::middle_end::ir::Program;
use crate::relooper::soupify::{soupify, Label};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

pub fn reloop(mut prog: Box<Program>) {
    for (_fun_id, function) in prog.program_instructions.functions {
        // function with no body (ie. one that we'll link to in JS runtime)
        if function.instrs.is_empty() {
            continue;
        }
        let (labels, entry) = soupify(
            function.instrs,
            &mut prog.program_metadata.label_id_generator,
        );
        let block = create_block_from_labels(labels, vec![entry]);
        match block {
            Some(block) => println!("created block\n{}", block),
            None => println!("No block created"),
        }
    }
    let labels = soupify(
        prog.program_instructions.global_instrs,
        &mut prog.program_metadata.label_id_generator,
    );
}

#[derive(Debug)]
pub enum Block {
    Simple {
        internal: Label,
        next: Option<Box<Block>>,
    },
    Loop {
        inner: Box<Block>,
        next: Option<Box<Block>>,
    },
    Multiple {
        handled_blocks: Vec<Box<Block>>,
        next: Option<Box<Block>>,
    },
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Block::Simple { internal, next } => match next {
                Some(next) => write!(
                    f,
                    "Simple {{ internal: {}, next: {} }}",
                    internal.label, next
                ),
                None => write!(f, "Simple {{ internal: {}, next: NULL }}", internal.label),
            },
            Block::Loop { inner, next } => match next {
                Some(next) => write!(f, "Loop {{ inner: {}, next: {} }}", inner, next),
                None => write!(f, "Loop {{ inner: {}, next: NULL }}", inner),
            },
            Block::Multiple {
                handled_blocks,
                next,
            } => {
                write!(f, "Multiple {{ handled: [")?;
                for handled in &handled_blocks[..handled_blocks.len() - 1] {
                    write!(f, "{}, ", handled)?;
                }
                write!(f, "{}", handled_blocks[handled_blocks.len() - 1])?;
                match next {
                    Some(next) => write!(f, "], next: {} }}", next),
                    None => write!(f, "], next: NULL }}"),
                }
            }
        }
    }
}

fn create_block_from_labels(
    mut labels: HashMap<LabelId, Label>,
    entries: Vec<LabelId>,
) -> Option<Box<Block>> {
    let reachability = calculate_reachability(&labels);
    // for label in labels.values() {
    //     println!("{}", label);
    // }
    // println!("{:#?}", reachability);

    if entries.is_empty() {
        return None;
    }

    // if we have a single entry that we can't return to
    if entries.len() == 1 {
        let single_entry = entries.first().unwrap();
        // check that the single entry isn't contained in the set of possible
        // destination labels from this entry
        if !reachability
            .get(single_entry)
            .unwrap()
            .contains(single_entry)
        {
            println!("Create simple block: {}", single_entry);
            let next_entries = labels.get(single_entry).unwrap().possible_branch_targets();
            let this_label = labels.remove(single_entry).unwrap();
            let next_block = create_block_from_labels(labels, next_entries);
            return Some(Box::new(Block::Simple {
                internal: this_label,
                next: next_block,
            }));
        }
    }

    todo!("implement rest of the relooper algorithm")
}

fn calculate_reachability(labels: &HashMap<LabelId, Label>) -> HashMap<LabelId, Vec<LabelId>> {
    let mut possible_branch_targets = HashMap::new();
    for label in labels.values() {
        possible_branch_targets.insert(label.label.to_owned(), label.possible_branch_targets());
    }

    // compute the transitive closure of possible_branch_targets
    let mut reachability = possible_branch_targets.clone();
    loop {
        let mut made_changes = false;
        for (_source_label, reachable_labels) in reachability.iter_mut() {
            let mut i = 0;
            loop {
                if i >= reachable_labels.len() {
                    break;
                }
                // for all the labels we currently know we can reach from _source_label,
                // if any of their branch targets aren't in the set of reachable nodes,
                // add them
                let reachable_label = reachable_labels.get(i).unwrap();
                for dest_label in possible_branch_targets.get(reachable_label).unwrap() {
                    if !reachable_labels.contains(dest_label) {
                        reachable_labels.push(dest_label.to_owned());
                        made_changes = true;
                    }
                }
                i += 1;
            }
        }
        // keep looping until there's no more labels to add
        if !made_changes {
            break;
        }
    }
    reachability
}
