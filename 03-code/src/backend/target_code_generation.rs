use crate::backend::target_code_generation_context::{
    ControlFlowElement, FunctionContext, ModuleContext,
};
use crate::backend::wasm_indices::{FuncIdx, LocalIdx};
use crate::backend::wasm_instructions::{BlockType, MemArg, WasmExpression, WasmInstruction};
use crate::backend::wasm_program::{WasmFunction, WasmProgram};
use crate::middle_end::ids::{FunId, Id, LabelId, VarId};
use crate::middle_end::instructions::{Constant, Dest, Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::relooper::blocks::{Block, LoopBlockId, MultipleBlockId};
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};
use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};

const PTR_SIZE: u32 = 4;

const FRAME_PTR_ADDR: u32 = 0;
const STACK_PTR_ADDR: u32 = FRAME_PTR_ADDR + PTR_SIZE;

pub fn generate_target_code(prog: ReloopedProgram) -> WasmProgram {
    // let mut wasm_program = WasmProgram::new();

    let (imported_functions, defined_functions) = separate_imported_and_defined_functions(
        &prog.program_metadata,
        prog.program_blocks.functions,
    );

    let mut module_context = ModuleContext::new();

    module_context.calculate_func_idxs(&imported_functions, &defined_functions);

    for (fun_id, function) in defined_functions {
        if let Some(block) = function.block {
            let mut function_wasm_instrs = Vec::new();

            let var_offsets = allocate_local_vars(
                &block,
                &mut function_wasm_instrs,
                function.type_info,
                function.param_var_mappings,
                &prog.program_metadata,
            );

            let mut function_context =
                FunctionContext::new(var_offsets, function.label_variable.unwrap());

            function_wasm_instrs.append(&mut convert_block_to_wasm(
                block,
                &mut function_context,
                &module_context,
                &prog.program_metadata,
            ))
        }
    }

    todo!()
}

fn separate_imported_and_defined_functions(
    prog_metadata: &Box<ProgramMetadata>,
    functions: HashMap<FunId, ReloopedFunction>,
) -> (
    Vec<(FunId, String, ReloopedFunction)>,
    Vec<(FunId, ReloopedFunction)>,
) {
    let imported_function_names = vec!["printf".to_owned()];

    let mut imported_functions: Vec<(FunId, String, ReloopedFunction)> = Vec::new();
    let mut defined_functions: Vec<(FunId, ReloopedFunction)> = Vec::new();

    let mut imported_function_ids: HashMap<FunId, String> = HashMap::new();
    for imported_function_name in imported_function_names {
        match prog_metadata.function_ids.get(&imported_function_name) {
            None => {
                // this function that could be imported isn't used in this program
            }
            Some(fun_id) => {
                imported_function_ids.insert(fun_id.to_owned(), imported_function_name);
            }
        }
    }

    for (fun_id, function) in functions {
        // check if this function is an imported one
        match imported_function_ids.get(&fun_id) {
            Some(fun_name) => {
                imported_functions.push((fun_id, fun_name.to_owned(), function));
            }
            None => {
                defined_functions.push((fun_id, function));
            }
        }
    }

    (imported_functions, defined_functions)
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

fn allocate_local_vars(
    block: &Box<Block>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_type: Box<IrType>,
    fun_param_var_mappings: Vec<VarId>,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, u32> {
    let block_vars = get_vars_with_types(block, prog_metadata);

    let mut var_offsets = HashMap::new();
    let mut offset = PTR_SIZE;

    let (return_type, param_types) = match *fun_type {
        IrType::Function(return_type, param_types, _is_variadic) => (return_type, param_types),
        _ => unreachable!(),
    };

    // increment offset by return value size
    let return_type_byte_size = match return_type.get_byte_size(prog_metadata) {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => {
            unreachable!()
        }
    };
    offset += return_type_byte_size as u32;

    // calculate offset of each param variable
    for param_i in 0..param_types.len() {
        let param_type = param_types.get(param_i).unwrap();
        let param_var_id = fun_param_var_mappings.get(param_i).unwrap();
        let param_byte_size = match param_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(size) => size,
            TypeSize::Runtime(_) => {
                unreachable!()
            }
        };
        println!(
            "  param {} ({}): offset {}",
            param_var_id, param_type, offset
        );
        var_offsets.insert(param_var_id.to_owned(), offset);
        offset += param_byte_size as u32;
    }

    // calculate offset of each local variable
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

    // update stack pointer to after allocated vars
    increment_stack_ptr(offset, wasm_instrs);

    var_offsets
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
            function_context
                .control_flow_stack
                .push(ControlFlowElement::Block(id.to_owned()));
            function_context
                .control_flow_stack
                .push(ControlFlowElement::Loop(id));
            let loop_instrs =
                convert_block_to_wasm(inner, function_context, module_context, prog_metadata);
            wasm_instrs.push(WasmInstruction::Block {
                blocktype: BlockType::None,
                instrs: vec![WasmInstruction::Loop {
                    blocktype: BlockType::None,
                    instrs: loop_instrs,
                }],
            });
            // pop the block and loop control flows we pushed before
            function_context.control_flow_stack.pop();
            function_context.control_flow_stack.pop();

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
            mut handled_blocks,
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

            wasm_instrs.append(&mut convert_handled_blocks(
                handled_blocks.into(),
                id,
                function_context,
                module_context,
                prog_metadata,
            ));

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
        Instruction::SimpleAssignment(_, _) => {}
        Instruction::LoadFromAddress(_, _) => {}
        Instruction::StoreToAddress(_, _) => {}
        Instruction::AllocateVariable(_, _) => {}
        Instruction::AddressOf(_, _) => {}
        Instruction::BitwiseNot(_, _) => {}
        Instruction::LogicalNot(_, _) => {}
        Instruction::Mult(_, _, _) => {}
        Instruction::Div(_, _, _) => {}
        Instruction::Mod(_, _, _) => {}
        Instruction::Add(_, _, _) => {}
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
                    .fun_id_to_func_idx_map
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

fn set_frame_ptr_to_stack_ptr(wasm_instrs: &mut Vec<WasmInstruction>) {
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
    increment_stack_ptr(PTR_SIZE, wasm_instrs);

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
    increment_stack_ptr(return_type_byte_size as u32, wasm_instrs);

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
                store(var_type, wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr(var_byte_size as u32, wasm_instrs);
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
                store(param_type.to_owned(), wasm_instrs);

                // advance the stack pointer
                increment_stack_ptr(param_byte_size as u32, wasm_instrs);
            }
            Src::StoreAddressVar(_) | Src::Fun(_) => {
                unreachable!()
            }
        }
        param_index += 1;
    }
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

/// Inserts instructions to compare the label variable against some number of label values,
/// leaving a single boolean value on the stack
fn test_label_equality(
    labels: Vec<LabelId>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &FunctionContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    assert!(!labels.is_empty());

    if labels.len() == 1 {
        // load value of label variable
        load_var(
            function_context.label_variable.to_owned(),
            wasm_instrs,
            function_context,
            prog_metadata,
        );
        // label value to compare against
        wasm_instrs.push(WasmInstruction::I64Const {
            n: labels.first().unwrap().as_u64() as i64,
        });
        // equality comparison
        wasm_instrs.push(WasmInstruction::I64Eq);
    } else {
        // there's more than one label to compare against, so do multiple equality tests and OR them together
        for label in &labels {
            // load value of label variable
            load_var(
                function_context.label_variable.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );
            // label value to compare against
            wasm_instrs.push(WasmInstruction::I64Const {
                n: label.as_u64() as i64,
            });
            // equality comparison
            wasm_instrs.push(WasmInstruction::I64Eq);
        }
        // OR all the results together. With n labels to test against, we need n-1 OR instructions.
        for _ in 0..labels.len() - 1 {
            wasm_instrs.push(WasmInstruction::I32Or);
        }
    }
}

fn convert_handled_blocks(
    mut handled_blocks: VecDeque<Box<Block>>,
    multiple_block_id: MultipleBlockId,
    function_context: &mut FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> Vec<WasmInstruction> {
    if handled_blocks.is_empty() {
        return Vec::new();
    }

    let mut instrs = Vec::new();

    let first_handled_block = handled_blocks.pop_front().unwrap();

    // get possible entry labels for handled block
    let handled_block_entries = first_handled_block.get_entry_labels();

    // check if label variable matches any of handled block entry labels
    test_label_equality(
        handled_block_entries,
        &mut instrs,
        function_context,
        prog_metadata,
    );

    function_context
        .control_flow_stack
        .push(ControlFlowElement::If(multiple_block_id.to_owned()));

    // if so, execute the handled block
    instrs.push(WasmInstruction::IfElse {
        blocktype: BlockType::None,
        if_instrs: convert_block_to_wasm(
            first_handled_block,
            function_context,
            module_context,
            prog_metadata,
        ),
        else_instrs: convert_handled_blocks(
            handled_blocks,
            multiple_block_id,
            function_context,
            module_context,
            prog_metadata,
        ),
    });

    // pop the if control flow we pushed before
    function_context.control_flow_stack.pop();

    instrs
}
