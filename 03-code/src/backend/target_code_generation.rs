use crate::backend::wasm_indices::{FuncIdx, LocalIdx};
use crate::backend::wasm_instructions::{MemArg, WasmExpression, WasmInstruction};
use crate::backend::wasm_program::{WasmFunction, WasmProgram};
use crate::middle_end::ids::{FunId, VarId};
use crate::middle_end::instructions::{Constant, Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::relooper::blocks::Block;
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};
use std::collections::HashMap;

const PTR_SIZE: u32 = 4;

const FRAME_PTR_ADDR: i32 = 0;
// const STACK_PTR_ADDR: u32 = FRAME_PTR_ADDR + PTR_SIZE;

pub fn generate_target_code(prog: ReloopedProgram) -> WasmProgram {
    let mut wasm_program = WasmProgram::new();

    for (fun_id, function) in prog.program_blocks.functions {
        let stack_frame_context = StackFrameContext::new();

        // let mut function_instrs = Vec::new();

        // store current value of frame ptr on top of stack
        // allocate space for return value
        // push params onto stack
    }

    wasm_program
}

// fn generate_target_function(relooped_function: ReloopedFunction) -> WasmFunction {}

// fn generate_function_body_instructions(block: Box<Block>) -> Vec<WasmInstruction> {
//     match *block {
//         Block::Simple { internal, next } => {}
//         Block::Loop { id, inner, next } => {}
//         Block::Multiple {
//             id,
//             handled_blocks,
//             next,
//         } => {}
//     }
// }

struct ModuleContext {
    func_idx_mappings: HashMap<FunId, FuncIdx>, // todo we need to calculate these before we convert the instrs
}

struct StackFrameContext {
    var_fp_offsets: HashMap<VarId, u32>,
    return_value_fp_offset: u32,
    top_of_stack_fp_offset: u32,
}

impl StackFrameContext {
    fn new() -> Self {
        StackFrameContext {
            var_fp_offsets: HashMap::new(),
            return_value_fp_offset: PTR_SIZE,
            top_of_stack_fp_offset: 0,
        }
    }
}

fn convert_ir_instr_to_wasm(
    instr: Instruction,
    stack_frame_context: &mut StackFrameContext,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> Vec<WasmInstruction> {
    let mut wasm_instrs = Vec::new();

    match instr {
        Instruction::SimpleAssignment(dest, src) => {
            // let dest_type = program_metadata.get_var_type(&dest).unwrap();
            //
            // let dest_offset = match context.var_fp_offsets.get(&dest) {
            //     None => {
            //         // allocate variable on top of stack
            //         let offset = context.top_of_stack_fp_offset;
            //         context
            //             .var_fp_offsets
            //             .insert(dest.to_owned(), context.top_of_stack_fp_offset);
            //         context.top_of_stack_fp_offset +=
            //             dest_type.get_byte_size(&program_metadata);
            //         offset
            //     }
            //     Some(offset) => offset,
            // };
            //
            // match src {
            //     Src::Var(var_id) => {
            //         let src_offset = context.var_fp_offsets.get(&var_id);
            //     }
            //     Src::Constant(_) => {}
            //     Src::StoreAddressVar(_) | Src::Fun(_) => {
            //         unreachable!()
            //     }
            // }
            //
            // match *dest_type {
            //     IrType::I8 => {}
            //     IrType::U8 => {}
            //     IrType::I16 => {}
            //     IrType::U16 => {}
            //     IrType::I32 => {}
            //     IrType::U32 => {}
            //     IrType::I64 => {}
            //     IrType::U64 => {}
            //     IrType::F32 => {}
            //     IrType::F64 => {}
            //     IrType::Struct(_) => {}
            //     IrType::Union(_) => {}
            //     IrType::Void => {}
            //     IrType::PointerTo(_) => {}
            //     IrType::ArrayOf(_, _) => {}
            //     IrType::Function(_, _, _) => {}
            // }
        }
        Instruction::LoadFromAddress(_, _) => {}
        Instruction::StoreToAddress(_, _) => {}
        Instruction::AllocateVariable(_, _) => {}
        Instruction::AddressOf(_, _) => {}
        Instruction::BitwiseNot(_, _) => {}
        Instruction::LogicalNot(_, _) => {}
        Instruction::Mult(_, _, _) => {}
        Instruction::Div(_, _, _) => {}
        Instruction::Mod(_, _, _) => {}
        Instruction::Add(dest, left, right) => {
            // // push left
            // match left {
            //     Src::Var(var_id) => match context.var_to_local_mappings.get(&var_id) {
            //         Some(local_idx) => wasm_instrs.push(WasmInstruction::LocalGet {
            //             local_idx: local_idx.to_owned(),
            //         }),
            //         None => {
            //             unreachable!("trying to access an undefined var")
            //         }
            //     },
            //     Src::Constant(constant) => {
            //         // push constant
            //         let dest_type = program_metadata.get_var_type(&dest).unwrap();
            //         todo!()
            //     }
            //     Src::StoreAddressVar(_) | Src::Fun(_) => {
            //         unreachable!()
            //     }
            // }
            // // push right
            // match right {
            //     Src::Var(var_id) => match context.var_to_local_mappings.get(&var_id) {
            //         Some(local_idx) => wasm_instrs.push(WasmInstruction::LocalGet {
            //             local_idx: local_idx.to_owned(),
            //         }),
            //         None => {
            //             unreachable!("trying to access an undefined var")
            //         }
            //     },
            //     Src::Constant(constant) => {
            //         // push constant
            //         let dest_type = program_metadata.get_var_type(&dest).unwrap();
            //         todo!()
            //     }
            //     Src::StoreAddressVar(_) | Src::Fun(_) => {
            //         unreachable!()
            //     }
            // }
            // // add
            // wasm_instrs.push(WasmInstruction::Add)
            // // store result
        }
        Instruction::Sub(_, _, _) => {}
        Instruction::LeftShift(_, _, _) => {}
        Instruction::RightShift(_, _, _) => {}
        Instruction::BitwiseAnd(_, _, _) => {}
        Instruction::BitwiseOr(_, _, _) => {}
        Instruction::BitwiseXor(_, _, _) => {}
        Instruction::LogicalAnd(_, _, _) => {}
        Instruction::LogicalOr(_, _, _) => {}
        Instruction::LessThan(_, _, _) => {}
        Instruction::GreaterThan(_, _, _) => {}
        Instruction::LessThanEq(_, _, _) => {}
        Instruction::GreaterThanEq(_, _, _) => {}
        Instruction::Equal(_, _, _) => {}
        Instruction::NotEqual(_, _, _) => {}
        Instruction::Call(dest, fun_id, params) => {
            let callee_function_type = prog_metadata.function_types.get(&fun_id).unwrap();

            set_up_new_stack_frame(
                callee_function_type,
                params,
                &mut wasm_instrs,
                stack_frame_context,
                prog_metadata,
            );

            // call the function
            wasm_instrs.push(WasmInstruction::Call {
                func_idx: module_context
                    .func_idx_mappings
                    .get(&fun_id)
                    .unwrap()
                    .to_owned(),
            });

            // store result to dest
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            store(dest_type, &mut wasm_instrs);
        }
        Instruction::Ret(_) => {}
        Instruction::Label(_) => {}
        Instruction::Br(_) => {}
        Instruction::BrIfEq(_, _, _) => {}
        Instruction::BrIfNotEq(_, _, _) => {}
        Instruction::PointerToStringLiteral(_, _) => {}
        Instruction::I8toI16(_, _) => {}
        Instruction::I8toU16(_, _) => {}
        Instruction::U8toI16(_, _) => {}
        Instruction::U8toU16(_, _) => {}
        Instruction::I16toI32(_, _) => {}
        Instruction::U16toI32(_, _) => {}
        Instruction::I16toU32(_, _) => {}
        Instruction::U16toU32(_, _) => {}
        Instruction::I32toU32(_, _) => {}
        Instruction::I32toU64(_, _) => {}
        Instruction::U32toU64(_, _) => {}
        Instruction::I64toU64(_, _) => {}
        Instruction::I32toI64(_, _) => {}
        Instruction::U32toI64(_, _) => {}
        Instruction::U32toF32(_, _) => {}
        Instruction::I32toF32(_, _) => {}
        Instruction::U64toF32(_, _) => {}
        Instruction::I64toF32(_, _) => {}
        Instruction::U32toF64(_, _) => {}
        Instruction::I32toF64(_, _) => {}
        Instruction::U64toF64(_, _) => {}
        Instruction::I64toF64(_, _) => {}
        Instruction::F32toF64(_, _) => {}
        Instruction::I32toI8(_, _) => {}
        Instruction::U32toI8(_, _) => {}
        Instruction::I64toI8(_, _) => {}
        Instruction::U64toI8(_, _) => {}
        Instruction::I32toU8(_, _) => {}
        Instruction::U32toU8(_, _) => {}
        Instruction::I64toU8(_, _) => {}
        Instruction::U64toU8(_, _) => {}
        Instruction::I64toI32(_, _) => {}
        Instruction::U64toI32(_, _) => {}
        Instruction::U32toPtr(_, _) => {}
        Instruction::I32toPtr(_, _) => {}
        Instruction::Nop => {}
        Instruction::Break(_) => {}
        Instruction::Continue(_) => {}
        Instruction::EndHandledBlock(_) => {}
        Instruction::IfEqElse(_, _, _, _) => {}
        Instruction::IfNotEqElse(_, _, _, _) => {}
    }

    wasm_instrs
}

fn load_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const { n: FRAME_PTR_ADDR });
    // load
    wasm_instrs.push(WasmInstruction::I32Load {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

fn increment_frame_pointer(offset: u32, wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const { n: FRAME_PTR_ADDR });
    // add offset to frame pointer
    load_frame_ptr(wasm_instrs);
    wasm_instrs.push(WasmInstruction::I32Const { n: offset as i32 });
    wasm_instrs.push(WasmInstruction::I32Add);
    // store frame pointer
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });
}

/// Insert instructions to load the given variable onto the wasm stack
fn load_var(
    var_id: VarId,
    wasm_instrs: &mut Vec<WasmInstruction>,
    stack_frame_context: &StackFrameContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    match stack_frame_context.var_fp_offsets.get(&var_id) {
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
            match *prog_metadata.get_var_type(&var_id).unwrap() {
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
    }
}

/// Insert a store instruction of the correct type
fn store(value_type: Box<IrType>, wasm_instrs: &mut Vec<WasmInstruction>) {
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

fn load_constant(constant: Constant, wasm_instrs: &mut Vec<WasmInstruction>) {
    todo!("push constant to wasm stack with the right type")
}

fn set_up_new_stack_frame(
    callee_function_type: &Box<IrType>,
    params: Vec<Src>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    stack_frame_context: &StackFrameContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    // create a new stack frame for the callee
    // ----------------------
    // | previous frame ptr |
    // | return value       |
    // | ------------------ |
    // | | params         | |
    // | ------------------ |
    // | ------------------ |
    // | | vars           | |
    // | ------------------ |
    // ----------------------
    let mut new_stack_frame_fp_offset = stack_frame_context.top_of_stack_fp_offset;
    let frame_ptr_increment_value = new_stack_frame_fp_offset;

    // store frame pointer at start of the stack frame
    // address operand for storing frame ptr
    load_frame_ptr(wasm_instrs);
    wasm_instrs.push(WasmInstruction::I32Const {
        n: new_stack_frame_fp_offset as i32,
    });
    wasm_instrs.push(WasmInstruction::I32Add);
    // value to store: current value of frame ptr
    load_frame_ptr(wasm_instrs);
    // store frame ptr to start of new stack frame
    wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg {
            align: 2,
            offset: 0,
        },
    });

    new_stack_frame_fp_offset += PTR_SIZE;

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
    new_stack_frame_fp_offset += return_type_byte_size as u32;

    // store function parameters in callee's stack frame
    let mut param_index = 0;
    for param in params {
        match param {
            Src::Var(var_id) => {
                // address operand for where to store param
                load_frame_ptr(wasm_instrs);
                wasm_instrs.push(WasmInstruction::I32Const {
                    n: new_stack_frame_fp_offset as i32,
                });
                wasm_instrs.push(WasmInstruction::I32Add);

                let var_type = prog_metadata.get_var_type(&var_id).unwrap();
                let var_byte_size = var_type
                    .get_byte_size(&prog_metadata)
                    .get_compile_time_value()
                    .unwrap();

                // load var onto the wasm stack (value to store)
                load_var(var_id, wasm_instrs, stack_frame_context, prog_metadata);

                // store param
                store(var_type, wasm_instrs);

                // advance the frame pointer offset
                new_stack_frame_fp_offset += var_byte_size as u32;
            }
            Src::Constant(constant) => {
                // address operand for where to store param
                load_frame_ptr(wasm_instrs);
                wasm_instrs.push(WasmInstruction::I32Const {
                    n: new_stack_frame_fp_offset as i32,
                });
                wasm_instrs.push(WasmInstruction::I32Add);

                // value to store
                load_constant(constant, wasm_instrs);

                // store
                let param_type = param_types.get(param_index).unwrap();
                let param_byte_size = param_type
                    .get_byte_size(prog_metadata)
                    .get_compile_time_value()
                    .unwrap();
                store(param_type.to_owned(), wasm_instrs);

                new_stack_frame_fp_offset += param_byte_size as u32;
            }
            Src::StoreAddressVar(_) | Src::Fun(_) => {
                unreachable!()
            }
        }
        param_index += 1;
    }

    // set the frame pointer to point at the new stack frame
    increment_frame_pointer(frame_ptr_increment_value, wasm_instrs);
}
