use std::collections::HashMap;

use crate::middle_end::ids::LabelId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::middle_end_error::MiddleEndError;

/// Check through the instructions and remove any labels that aren't the target of
/// a branch instruction.
///
/// Also remove any Nop instructions while we're here.
pub fn remove_unused_labels(instrs: &mut Vec<Instruction>) -> Result<(), MiddleEndError> {
    // map of label: used
    let mut labels: HashMap<LabelId, bool> = HashMap::new();
    // discover all the labels in the instructions, and check if they're used or not
    for instr in instrs.iter() {
        match instr {
            Instruction::Label(_, label_id) => match labels.get(label_id) {
                None => {
                    // if we find a new label that we haven't seen yet, mark it as unused
                    labels.insert(label_id.to_owned(), false);
                }
                Some(_) => {}
            },
            Instruction::Br(_, label_id)
            | Instruction::BrIfEq(_, _, _, label_id)
            | Instruction::BrIfNotEq(_, _, _, label_id) => {
                // found a usage of the label
                labels.insert(label_id.to_owned(), true);
            }
            _ => {}
        }
    }

    // remove all instructions for which the closure returns false
    instrs.retain(|instr| {
        match &instr {
            Instruction::Label(_, label_id) => {
                // if label is unused, remove the instruction
                return labels.get(label_id).unwrap().to_owned();
            }
            // also remove nops
            Instruction::Nop(..) => false,
            _ => true,
        }
    });

    Ok(())
}
