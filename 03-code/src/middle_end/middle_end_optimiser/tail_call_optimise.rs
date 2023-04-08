use crate::middle_end::ids::{FunId, ValueType, VarId};
use crate::middle_end::instructions::{Dest, Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;

pub fn tail_call_optimise(
    fun_instrs: &mut Vec<Instruction>,
    this_fun_id: &FunId,
    param_var_mappings: &[VarId],
    prog_metadata: &mut ProgramMetadata,
) {
    // find any call instructions that are the last instruction of a function

    // instr index, call instr parameters
    let mut current_call_instr: Option<(usize, &Dest, &FunId, &Vec<Src>)> = None;

    // index to start replacing instrs, how many instrs to remove, new instrs to insert
    let mut replace_instrs: Vec<(usize, u32, Vec<Instruction>)> = Vec::new();

    // label so we can jump back to the start of the function
    let start_of_fun_label = prog_metadata.new_label();
    fun_instrs.insert(
        0,
        Instruction::Label(prog_metadata.new_instr_id(), start_of_fun_label.to_owned()),
    );

    for instr_i in 0..fun_instrs.len() {
        let instr = fun_instrs.get(instr_i).unwrap();
        match instr {
            Instruction::Call(_, dest, fun_id, params) => {
                current_call_instr = Some((instr_i, dest, fun_id, params));
            }
            Instruction::Ret(_, return_src) => {
                if let Some((call_instr_i, call_instr_dest, call_instr_fun_id, params)) =
                    current_call_instr
                {
                    if let Some(Src::Var(return_var_src)) = return_src {
                        if return_var_src == call_instr_dest {
                            let mut new_instrs = Vec::new();
                            // check if recursive call
                            if call_instr_fun_id == this_fun_id {
                                // optimise tail recursion
                                // first make a temp copy of each of the param vars, so we don't
                                // accidentally overwrite a value before we use it in another param
                                let mut temp_param_vars = Vec::new();
                                for param_i in 0..params.len() {
                                    let new_param_src = params.get(param_i).unwrap();
                                    let temp_var = prog_metadata.new_var(ValueType::RValue);
                                    prog_metadata
                                        .add_var_type(
                                            temp_var.to_owned(),
                                            new_param_src.get_type(prog_metadata).unwrap(),
                                        )
                                        .unwrap();
                                    temp_param_vars.push(temp_var.to_owned());
                                    new_instrs.push(Instruction::SimpleAssignment(
                                        prog_metadata.new_instr_id(),
                                        temp_var,
                                        new_param_src.to_owned(),
                                    ));
                                }
                                // store the temp params to the actual params
                                for param_i in 0..params.len() {
                                    let temp_param = temp_param_vars.get(param_i).unwrap();
                                    let param_var = param_var_mappings.get(param_i).unwrap();
                                    new_instrs.push(Instruction::SimpleAssignment(
                                        prog_metadata.new_instr_id(),
                                        param_var.to_owned(),
                                        Src::Var(temp_param.to_owned()),
                                    ));
                                }
                                // jump back to start of function
                                new_instrs.push(Instruction::Br(
                                    prog_metadata.new_instr_id(),
                                    start_of_fun_label.to_owned(),
                                ));
                            } else {
                                // optimise non-recursive tail call
                                new_instrs.push(Instruction::TailCall(
                                    prog_metadata.new_instr_id(),
                                    call_instr_fun_id.to_owned(),
                                    params.to_vec(),
                                ));
                            }
                            replace_instrs.push((
                                call_instr_i,
                                (instr_i - call_instr_i + 1) as u32,
                                new_instrs,
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

    for (replace_index, remove_count, new_instrs) in replace_instrs {
        for _ in 0..remove_count {
            fun_instrs.remove(replace_index);
        }
        let mut i = replace_index;
        for new_instr in new_instrs {
            fun_instrs.insert(i, new_instr);
            i += 1;
        }
    }
}
