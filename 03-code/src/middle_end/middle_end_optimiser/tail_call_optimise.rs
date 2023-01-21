use crate::middle_end::ids::FunId;
use crate::middle_end::instructions::Instruction::TailCall;
use crate::middle_end::instructions::{Dest, Instruction, Src};
use std::collections::HashMap;

pub fn tail_call_optimise(fun_instrs: &mut Vec<Instruction>) {
    // find any call instructions that are the last instruction of a function

    // instr index, call instr parameters
    let mut current_call_instr: Option<(usize, &Dest, &FunId, &Vec<Src>)> = None;

    // index to start replacing instrs, how many instrs to remove, new instrs to insert
    let mut replace_instrs: Vec<(usize, u32, Vec<Instruction>)> = Vec::new();

    for instr_i in 0..fun_instrs.len() {
        let instr = fun_instrs.get(instr_i).unwrap();
        match instr {
            Instruction::Call(dest, fun_id, params) => {
                current_call_instr = Some((instr_i, dest, fun_id, params));
            }
            Instruction::Ret(return_src) => {
                if let Some((call_instr_i, call_instr_dest, fun_id, params)) = current_call_instr {
                    if let Some(Src::Var(return_var_src)) = return_src {
                        if return_var_src == call_instr_dest {
                            // tail call optimise!
                            replace_instrs.push((
                                call_instr_i,
                                (instr_i - call_instr_i + 1) as u32,
                                vec![todo!("set param values instead of calling new function")],
                            ));
                        }
                    }
                }
                current_call_instr = None;
            }
            _ => {
                current_call_instr = None;
            }
        }
    }

    for (replace_index, remove_count, new_instr) in replace_instrs {
        for _ in 0..remove_count {
            fun_instrs.remove(replace_index);
        }
        fun_instrs.insert(replace_index, new_instr);
    }
}
