use crate::middle_end::ids::FunId;
use crate::middle_end::instructions::{Instruction, Src};
use crate::middle_end::ir::{Function, Program};
use crate::middle_end::ir_types::IrType;
use crate::middle_end::middle_end_error::MiddleEndError;
use log::info;
use std::collections::HashMap;

pub fn handle_variadic_functions(prog: &mut Box<Program>) -> Result<(), MiddleEndError> {
    //                           callee fun id -> (new fun id, concrete type)
    let mut concrete_function_types: HashMap<FunId, Vec<(FunId, Box<IrType>)>> = HashMap::new();

    for (fun_id, fun_type) in &prog.program_metadata.function_types {
        if fun_type.is_function_variadic()? {
            concrete_function_types.insert(fun_id.to_owned(), Vec::new());
        }
    }

    // find all function calls that call a variadic function
    for (_fun_id, function) in &mut prog.program_instructions.functions {
        for instr_i in 0..function.instrs.len() {
            let instr = function.instrs.get(instr_i).unwrap();
            let mut replace_instr = None;
            match instr {
                Instruction::Call(dest, callee_fun_id, params) => {
                    // check if callee function is variadic
                    let callee_fun_type = prog
                        .program_metadata
                        .function_types
                        .get(callee_fun_id)
                        .unwrap();
                    if callee_fun_type.is_function_variadic()? {
                        let non_variadic_param_types =
                            callee_fun_type.get_function_param_types()?;

                        let mut concrete_param_types = Vec::new();
                        // the non-variadic params are always the same
                        concrete_param_types.append(&mut non_variadic_param_types.to_owned());

                        for param in &params[non_variadic_param_types.len()..] {
                            let param_type = match param {
                                Src::Var(var_id) => {
                                    prog.program_metadata.get_var_type(var_id).unwrap()
                                }
                                Src::Constant(constant) => constant.get_type(None),
                                _ => unreachable!(),
                            };
                            concrete_param_types.push(param_type);
                        }

                        let concrete_fun_type = Box::new(IrType::Function(
                            callee_fun_type.get_function_return_type()?,
                            concrete_param_types,
                            false,
                        ));

                        // default value to make compiler happy; it'll get overwritten
                        let mut this_instr_new_fun_id = callee_fun_id.to_owned();

                        // check if this concrete function type already exists
                        let mut concrete_type_already_exists = false;
                        for (existing_concrete_fun_id, existing_concrete_fun_type) in
                            concrete_function_types.get(callee_fun_id).unwrap()
                        {
                            if **existing_concrete_fun_type == *concrete_fun_type {
                                this_instr_new_fun_id = existing_concrete_fun_id.to_owned();
                                concrete_type_already_exists = true;
                                break;
                            }
                        }

                        if !concrete_type_already_exists {
                            // store new concrete type
                            let new_concrete_fun_id = prog.program_metadata.new_fun_id();
                            concrete_function_types
                                .get_mut(callee_fun_id)
                                .unwrap()
                                .push((new_concrete_fun_id.to_owned(), concrete_fun_type));

                            this_instr_new_fun_id = new_concrete_fun_id;
                        }

                        // replace fun id in instruction
                        replace_instr = Some(Instruction::Call(
                            dest.to_owned(),
                            this_instr_new_fun_id,
                            params.to_owned(),
                        ));
                    }
                }
                _ => {}
            }

            if let Some(new_instr) = replace_instr {
                function.instrs.remove(instr_i);
                function.instrs.insert(instr_i, new_instr);
            }
        }
    }

    // store new function types in prog
    for (variadic_fun_id, concrete_functions) in concrete_function_types {
        prog.program_metadata
            .variadic_function_concrete_variants
            .insert(variadic_fun_id.to_owned(), Vec::new());
        for (concrete_fun_id, concrete_fun_type) in concrete_functions {
            prog.program_metadata
                .function_types
                .insert(concrete_fun_id.to_owned(), concrete_fun_type.to_owned());
            prog.program_metadata.function_param_var_mappings.insert(
                concrete_fun_id.to_owned(),
                prog.program_metadata
                    .function_param_var_mappings
                    .get(&variadic_fun_id)
                    .unwrap()
                    .to_owned(),
            );

            prog.program_instructions.insert_function(
                concrete_fun_id.to_owned(),
                Function::declaration(concrete_fun_type),
            );

            prog.program_metadata
                .variadic_function_concrete_variants
                .get_mut(&variadic_fun_id)
                .unwrap()
                .push(concrete_fun_id);
        }
    }

    Ok(())
}
