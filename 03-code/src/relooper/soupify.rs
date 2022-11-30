use crate::middle_end::ids::LabelId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::Program;
use std::collections::HashMap;

/// Given a list of instructions, generate a 'soup of labelled blocks'
pub fn soupify(prog: &mut Box<Program>) {
    for (_fun_id, function) in &mut prog.functions {
        remove_consecutive_labels(&mut function.instrs);
        remove_label_fallthrough(&mut function.instrs);
    }
    remove_consecutive_labels(&mut prog.global_instrs);
    remove_label_fallthrough(&mut prog.global_instrs);

    add_block_gap_labels_after_conditionals(prog);

    println!("soupified: {}", prog);

    // for instr in instrs {
    //     println!("  {}", instr);
    //     match instr {
    //         Instruction::Label(label_id) => {
    //             // start of a block
    //         }
    //         Instruction::Br(label_id)
    //         | Instruction::BrIfEq(_, _, label_id)
    //         | Instruction::BrIfNotEq(_, _, label_id)
    //         | Instruction::BrIfGT(_, _, label_id)
    //         | Instruction::BrIfLT(_, _, label_id)
    //         | Instruction::BrIfGE(_, _, label_id)
    //         | Instruction::BrIfLE(_, _, label_id) => {
    //             // end of a block
    //         }
    //         i => {
    //             // inside a block
    //         }
    //     }
    // }
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

fn add_block_gap_labels_after_conditionals(prog: &mut Box<Program>) {
    for (_fun_id, function) in &mut prog.functions {
        let mut i = 0;
        loop {
            if i >= function.instrs.len() {
                break;
            }
            let instr = function.instrs.get(i).unwrap();
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
                    // increment i accounting for the new instructions we added
                    i += 3;
                    let new_label = prog.new_label();
                    function.instrs.push(Instruction::Br(new_label.to_owned()));
                    function.instrs.push(Instruction::Label(new_label));
                }
            }
        }
    }

    // global instrs
    let mut i = 0;
    loop {
        if i >= prog.global_instrs.len() {
            break;
        }
        let instr = prog.global_instrs.get(i).unwrap();
        let mut instrs_to_insert: Option<Vec<Instruction>> = None;

        match instr {
            Instruction::BrIfEq(_, _, _)
            | Instruction::BrIfNotEq(_, _, _)
            | Instruction::BrIfGT(_, _, _)
            | Instruction::BrIfLT(_, _, _)
            | Instruction::BrIfGE(_, _, _)
            | Instruction::BrIfLE(_, _, _) => {
                let new_label = prog.new_label();
                instrs_to_insert = Some(vec![
                    Instruction::Br(new_label.to_owned()),
                    Instruction::Label(new_label),
                ])
            }
            _ => {}
        }
        match instrs_to_insert {
            None => i += 1,
            Some(mut instrs) => {
                // increment i accounting for the new instructions we added
                i += prog.global_instrs.len() + 1;
                prog.global_instrs.append(&mut instrs);
            }
        }
    }
}
