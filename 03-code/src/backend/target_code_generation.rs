use crate::backend::wasm_indices::{FuncIdx, LocalIdx};
use crate::backend::wasm_instructions::{BlockType, MemArg, WasmExpression, WasmInstruction};
use crate::backend::wasm_program::{WasmFunction, WasmProgram};
use crate::middle_end::ids::{FunId, VarId};
use crate::middle_end::instructions::{Constant, Dest, Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::relooper::blocks::Block;
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};
use std::collections::HashMap;

const PTR_SIZE: u32 = 4;

const FRAME_PTR_ADDR: u32 = 0;
const STACK_PTR_ADDR: u32 = FRAME_PTR_ADDR + PTR_SIZE;

pub fn generate_target_code(prog: ReloopedProgram) -> WasmProgram {
    // let mut wasm_program = WasmProgram::new();

    for (fun_id, function) in prog.program_blocks.functions {
        if let Some(block) = function.block {
            println!("calculating var offsets for function {}", fun_id);
            let var_offsets =
                calculate_var_offsets_from_fp(&block, function.type_info, &prog.program_metadata);

            let mut function_context =
                FunctionContext::new(var_offsets, function.label_variable.unwrap());
        }

        // let mut function_instrs = Vec::new();

        // store current value of frame ptr on top of stack
        // allocate space for return value
        // push params onto stack
    }

    todo!()
}

fn get_vars_with_types(
    block: &Box<Block>,
    prog_metadata: &Box<ProgramMetadata>,
) -> Vec<(VarId, Box<IrType>)> {
    match &**block {
        Block::Simple { internal, next } => {
            let mut vars = Vec::new();

            for instr in &internal.instrs {
                match instr {
                    Instruction::SimpleAssignment(dest, _)
                    | Instruction::LoadFromAddress(dest, _)
                    | Instruction::StoreToAddress(dest, _)
                    | Instruction::AllocateVariable(dest, _)
                    | Instruction::AddressOf(dest, _)
                    | Instruction::BitwiseNot(dest, _)
                    | Instruction::LogicalNot(dest, _)
                    | Instruction::Mult(dest, _, _)
                    | Instruction::Div(dest, _, _)
                    | Instruction::Mod(dest, _, _)
                    | Instruction::Add(dest, _, _)
                    | Instruction::Sub(dest, _, _)
                    | Instruction::LeftShift(dest, _, _)
                    | Instruction::RightShift(dest, _, _)
                    | Instruction::BitwiseAnd(dest, _, _)
                    | Instruction::BitwiseOr(dest, _, _)
                    | Instruction::BitwiseXor(dest, _, _)
                    | Instruction::LogicalAnd(dest, _, _)
                    | Instruction::LogicalOr(dest, _, _)
                    | Instruction::LessThan(dest, _, _)
                    | Instruction::GreaterThan(dest, _, _)
                    | Instruction::LessThanEq(dest, _, _)
                    | Instruction::GreaterThanEq(dest, _, _)
                    | Instruction::Equal(dest, _, _)
                    | Instruction::NotEqual(dest, _, _)
                    | Instruction::Call(dest, _, _)
                    | Instruction::PointerToStringLiteral(dest, _)
                    | Instruction::I8toI16(dest, _)
                    | Instruction::I8toU16(dest, _)
                    | Instruction::U8toI16(dest, _)
                    | Instruction::U8toU16(dest, _)
                    | Instruction::I16toI32(dest, _)
                    | Instruction::U16toI32(dest, _)
                    | Instruction::I16toU32(dest, _)
                    | Instruction::U16toU32(dest, _)
                    | Instruction::I32toU32(dest, _)
                    | Instruction::I32toU64(dest, _)
                    | Instruction::U32toU64(dest, _)
                    | Instruction::I64toU64(dest, _)
                    | Instruction::I32toI64(dest, _)
                    | Instruction::U32toI64(dest, _)
                    | Instruction::U32toF32(dest, _)
                    | Instruction::I32toF32(dest, _)
                    | Instruction::U64toF32(dest, _)
                    | Instruction::I64toF32(dest, _)
                    | Instruction::U32toF64(dest, _)
                    | Instruction::I32toF64(dest, _)
                    | Instruction::U64toF64(dest, _)
                    | Instruction::I64toF64(dest, _)
                    | Instruction::F32toF64(dest, _)
                    | Instruction::I32toI8(dest, _)
                    | Instruction::U32toI8(dest, _)
                    | Instruction::I64toI8(dest, _)
                    | Instruction::U64toI8(dest, _)
                    | Instruction::I32toU8(dest, _)
                    | Instruction::U32toU8(dest, _)
                    | Instruction::I64toU8(dest, _)
                    | Instruction::U64toU8(dest, _)
                    | Instruction::I64toI32(dest, _)
                    | Instruction::U64toI32(dest, _)
                    | Instruction::U32toPtr(dest, _)
                    | Instruction::I32toPtr(dest, _) => {
                        let dest_type = prog_metadata.get_var_type(dest).unwrap();
                        vars.push((dest.to_owned(), dest_type));
                    }
                    Instruction::Ret(_)
                    | Instruction::Label(_)
                    | Instruction::Br(_)
                    | Instruction::BrIfEq(_, _, _)
                    | Instruction::BrIfNotEq(_, _, _)
                    | Instruction::Nop
                    | Instruction::Break(_)
                    | Instruction::Continue(_)
                    | Instruction::EndHandledBlock(_)
                    | Instruction::IfEqElse(_, _, _, _)
                    | Instruction::IfNotEqElse(_, _, _, _) => {}
                }
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_with_types(&next, prog_metadata));
                    vars
                }
            }
        }
        Block::Loop { id, inner, next } => {
            let mut inner_block_vars = get_vars_with_types(&inner, prog_metadata);
            match next {
                None => inner_block_vars,
                Some(next) => {
                    inner_block_vars.extend(get_vars_with_types(&next, prog_metadata));
                    inner_block_vars
                }
            }
        }
        Block::Multiple {
            id,
            handled_blocks,
            next,
        } => {
            let mut vars = Vec::new();

            for handled in handled_blocks {
                vars.extend(get_vars_with_types(&handled, prog_metadata));
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_with_types(&next, prog_metadata));
                    vars
                }
            }
        }
    }
}

fn calculate_var_offsets_from_fp(
    block: &Box<Block>,
    fun_type: Box<IrType>,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, u32> {
    let block_vars = get_vars_with_types(block, prog_metadata);

    let mut var_offsets = HashMap::new();
    let mut offset = calculate_start_of_vars_offset_from_fp(fun_type, prog_metadata);

    for (var_id, var_type) in block_vars {
        let byte_size = match var_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(size) => size,
            TypeSize::Runtime(_) => {
                unreachable!()
            }
        };
        println!("  var {} ({}): offset {}", var_id, var_type, offset);
        var_offsets.insert(var_id, offset);
        offset += byte_size as u32;
    }

    var_offsets
}

fn calculate_start_of_vars_offset_from_fp(
    fun_type: Box<IrType>,
    prog_metadata: &Box<ProgramMetadata>,
) -> u32 {
    let mut offset = PTR_SIZE; // account for prev frame pointer

    let (return_type, param_types) = match *fun_type {
        IrType::Function(return_type, param_types, _is_variadic) => (return_type, param_types),
        _ => unreachable!(),
    };
    let return_type_byte_size = match return_type.get_byte_size(prog_metadata) {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => {
            unreachable!()
        }
    };
    offset += return_type_byte_size as u32;

    for param_type in param_types {
        let param_byte_size = match param_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(size) => size,
            TypeSize::Runtime(_) => {
                unreachable!()
            }
        };
        offset += param_byte_size as u32;
    }

    offset
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

struct FunctionContext {
    var_fp_offsets: HashMap<VarId, u32>,
    return_value_fp_offset: u32,
    top_of_stack_fp_offset: u32,
    label_variable: VarId,
}

impl FunctionContext {
    fn new(var_fp_offsets: HashMap<VarId, u32>, label_variable: VarId) -> Self {
        FunctionContext {
            var_fp_offsets,
            return_value_fp_offset: PTR_SIZE,
            top_of_stack_fp_offset: 0,
            label_variable,
        }
    }
}

fn convert_block_to_wasm(
    block: Box<Block>,
    function_context: &mut FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> Vec<WasmInstruction> {
    let mut wasm_instrs = Vec::new();

    match *block {
        Block::Simple { internal, next } => {
            for instr in internal.instrs {
                wasm_instrs.append(&mut convert_ir_instr_to_wasm(
                    instr,
                    function_context,
                    module_context,
                    prog_metadata,
                ));
            }

            match next {
                None => {}
                Some(next) => {
                    wasm_instrs.append(&mut convert_block_to_wasm(
                        next,
                        function_context,
                        module_context,
                        prog_metadata,
                    ));
                }
            }
        }
        Block::Loop { id, inner, next } => {
            let loop_instrs =
                convert_block_to_wasm(inner, function_context, module_context, prog_metadata);
            wasm_instrs.push(WasmInstruction::Loop {
                blocktype: BlockType::None,
                instrs: loop_instrs,
            });

            match next {
                None => {}
                Some(next) => {
                    wasm_instrs.append(&mut convert_block_to_wasm(
                        next,
                        function_context,
                        module_context,
                        prog_metadata,
                    ));
                }
            }
        }
        Block::Multiple {
            id,
            handled_blocks,
            next,
        } => {
            // select which of the handled blocks to execute (if any)
            //
            // load the label variable
            load_var(
                function_context.label_variable.to_owned(),
                &mut wasm_instrs,
                function_context,
                prog_metadata,
            );

            for handled_block in handled_blocks {
                let handled_block_label = match *handled_block {
                    Block::Simple { internal, next } => {}
                    Block::Loop { id, inner, next } => {}
                    Block::Multiple {
                        id,
                        handled_blocks,
                        next,
                    } => {}
                };
                // todo get possible entry labels for handled block

                // todo check if label variable matches handled block label
            }

            // todo select between handled blocks

            match next {
                None => {}
                Some(next) => {
                    wasm_instrs.append(&mut convert_block_to_wasm(
                        next,
                        function_context,
                        module_context,
                        prog_metadata,
                    ));
                }
            }
        }
    }

    wasm_instrs
}

fn convert_ir_instr_to_wasm(
    instr: Instruction,
    function_context: &mut FunctionContext,
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
                function_context,
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

            pop_stack_frame(
                dest,
                callee_function_type,
                &mut wasm_instrs,
                function_context,
                prog_metadata,
            );
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

fn increment_frame_ptr(offset: u32, wasm_instrs: &mut Vec<WasmInstruction>) {
    // address operand
    wasm_instrs.push(WasmInstruction::I32Const {
        n: FRAME_PTR_ADDR as i32,
    });
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

fn restore_previous_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
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

fn load_stack_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
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

fn increment_stack_ptr(offset: u32, wasm_instrs: &mut Vec<WasmInstruction>) {
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

fn set_stack_ptr_to_frame_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
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

/// Insert instructions to load the given variable onto the wasm stack
fn load_var(
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

/// Insert a load instruction of the correct type
fn load(value_type: Box<IrType>, wasm_instrs: &mut Vec<WasmInstruction>) {
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

fn store_var(
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
    let mut new_stack_frame_fp_offset = function_context.top_of_stack_fp_offset;
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
        mem_arg: MemArg::zero(),
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
                load_var(var_id, wasm_instrs, function_context, prog_metadata);

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
    increment_frame_ptr(frame_ptr_increment_value, wasm_instrs);
}

fn pop_stack_frame(
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
