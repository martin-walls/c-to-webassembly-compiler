use std::collections::HashMap;

use log::info;

use crate::backend::memory_constants::{
    FRAME_PTR_ADDR, PTR_SIZE, STACK_PTR_ADDR, TEMP_FRAME_PTR_ADDR,
};
use crate::backend::memory_operations::{load, load_constant, load_var, store, store_var};
use crate::backend::profiler::log_stack_ptr;
use crate::backend::target_code_generation_context::{FunctionContext, ModuleContext};
use crate::backend::wasm_instructions::{MemArg, WasmInstruction};
use crate::middle_end::instructions::{Dest, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};

pub fn load_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: FRAME_PTR_ADDR as i32,
    });
    // load
    wasm_instrs.push(WasmInstruction::I32Load {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

fn load_temp_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: TEMP_FRAME_PTR_ADDR as i32,
    });
    // load
    wasm_instrs.push(WasmInstruction::I32Load {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

pub fn restore_previous_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand for storing frame pointer
    wasm_instrs.push(WasmInstruction::I32Const {
        n: FRAME_PTR_ADDR as i32,
    });
    // load the previous frame ptr value (the value that the frame ptr currently points at)
    load_frame_ptr(wasm_instrs);
    wasm_instrs.push(WasmInstruction::I32Load {
        mem_arg: MemArg::zero(),
    });
    // set the frame ptr
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

pub fn load_stack_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: STACK_PTR_ADDR as i32,
    });
    // load
    wasm_instrs.push(WasmInstruction::I32Load {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

pub fn increment_stack_ptr_by_known_offset(
    offset: u32,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: STACK_PTR_ADDR as i32,
    });
    // add offset to stack pointer
    load_stack_ptr(wasm_instrs);
    wasm_instrs.push(WasmInstruction::I32Const { n: offset as i32 });
    wasm_instrs.push(WasmInstruction::I32Add);
    // store stack pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });

    // log stack ptr every time we change it
    log_stack_ptr(wasm_instrs, module_context);
}

pub fn increment_stack_ptr_dynamic(
    mut load_byte_size_instrs: Vec<WasmInstruction>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: STACK_PTR_ADDR as i32,
    });

    // load stack pointer and byte size, and add them together
    load_stack_ptr(wasm_instrs);
    wasm_instrs.append(&mut load_byte_size_instrs);
    wasm_instrs.push(WasmInstruction::I32Add);

    // store to stack pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });

    // log stack ptr every time we change it
    log_stack_ptr(wasm_instrs, module_context);
}

pub fn set_stack_ptr_to_frame_ptr(
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: STACK_PTR_ADDR as i32,
    });
    // load frame pointer value, to store in stack pointer
    load_frame_ptr(wasm_instrs);
    // store stack pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });

    // log stack ptr every time we change it
    log_stack_ptr(wasm_instrs, module_context);
}

fn set_temp_frame_ptr_to_stack_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: TEMP_FRAME_PTR_ADDR as i32,
    });
    // load stack pointer value, to store in temp frame pointer
    load_stack_ptr(wasm_instrs);
    // store temp frame pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

fn set_frame_ptr_to_temp_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: FRAME_PTR_ADDR as i32,
    });
    // load stack pointer value, to store in temp frame pointer
    load_temp_frame_ptr(wasm_instrs);
    // store frame pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

pub fn set_frame_ptr_to_stack_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: FRAME_PTR_ADDR as i32,
    });
    // load stack pointer value, to store in frame pointer
    load_stack_ptr(wasm_instrs);
    // store frame pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

pub fn set_up_new_stack_frame(
    callee_function_type: &IrType,
    params: Vec<Src>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) {
    // create a new stack frame for the callee
    //
    // ----------------------
    // | previous frame ptr | <-- frame ptr
    // | return value       |
    // | ------------------ |
    // | | params         | |
    // | ------------------ |
    // | ------------------ |
    // | | vars           | |
    // | ------------------ |
    // ---------------------- <-- stack ptr
    //

    // store frame pointer at start of the stack frame
    // address operand for storing frame ptr (current top of stack)
    load_stack_ptr(wasm_instrs);
    // value to store: current value of frame ptr
    load_frame_ptr(wasm_instrs);
    // store frame ptr to start of new stack frame
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg::zero(),
    });

    // save the address of the start of the new stack frame
    set_temp_frame_ptr_to_stack_ptr(wasm_instrs);

    // increment stack pointer
    increment_stack_ptr_by_known_offset(PTR_SIZE, wasm_instrs, module_context);

    let (return_type, param_types) = match callee_function_type {
        IrType::Function(return_type, param_types, _is_variadic) => (return_type, param_types),
        _ => unreachable!(),
    };

    info!("return type: {:?}", return_type);
    info!("param type: {:?}", param_types);

    // leave space for the return value
    let return_type_byte_size = match return_type.get_byte_size(prog_metadata) {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => {
            unreachable!()
        }
    };
    increment_stack_ptr_by_known_offset(return_type_byte_size as u32, wasm_instrs, module_context);

    // store function parameters in callee's stack frame
    for (param_index, param) in params.into_iter().enumerate() {
        match param {
            Src::Var(var_id) => {
                // address operand for where to store param
                load_stack_ptr(wasm_instrs);

                let var_type = prog_metadata.get_var_type(&var_id).unwrap();
                let var_byte_size = var_type
                    .get_byte_size(prog_metadata)
                    .get_compile_time_value()
                    .unwrap();

                info!(
                    "storing param of type {:?} with size {} bytes",
                    var_type, var_byte_size
                );

                // load var onto the wasm stack (value to store)
                load_var(var_id, wasm_instrs, function_context, prog_metadata);

                // store param
                store(var_type, wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr_by_known_offset(
                    var_byte_size as u32,
                    wasm_instrs,
                    module_context,
                );
            }
            Src::Constant(constant) => {
                // address operand for where to store param
                load_stack_ptr(wasm_instrs);

                let param_type = if param_index >= param_types.len() {
                    constant.get_type_minimum_i32()
                } else {
                    param_types.get(param_index).unwrap().to_owned()
                };

                let param_byte_size = param_type
                    .get_byte_size(prog_metadata)
                    .get_compile_time_value()
                    .unwrap();

                // value to store
                load_constant(constant, param_type.to_owned(), wasm_instrs);

                // store
                store(param_type.to_owned(), wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr_by_known_offset(
                    param_byte_size as u32,
                    wasm_instrs,
                    module_context,
                );
            }
            Src::StoreAddressVar(_) | Src::Fun(_) => {
                unreachable!()
            }
        }
    }

    // set the frame pointer to point at the new stack frame
    set_frame_ptr_to_temp_frame_ptr(wasm_instrs);
}

pub fn pop_stack_frame(
    result_dest: Dest,
    callee_function_type: &IrType,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) {
    // pop the top stack frame
    // restore the stack pointer value
    set_stack_ptr_to_frame_ptr(wasm_instrs, module_context);
    // restore the previous frame ptr value
    restore_previous_frame_ptr(wasm_instrs);

    // store the result to dest
    //
    // return value is stored in the stack frame we're popping, after the previous frame ptr
    // so, at stack ptr + PTR_SIZE
    // address operand for loading return value
    let mut store_value_instrs = Vec::new();
    load_stack_ptr(&mut store_value_instrs);
    store_value_instrs.push(WasmInstruction::I32Const { n: PTR_SIZE as i32 });
    store_value_instrs.push(WasmInstruction::I32Add);

    let return_type = match callee_function_type {
        IrType::Function(return_type, _, _) => &**return_type,
        _ => unreachable!(),
    };
    match return_type {
        IrType::Void => {
            // if function returns void, don't load return value
        }
        _ => {
            load(return_type.to_owned(), &mut store_value_instrs);

            store_var(
                result_dest,
                store_value_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
    }
}

pub fn overwrite_current_stack_frame_with_new_stack_frame(
    callee_function_type: &IrType,
    params: Vec<Src>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) {
    // leave frame ptr where it is

    let (return_type, param_types) = match callee_function_type {
        IrType::Function(return_type, param_types, _is_variadic) => (&**return_type, param_types),
        _ => unreachable!(),
    };

    // first copy params to temp space at top of stack, to avoid overwriting them
    // before we copy them to their new param position
    // use temp_fp to hold the stack ptr value, so we can offset from it while
    // constructing the new stack frame
    // todo make sure this is definitely after the space for all the new params
    set_temp_frame_ptr_to_stack_ptr(wasm_instrs);

    let mut temp_stack_ptr_offset = 0;
    let mut param_var_stack_ptr_offsets = HashMap::new();
    for param_index in 0..params.len() {
        let param = params.get(param_index).unwrap();
        if let Src::Var(var_id) = param {
            // offset from top of stack to store param
            load_temp_frame_ptr(wasm_instrs);
            wasm_instrs.push(WasmInstruction::I32Const {
                n: temp_stack_ptr_offset as i32,
            });
            wasm_instrs.push(WasmInstruction::I32Add);

            let var_type = prog_metadata.get_var_type(var_id).unwrap();
            let var_byte_size = var_type
                .get_byte_size(prog_metadata)
                .get_compile_time_value()
                .unwrap();

            load_var(
                var_id.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );

            // store param in temp space at top of stack
            store(var_type, wasm_instrs);

            param_var_stack_ptr_offsets.insert(var_id.to_owned(), temp_stack_ptr_offset);

            // increment offset
            temp_stack_ptr_offset += var_byte_size;
        }
    }

    // reset stack ptr for setting up the new stack frame
    set_stack_ptr_to_frame_ptr(wasm_instrs, module_context);
    increment_stack_ptr_by_known_offset(PTR_SIZE, wasm_instrs, module_context);

    // leave space for the return value
    let return_type_byte_size = match return_type.get_byte_size(prog_metadata) {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => {
            unreachable!()
        }
    };
    increment_stack_ptr_by_known_offset(return_type_byte_size as u32, wasm_instrs, module_context);

    // now store the params in their correct positions in the new stack frame
    for param_index in 0..params.len() {
        let param = params.get(param_index).unwrap();
        match param {
            Src::Var(var_id) => {
                // address operand for where to store param
                load_stack_ptr(wasm_instrs);

                let var_type = prog_metadata.get_var_type(var_id).unwrap();
                let var_byte_size = var_type
                    .get_byte_size(prog_metadata)
                    .get_compile_time_value()
                    .unwrap();

                // load var from temp space we put it in earlier
                load_temp_frame_ptr(wasm_instrs);
                let temp_frame_ptr_offset = *param_var_stack_ptr_offsets.get(var_id).unwrap();
                wasm_instrs.push(WasmInstruction::I32Const {
                    n: temp_frame_ptr_offset as i32,
                });
                wasm_instrs.push(WasmInstruction::I32Add);

                load(var_type.to_owned(), wasm_instrs);

                // store param
                store(var_type, wasm_instrs);

                // advance stack ptr
                increment_stack_ptr_by_known_offset(
                    var_byte_size as u32,
                    wasm_instrs,
                    module_context,
                );
            }
            Src::Constant(constant) => {
                // address operand for where to store param
                load_stack_ptr(wasm_instrs);

                let param_type = if param_index >= param_types.len() {
                    constant.get_type_minimum_i32()
                } else {
                    param_types.get(param_index).unwrap().to_owned()
                };

                let param_byte_size = param_type
                    .get_byte_size(prog_metadata)
                    .get_compile_time_value()
                    .unwrap();

                // value to store
                load_constant(constant.to_owned(), param_type.to_owned(), wasm_instrs);

                // store
                store(param_type.to_owned(), wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr_by_known_offset(
                    param_byte_size as u32,
                    wasm_instrs,
                    module_context,
                );
            }
            Src::StoreAddressVar(_) | Src::Fun(_) => {
                unreachable!()
            }
        }
    }
}
