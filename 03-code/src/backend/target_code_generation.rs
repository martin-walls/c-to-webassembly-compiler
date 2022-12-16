use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};

use crate::backend::allocate_local_vars::allocate_local_vars;
use crate::backend::memory_operations::{load, load_src, load_var, store, store_var};
use crate::backend::stack_frame_operations::{
    increment_stack_ptr_by_known_offset, increment_stack_ptr_dynamic, load_frame_ptr,
    load_stack_ptr, pop_stack_frame, set_up_new_stack_frame,
};
use crate::backend::target_code_generation_context::{
    ControlFlowElement, FunctionContext, ModuleContext,
};
use crate::backend::wasm_instructions::{BlockType, MemArg, WasmInstruction};
use crate::backend::wasm_program::WasmProgram;
use crate::middle_end::ids::{FunId, Id, LabelId};
use crate::middle_end::instructions::{Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::{Block, LoopBlockId, MultipleBlockId};
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};

pub const PTR_SIZE: u32 = 4;

pub const FRAME_PTR_ADDR: u32 = 0;
pub const STACK_PTR_ADDR: u32 = FRAME_PTR_ADDR + PTR_SIZE;

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

            // todo attach function to module
        }
    }

    todo!("finish implementing target code generation")
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
                convert_ir_instr_to_wasm(
                    instr,
                    &mut wasm_instrs,
                    function_context,
                    module_context,
                    prog_metadata,
                );
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
    wasm_instrs: &mut Vec<WasmInstruction>,
    function_context: &mut FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    match instr {
        Instruction::SimpleAssignment(dest, src) => {
            // load src onto wasm stack
            let mut load_src_instrs = Vec::new();
            load_src(src, &mut load_src_instrs, function_context, prog_metadata);

            // store to dest
            store_var(
                dest,
                load_src_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::LoadFromAddress(dest, src) => {
            // load the ptr address onto wasm stack
            let mut load_instrs = Vec::new();
            load_src(src, &mut load_instrs, function_context, prog_metadata);

            // load the value at that pointer, which should have the same type as dest
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load(dest_type, &mut load_instrs);

            // store to dest
            store_var(
                dest,
                load_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::StoreToAddress(dest, src) => {
            // store the value of src to the location pointed to by dest
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            let inner_dest_type = dest_type.dereference_pointer_type().unwrap();

            // load the value of dest - the address operand
            load_var(dest, wasm_instrs, function_context, prog_metadata);

            // load the value to store
            load_src(src, wasm_instrs, function_context, prog_metadata);

            store(inner_dest_type, wasm_instrs);
        }
        Instruction::AllocateVariable(dest, byte_size) => {
            // allocate byte_size many bytes on the stack, and set dest to be a pointer to there
            //
            // store the current stack pointer to dest
            let mut load_stack_ptr_instrs = Vec::new();
            load_stack_ptr(&mut load_stack_ptr_instrs);
            store_var(
                dest,
                load_stack_ptr_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );

            let mut load_byte_size_instrs = Vec::new();

            // load the number of bytes to allocate
            load_src(
                byte_size,
                &mut load_byte_size_instrs,
                function_context,
                prog_metadata,
            );

            // increment stack pointer
            increment_stack_ptr_dynamic(load_byte_size_instrs, wasm_instrs);
        }
        Instruction::AddressOf(dest, src) => {
            // store the address of src in dest
            // src can only be a Var, not a Constant
            let src_var = match src {
                Src::Var(var_id) => var_id,
                Src::Constant(_) | Src::StoreAddressVar(_) | Src::Fun(_) => {
                    unreachable!()
                }
            };

            match function_context.var_fp_offsets.get(&src_var) {
                None => {
                    // todo check if src_var is a global variable, and get its address
                }
                Some(fp_offset) => {
                    // load the frame pointer, add the offset to it, and store the result in dest
                    let mut load_instrs = Vec::new();
                    load_frame_ptr(&mut load_instrs);
                    wasm_instrs.push(WasmInstruction::I32Const {
                        n: *fp_offset as i32,
                    });
                    wasm_instrs.push(WasmInstruction::I32Add);
                    store_var(
                        dest,
                        load_instrs,
                        wasm_instrs,
                        function_context,
                        prog_metadata,
                    );
                }
            }
        }
        Instruction::BitwiseNot(dest, src) => {
            // bitwise not is implemented as XORing with -1
            //
            // load src
            let src_type = src.get_type(prog_metadata).unwrap();

            let mut temp_instrs = Vec::new();
            load_src(src, &mut temp_instrs, function_context, prog_metadata);

            // bitwise not
            match *src_type {
                IrType::I32 | IrType::U32 => {
                    temp_instrs.push(WasmInstruction::I32Const { n: -1 });
                    temp_instrs.push(WasmInstruction::I32Xor);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Const { n: -1 });
                    temp_instrs.push(WasmInstruction::I64Xor);
                }
                _ => {
                    unreachable!()
                }
            }

            // store to dest
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::LogicalNot(dest, src) => {
            // logical not is implemented as a test-for-zero and set
            //
            // load src
            let src_type = src.get_type(prog_metadata).unwrap();

            let mut temp_instrs = Vec::new();
            load_src(src, &mut temp_instrs, function_context, prog_metadata);

            // logical not
            // test if src is zero
            match *src_type {
                IrType::I32 | IrType::U32 | IrType::PointerTo(_) => {
                    temp_instrs.push(WasmInstruction::I32Eqz);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Eqz);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Const { z: 0. });
                    temp_instrs.push(WasmInstruction::F32Eq);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Const { z: 0. });
                    temp_instrs.push(WasmInstruction::F64Eq);
                }
                _ => {
                    unreachable!()
                }
            }
            // if so, result is 1, else 0
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs: vec![WasmInstruction::I32Const { n: 1 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 0 }],
            });

            // store to dest
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
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
                wasm_instrs,
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
                wasm_instrs,
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
