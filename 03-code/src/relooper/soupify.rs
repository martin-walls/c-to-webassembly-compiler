use crate::middle_end::ids::{IdGenerator, LabelId};
use crate::middle_end::instructions::Instruction;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

/// A 'label' block. This is a list of instructions starting with a label
/// and ending with one or more branch instructions.
/// We call it a label to distinguish it from the output blocks we're generating.
#[derive(Debug)]
pub struct Label {
    pub label: LabelId,
    pub instrs: Vec<Instruction>,
}

impl Label {
    fn new(label: LabelId) -> Self {
        Label {
            label,
            instrs: Vec::new(),
        }
    }

    pub fn possible_branch_targets(&self) -> Vec<LabelId> {
        let mut targets = Vec::new();
        for instr in &self.instrs {
            match instr {
                Instruction::Br(label_id)
                | Instruction::BrIfEq(_, _, label_id)
                | Instruction::BrIfNotEq(_, _, label_id)
                | Instruction::BrIfGT(_, _, label_id)
                | Instruction::BrIfLT(_, _, label_id)
                | Instruction::BrIfGE(_, _, label_id)
                | Instruction::BrIfLE(_, _, label_id) => {
                    // set semantics - only want one copy of each label to branch to,
                    // even if there are multiple branches
                    if !targets.contains(label_id) {
                        targets.push(label_id.to_owned());
                    }
                }
                _ => {}
            }
        }
        targets
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Label: {}", self.label)?;
        for instr in &self.instrs {
            write!(f, "\n  {}", instr)?;
        }
        write!(f, "")
    }
}

/// Given a list of instructions, generate a 'soup of labelled blocks'
pub fn soupify(
    mut instrs: Vec<Instruction>,
    label_generator: &mut IdGenerator<LabelId>,
) -> (HashMap<LabelId, Label>, LabelId) {
    remove_consecutive_labels(&mut instrs);
    remove_label_fallthrough(&mut instrs);
    add_block_gap_labels_after_conditionals(&mut instrs, label_generator);
    insert_entry_label_if_necessary(&mut instrs, label_generator);
    instructions_to_soup_of_labels(instrs)
}

/// Add a new label at the start of the instructions, to be our entry-point
fn insert_entry_label_if_necessary(
    instrs: &mut Vec<Instruction>,
    label_generator: &mut IdGenerator<LabelId>,
) {
    match instrs.get(0) {
        Some(Instruction::Label(_)) => {}
        Some(_) => {
            instrs.insert(0, Instruction::Label(label_generator.new_id()));
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
            Instruction::Label(label_id) => {
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
            Instruction::Br(label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::Br(new_label_id.to_owned().to_owned());
                }
            },
            Instruction::BrIfEq(s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfEq(
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            Instruction::BrIfNotEq(s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfNotEq(
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            Instruction::BrIfGT(s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfGT(
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            Instruction::BrIfLT(s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfLT(
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            Instruction::BrIfGE(s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfGE(
                        s1.to_owned(),
                        s2.to_owned(),
                        new_label_id.to_owned().to_owned(),
                    );
                }
            },
            Instruction::BrIfLE(s1, s2, label_id) => match label_remappings.get(label_id) {
                None => {}
                Some(new_label_id) => {
                    instrs[i] = Instruction::BrIfLE(
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
        if let Instruction::Label(label_id) = instr {
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
fn remove_label_fallthrough(instrs: &mut Vec<Instruction>) {
    let mut prev_instr_was_branch = false;

    let mut i = 0;
    loop {
        if i >= instrs.len() {
            break;
        }
        let instr = instrs.get(i).unwrap();
        let mut instr_to_insert: Option<Instruction> = None;
        match instr {
            Instruction::Label(label_id) => {
                // if the last instruction was already a branch, do nothing
                if prev_instr_was_branch {
                    prev_instr_was_branch = false;
                } else {
                    // if the previous instruction isn't a branch, insert a branch
                    // to this label
                    instr_to_insert = Some(Instruction::Br(label_id.to_owned()));
                }
            }
            Instruction::Br(_)
            | Instruction::BrIfEq(_, _, _)
            | Instruction::BrIfNotEq(_, _, _)
            | Instruction::BrIfGT(_, _, _)
            | Instruction::BrIfLT(_, _, _)
            | Instruction::BrIfGE(_, _, _)
            | Instruction::BrIfLE(_, _, _) => {
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
    label_generator: &mut IdGenerator<LabelId>,
) {
    let mut i = 0;
    loop {
        if i >= instrs.len() {
            break;
        }
        let instr = instrs.get(i).unwrap();
        let mut insert_gap_label = false;

        match instr {
            Instruction::BrIfEq(_, _, _)
            | Instruction::BrIfNotEq(_, _, _)
            | Instruction::BrIfGT(_, _, _)
            | Instruction::BrIfLT(_, _, _)
            | Instruction::BrIfGE(_, _, _)
            | Instruction::BrIfLE(_, _, _) => {
                insert_gap_label = true;
            }
            _ => {}
        }
        match insert_gap_label {
            false => i += 1,
            true => {
                let new_label = label_generator.new_id();
                instrs.insert(i, Instruction::Br(new_label.to_owned()));
                instrs.insert(i + 1, Instruction::Label(new_label));
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
            Instruction::Label(label_id) => {
                if let None = current_label_id {
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
