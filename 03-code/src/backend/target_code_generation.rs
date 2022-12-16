use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};

use crate::backend::allocate_local_vars::allocate_local_vars;
use crate::backend::memory_operations::load_var;
use crate::backend::stack_frame_operations::{pop_stack_frame, set_up_new_stack_frame};
use crate::backend::target_code_generation_context::{
    ControlFlowElement, FunctionContext, ModuleContext,
};
use crate::backend::wasm_instructions::{BlockType, WasmInstruction};
use crate::backend::wasm_program::WasmProgram;
use crate::middle_end::ids::{FunId, Id, LabelId};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
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
