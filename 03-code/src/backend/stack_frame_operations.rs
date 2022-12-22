use crate::backend::memory_operations::{load, load_constant, load_var, store, store_var};
use crate::backend::target_code_generation::{FRAME_PTR_ADDR, PTR_SIZE, STACK_PTR_ADDR};
use crate::backend::target_code_generation_context::FunctionContext;
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

pub fn increment_stack_ptr_by_known_offset(offset: u32, wasm_instrs: &mut Vec<WasmInstruction>) {
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
}

pub fn increment_stack_ptr_dynamic(
    mut load_byte_size_instrs: Vec<WasmInstruction>,
    wasm_instrs: &mut Vec<WasmInstruction>,
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
}

pub fn set_stack_ptr_to_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
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
    callee_function_type: &Box<IrType>,
    params: Vec<Src>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &Box<ProgramMetadata>,
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

    // set the frame pointer to point at the new stack frame
    set_frame_ptr_to_stack_ptr(wasm_instrs);

    // increment stack pointer
    increment_stack_ptr_by_known_offset(PTR_SIZE, wasm_instrs);

    let (return_type, param_types) = match &**callee_function_type {
        IrType::Function(return_type, param_types, _is_variadic) => (return_type, param_types),
        _ => unreachable!(),
    };

    // leave space for the return value
    let return_type_byte_size = match return_type.get_byte_size(prog_metadata) {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => {
            unreachable!()
        }
    };
    increment_stack_ptr_by_known_offset(return_type_byte_size as u32, wasm_instrs);

    // store function parameters in callee's stack frame
    let mut param_index = 0;
    for param in params {
        match param {
            Src::Var(var_id) => {
                // address operand for where to store param
                load_stack_ptr(wasm_instrs);

                let var_type = prog_metadata.get_var_type(&var_id).unwrap();
                let var_byte_size = var_type
                    .get_byte_size(&prog_metadata)
                    .get_compile_time_value()
                    .unwrap();

                // load var onto the wasm stack (value to store)
                load_var(var_id, wasm_instrs, function_context, prog_metadata);

                // store param
                println!("storing param in stack frame");
                store(var_type, wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr_by_known_offset(var_byte_size as u32, wasm_instrs);
            }
            Src::Constant(constant) => {
                // address operand for where to store param
                load_stack_ptr(wasm_instrs);

                // value to store
                load_constant(constant, wasm_instrs);

                // store
                let param_type = param_types.get(param_index).unwrap();
                let param_byte_size = param_type
                    .get_byte_size(prog_metadata)
                    .get_compile_time_value()
                    .unwrap();
                println!("storing constant param in stack frame");
                store(param_type.to_owned(), wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr_by_known_offset(param_byte_size as u32, wasm_instrs);
            }
            Src::StoreAddressVar(_) | Src::Fun(_) => {
                unreachable!()
            }
        }
        param_index += 1;
    }
}

pub fn pop_stack_frame(
    result_dest: Dest,
    callee_function_type: &Box<IrType>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    // pop the top stack frame
    // restore the stack pointer value
    set_stack_ptr_to_frame_ptr(wasm_instrs);
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

    let return_type = match &**callee_function_type {
        IrType::Function(return_type, _, _) => return_type,
        _ => unreachable!(),
    };
    load(return_type.to_owned(), &mut store_value_instrs);

    store_var(
        result_dest,
        store_value_instrs,
        wasm_instrs,
        function_context,
        prog_metadata,
    );
}
