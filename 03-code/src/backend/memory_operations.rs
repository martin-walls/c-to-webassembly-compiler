use crate::backend::stack_frame_operations::load_frame_ptr;
use crate::backend::target_code_generation_context::FunctionContext;
use crate::backend::wasm_instructions::{MemArg, WasmInstruction};
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::{Constant, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;

/// Insert a load instruction of the correct type
pub fn load(value_type: Box<IrType>, wasm_instrs: &mut Vec<WasmInstruction>) {
    match *value_type {
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
        IrType::I32 | IrType::U32 | IrType::PointerTo(_) => {
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
        _ => unreachable!(),
    }
}

/// Insert a store instruction of the correct type
pub fn store(value_type: Box<IrType>, wasm_instrs: &mut Vec<WasmInstruction>) {
    match *value_type {
        IrType::I8 | IrType::U8 => wasm_instrs.push(WasmInstruction::I32Store8 {
            mem_arg: MemArg::zero(),
        }),
        IrType::I16 | IrType::U16 => wasm_instrs.push(WasmInstruction::I32Store16 {
            mem_arg: MemArg::zero(),
        }),
        IrType::I32 | IrType::U32 | IrType::PointerTo(_) => {
            wasm_instrs.push(WasmInstruction::I32Store {
                mem_arg: MemArg::zero(),
            });
        }
        IrType::I64 | IrType::U64 => wasm_instrs.push(WasmInstruction::I64Store {
            mem_arg: MemArg::zero(),
        }),
        IrType::F32 => wasm_instrs.push(WasmInstruction::F32Store {
            mem_arg: MemArg::zero(),
        }),
        IrType::F64 => wasm_instrs.push(WasmInstruction::F64Store {
            mem_arg: MemArg::zero(),
        }),
        _ => unreachable!(),
    }
}

/// Insert instructions to load the given variable onto the wasm stack
pub fn load_var(
    var_id: VarId,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    match function_context.var_fp_offsets.get(&var_id) {
        None => {
            // todo check if var is a global variable, and calculate the address from that
        }
        Some(fp_offset) => {
            // calculate address operand
            load_frame_ptr(wasm_instrs);
            wasm_instrs.push(WasmInstruction::I32Const {
                n: *fp_offset as i32,
            });
            wasm_instrs.push(WasmInstruction::I32Add);

            // load
            load(prog_metadata.get_var_type(&var_id).unwrap(), wasm_instrs);
        }
    }
}

/// Insert instructions to store into the given variable
pub fn store_var(
    var_id: VarId,
    mut store_value_instrs: Vec<WasmInstruction>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    match function_context.var_fp_offsets.get(&var_id) {
        None => {
            // todo check if var is a global variable, and calculate the address from that
        }
        Some(fp_offset) => {
            // calculate address operand
            load_frame_ptr(wasm_instrs);
            wasm_instrs.push(WasmInstruction::I32Const {
                n: *fp_offset as i32,
            });
            wasm_instrs.push(WasmInstruction::I32Add);

            wasm_instrs.append(&mut store_value_instrs);

            // store
            store(prog_metadata.get_var_type(&var_id).unwrap(), wasm_instrs);
        }
    }
}

pub fn load_constant(constant: Constant, wasm_instrs: &mut Vec<WasmInstruction>) {
    let constant_type = constant.get_type(None);
    match *constant_type {
        IrType::I8 | IrType::U8 | IrType::I16 | IrType::U16 | IrType::I32 | IrType::U32 => {
            match constant {
                Constant::Int(n) => wasm_instrs.push(WasmInstruction::I32Const { n: n as i32 }),
                Constant::Float(_) => {
                    unreachable!()
                }
            }
        }
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
        _ => {
            unreachable!()
        }
    }
}

pub fn load_src(
    src: Src,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    match src {
        Src::Var(var_id) => load_var(var_id, wasm_instrs, function_context, prog_metadata),
        Src::Constant(constant) => load_constant(constant, wasm_instrs),
        Src::StoreAddressVar(_) | Src::Fun(_) => {
            unreachable!()
        }
    }
}
