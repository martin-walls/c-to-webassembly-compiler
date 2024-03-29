use log::debug;

use crate::back_end::stack_frame_operations::load_frame_ptr;
use crate::back_end::target_code_generation_context::FunctionContext;
use crate::back_end::wasm_instructions::{MemArg, WasmInstruction};
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::{Constant, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;

/// Insert a load instruction of the correct type
pub fn load(value_type: IrType, wasm_instrs: &mut Vec<WasmInstruction>) {
    match value_type {
        IrType::I8 => wasm_instrs.push(WasmInstruction::I32Load8S {
            mem_arg: MemArg::zero(),
        }),

        IrType::U8 => wasm_instrs.push(WasmInstruction::I32Load8U {
            mem_arg: MemArg::zero(),
        }),
        IrType::I16 => wasm_instrs.push(WasmInstruction::I32Load16S {
            mem_arg: MemArg::zero(),
        }),
        IrType::U16 => wasm_instrs.push(WasmInstruction::I32Load16U {
            mem_arg: MemArg::zero(),
        }),
        IrType::I32 | IrType::U32 | IrType::PointerTo(_) | IrType::ArrayOf(_, _) => {
            wasm_instrs.push(WasmInstruction::I32Load {
                mem_arg: MemArg::zero(),
            });
        }
        IrType::I64 | IrType::U64 => wasm_instrs.push(WasmInstruction::I64Load {
            mem_arg: MemArg::zero(),
        }),
        IrType::F32 => wasm_instrs.push(WasmInstruction::F32Load {
            mem_arg: MemArg::zero(),
        }),
        IrType::F64 => wasm_instrs.push(WasmInstruction::F64Load {
            mem_arg: MemArg::zero(),
        }),
        _ => {
            unreachable!()
        }
    }
}

/// Insert a store instruction of the correct type
pub fn store(value_type: IrType, wasm_instrs: &mut Vec<WasmInstruction>) {
    match value_type {
        IrType::I8 | IrType::U8 => wasm_instrs.push(WasmInstruction::I32Store8 {
            mem_arg: MemArg::zero(),
        }),
        IrType::I16 | IrType::U16 => wasm_instrs.push(WasmInstruction::I32Store16 {
            mem_arg: MemArg::zero(),
        }),
        IrType::I32 | IrType::U32 | IrType::PointerTo(_) | IrType::ArrayOf(_, _) => {
            // if storing to an 'array' type, it's storing a pointer to the array
            wasm_instrs.push(WasmInstruction::I32Store {
                mem_arg: MemArg::zero(),
            });
        }
        IrType::I64 | IrType::U64 => {
            wasm_instrs.push(WasmInstruction::I64Store {
                mem_arg: MemArg::zero(),
            });
        }
        IrType::F32 => wasm_instrs.push(WasmInstruction::F32Store {
            mem_arg: MemArg::zero(),
        }),
        IrType::F64 => wasm_instrs.push(WasmInstruction::F64Store {
            mem_arg: MemArg::zero(),
        }),
        t => {
            debug!("store type: {}", t);
            unreachable!()
        }
    }
}

/// load the memory address of the given variable onto the stack
pub fn load_var_address(
    var_id: &VarId,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &ProgramMetadata,
) {
    if prog_metadata.is_var_the_null_dest(var_id) {
        return;
    }

    match function_context.var_fp_offsets.get(var_id) {
        None => {
            match function_context.global_var_addrs.get(var_id) {
                None => {
                    debug!("var_id: {}", var_id);
                    unreachable!("Every var is either local or global")
                }
                Some(global_addr) => {
                    // address of global var
                    wasm_instrs.push(WasmInstruction::I32Const {
                        n: *global_addr as i32,
                    });
                }
            }
        }
        Some(fp_offset) => {
            // load frame ptr and add variable offset
            load_frame_ptr(wasm_instrs);
            wasm_instrs.push(WasmInstruction::I32Const {
                n: *fp_offset as i32,
            });
            wasm_instrs.push(WasmInstruction::I32Add);
        }
    }
}

/// Insert instructions to load the given variable onto the wasm stack
pub fn load_var(
    var_id: VarId,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &ProgramMetadata,
) {
    if prog_metadata.is_var_the_null_dest(&var_id) {
        return;
    }

    let var_type = prog_metadata.get_var_type(&var_id).unwrap();

    load_var_address(&var_id, wasm_instrs, function_context, prog_metadata);

    load(var_type, wasm_instrs);
}

/// Insert instructions to store into the given variable
pub fn store_var(
    var_id: VarId,
    mut store_value_instrs: Vec<WasmInstruction>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &ProgramMetadata,
) {
    // ignore stores to the null dest
    if prog_metadata.is_var_the_null_dest(&var_id) {
        return;
    }

    // address operand
    load_var_address(&var_id, wasm_instrs, function_context, prog_metadata);

    // put the value to store onto the stack
    wasm_instrs.append(&mut store_value_instrs);

    // store
    store(prog_metadata.get_var_type(&var_id).unwrap(), wasm_instrs);
}

pub fn load_constant(
    constant: Constant,
    constant_type: IrType,
    wasm_instrs: &mut Vec<WasmInstruction>,
) {
    // let constant_type = constant.get_type(None);
    match constant_type {
        IrType::I8
        | IrType::U8
        | IrType::I16
        | IrType::U16
        | IrType::I32
        | IrType::U32
        | IrType::PointerTo(_) => match constant {
            Constant::Int(n) => wasm_instrs.push(WasmInstruction::I32Const { n: n as i32 }),
            Constant::Float(_) => {
                unreachable!()
            }
        },
        IrType::I64 | IrType::U64 => match constant {
            Constant::Int(n) => wasm_instrs.push(WasmInstruction::I64Const { n: n as i64 }),
            Constant::Float(_) => {
                unreachable!()
            }
        },
        IrType::F32 => match constant {
            Constant::Float(z) => wasm_instrs.push(WasmInstruction::F32Const { z: z as f32 }),
            Constant::Int(_) => {
                unreachable!()
            }
        },
        IrType::F64 => match constant {
            Constant::Float(z) => wasm_instrs.push(WasmInstruction::F64Const { z }),
            Constant::Int(_) => {
                unreachable!()
            }
        },
        t => {
            debug!("{}", t);
            unreachable!()
        }
    }
}

/// Param dest_type is optional for var loads, but required for constant loads.
pub fn load_src(
    src: Src,
    dest_type: IrType,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &ProgramMetadata,
) {
    match src {
        Src::Var(var_id) => load_var(var_id, wasm_instrs, function_context, prog_metadata),
        Src::Constant(constant) => load_constant(constant, dest_type, wasm_instrs),
        Src::StoreAddressVar(_) | Src::Fun(_) => {
            unreachable!()
        }
    }
}
