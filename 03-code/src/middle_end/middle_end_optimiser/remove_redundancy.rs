use crate::middle_end::ids::LabelId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::middle_end_error::MiddleEndError;
use std::collections::HashMap;

pub fn remove_unused_labels(instrs: &mut Vec<Instruction>) -> Result<(), MiddleEndError> {
    // map of label: used
    let mut labels: HashMap<LabelId, bool> = HashMap::new();
    // discover all the labels in the instructions, and check if they're used or not
    for instr in instrs.iter() {
        match instr {
            Instruction::Label(label_id) => match labels.get(label_id) {
                None => {
                    // if we find a new label that we haven't seen yet, mark it as unused
                    labels.insert(label_id.to_owned(), false);
                }
                Some(_) => {}
            },
            Instruction::Br(label_id)
            | Instruction::BrIfEq(_, _, label_id)
            | Instruction::BrIfNotEq(_, _, label_id)
            | Instruction::BrIfGT(_, _, label_id)
            | Instruction::BrIfLT(_, _, label_id)
            | Instruction::BrIfGE(_, _, label_id)
            | Instruction::BrIfLE(_, _, label_id) => {
                // found a usage of the label
                labels.insert(label_id.to_owned(), true);
            }
            _ => {}
        }
    }

    // remove all instructions for which the closure returns false
    instrs.retain(|instr| {
        if let Instruction::Label(label_id) = instr {
            // if label is unused, remove the instruction
            return labels.get(&label_id).unwrap().to_owned();
        }
        true
    });

    // let mut instrs_to_remove = Vec::new();

    // for instr_i in 0..instrs.len() {
    //     if let Instruction::Label(label_id) = instrs.get(instr_i).unwrap() {
    //         // we can safely unwrap because we just iterated and found all the labels
    //         // in the last step
    //         if !labels.get(label_id).unwrap() {
    //             // if label is unused, remove the label instruction
    //             // don't do the removing inside the loop, cos it'll mess up the
    //             // loop index
    //             instrs_to_remove.push(instr_i);
    //         }
    //     }
    // }
    //
    // for instr_i in instrs_to_remove {
    //     instrs.remove(instr_i);
    // }

    Ok(())
}
