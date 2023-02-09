use std::collections::HashMap;

use log::trace;

use crate::middle_end::ids::LabelId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
use crate::relooper::blocks::Label;
use crate::relooper::relooper::Labels;

/// Given a list of instructions, generate a 'soup of labelled blocks'
pub fn soupify(
    mut instrs: Vec<Instruction>,
    prog_metadata: &mut Box<ProgramMetadata>,
) -> (Labels, LabelId) {
    assert!(
        !instrs.is_empty(),
        "List of instructions to soupify should be non-empty"
    );
    remove_label_fallthrough(&mut instrs, prog_metadata);
    add_block_gap_labels_after_conditionals(&mut instrs, prog_metadata);
    insert_entry_label_if_necessary(&mut instrs, prog_metadata);
    remove_consecutive_labels(&mut instrs);
    trace!("Processed instrs for soupifying:");
    for instr in &instrs {
        trace!("  {}", instr);
    }
    instructions_to_soup_of_labels(instrs)
}

/// Add a new label at the start of the instructions, to be our entry-point
fn insert_entry_label_if_necessary(
    instrs: &mut Vec<Instruction>,
    prog_metadata: &mut Box<ProgramMetadata>,
) {
    match instrs.get(0) {
        Some(Instruction::Label(..)) => {}
        Some(_) => {
            instrs.insert(
                0,
                Instruction::Label(
                    prog_metadata.new_instr_id(),
                    prog_metadata.label_id_generator.new_id(),
                ),
            );
        }
        None => {}
    }
}

/// Combine any consecutive label instructions into a single label, and remap
/// all branches to those labels to the single label.
fn remove_consecutive_labels(instrs: &mut Vec<Instruction>) {
    // remap label x to label y
    let mut label_remappings = HashMap::new();
    // keep track of whether the last instruction we saw was also a label
    let mut prev_instr_label: Option<&LabelId> = None;

    for instr in instrs.iter() {
        match instr {
            Instruction::Label(_, label_id) => {
                if let Some(prev_label_id) = prev_instr_label {
                    label_remappings.insert(label_id.to_owned(), prev_label_id.to_owned());
                    // keep prev_instr_label the same for the next instr
                } else {
                    prev_instr_label = Some(label_id);
                }
            }
            _ => {
                prev_instr_label = None;
            }
        }
    }

    // remap the labels
    for i in 0..instrs.len() {
        let instr = instrs.get(i).unwrap();
        match instr {
            Instruction::Br(id, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::Br(id.to_owned(), new_label_id.to_owned().to_owned());
                }
            },
            Instruction::BrIfEq(id, s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfEq(
                        id.to_owned(),
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            Instruction::BrIfNotEq(id, s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfNotEq(
                        id.to_owned(),
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            _ => {}
        }
    }

    // remove all labels we've remapped
    // (remove all instructions for which the closure returns false)
    instrs.retain(|instr| {
        if let Instruction::Label(_, label_id) = instr {
            // if the label has been remapped, contains_key is true, so
            // return the negation
            return !label_remappings.contains_key(label_id);
        }
        true
    })
}

/// Make sure that there is no fall-through from one block to the next, by adding
/// branch instructions where fall-through exists. This adds a lot of redundant
/// branch instructions, but this will allow us to split the instructions into a soup of blocks.
/// We'll optimise out the redundant branches afterwards.
fn remove_label_fallthrough(
    instrs: &mut Vec<Instruction>,
    prog_metadata: &mut Box<ProgramMetadata>,
) {
    let mut prev_instr_was_branch = false;

    let mut i = 0;
    loop {
        if i >= instrs.len() {
            break;
        }
        let instr = instrs.get(i).unwrap();
        let mut instr_to_insert: Option<Instruction> = None;
        match instr {
            Instruction::Label(_, label_id) => {
                // if the last instruction was already a branch, do nothing
                if prev_instr_was_branch {
                    prev_instr_was_branch = false;
                } else {
                    // if the previous instruction isn't a branch, insert a branch
                    // to this label
                    instr_to_insert = Some(Instruction::Br(
                        prog_metadata.new_instr_id(),
                        label_id.to_owned(),
                    ));
                }
            }
            Instruction::Br(..) | Instruction::BrIfEq(..) | Instruction::BrIfNotEq(..) => {
                prev_instr_was_branch = true;
            }
            _ => {
                prev_instr_was_branch = false;
            }
        }
        match instr_to_insert {
            None => i += 1,
            Some(instr) => {
                instrs.insert(i, instr);
                // increment i an extra time to account for the new instruction
                // we added to the vector
                i += 2;
            }
        }
    }
}

/// After a conditional branch, insert an unconditional branch and the label it
/// branches to directly after, so that a block always ends with a branch
fn add_block_gap_labels_after_conditionals(
    instrs: &mut Vec<Instruction>,
    prog_metadata: &mut Box<ProgramMetadata>,
) {
    let mut i = 0;
    loop {
        if i >= instrs.len() {
            break;
        }
        let instr = instrs.get(i).unwrap();
        let mut insert_gap_label = false;

        match instr {
            Instruction::BrIfEq(..) | Instruction::BrIfNotEq(..) => {
                insert_gap_label = true;
            }
            _ => {}
        }
        match insert_gap_label {
            false => i += 1,
            true => {
                let new_label = prog_metadata.label_id_generator.new_id();
                instrs.insert(
                    i + 1,
                    Instruction::Br(prog_metadata.new_instr_id(), new_label.to_owned()),
                );
                instrs.insert(
                    i + 2,
                    Instruction::Label(prog_metadata.new_instr_id(), new_label),
                );
                // increment i accounting for the new instructions we added
                i += 3;
            }
        }
    }
}

/// Convert a list of instructions, which has been processed to add appropriate
/// label instructions, to a 'soup of blocks' (which we call labels).
///
/// The first label in the resulting vector is the entry-point.
fn instructions_to_soup_of_labels(instrs: Vec<Instruction>) -> (HashMap<LabelId, Label>, LabelId) {
    let mut labels = HashMap::new();
    let mut current_label_id: Option<LabelId> = None;
    let mut entry: Option<LabelId> = None;
    for instr in instrs {
        match instr {
            Instruction::Label(_, label_id) => {
                if current_label_id.is_none() {
                    // no previous block => this new block is the entry
                    entry = Some(label_id.to_owned());
                }
                // start of a new block
                current_label_id = Some(label_id.to_owned());
                let new_label = Label::new(label_id.to_owned());
                labels.insert(label_id, new_label);
            }
            i => {
                // any other instruction is continuation of the current block
                if let Some(current_label_id) = &current_label_id {
                    labels.get_mut(current_label_id).unwrap().instrs.push(i);
                }
            }
        }
    }
    (labels, entry.unwrap())
}
