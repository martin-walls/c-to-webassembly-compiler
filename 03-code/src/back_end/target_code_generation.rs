use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};

use log::{debug, info};

use crate::back_end::backend_error::BackendError;
use crate::back_end::import_export_names::get_imported_function_names;
use crate::back_end::import_export_names::MAIN_FUNCTION_EXPORT_NAME;
use crate::back_end::initialise_memory::initialise_memory;
use crate::back_end::memory_constants::PTR_SIZE;
use crate::back_end::memory_operations::{
    load, load_constant, load_src, load_var, load_var_address, store, store_var,
};
use crate::back_end::profiler::initialise_profiler;
use crate::back_end::stack_allocation::allocate_vars::{allocate_global_vars, allocate_local_vars};
use crate::back_end::stack_frame_operations::{
    increment_stack_ptr_by_known_offset, increment_stack_ptr_dynamic, load_frame_ptr,
    load_stack_ptr, overwrite_current_stack_frame_with_new_stack_frame, pop_stack_frame,
    set_frame_ptr_to_stack_ptr, set_up_new_stack_frame,
};
use crate::back_end::target_code_generation_context::{
    ControlFlowElement, FunctionContext, ModuleContext,
};
use crate::back_end::wasm_indices::{FuncIdx, LabelIdx, LocalIdx, TypeIdx};
use crate::back_end::wasm_instructions::{BlockType, MemArg, WasmExpression, WasmInstruction};
use crate::back_end::wasm_module::exports_section::{ExportDescriptor, WasmExport};
use crate::back_end::wasm_module::module::WasmModule;
use crate::back_end::wasm_module::types_section::WasmFunctionType;
use crate::back_end::wasm_types::{NumType, ValType};
use crate::id::Id;
use crate::middle_end::ids::{FunId, LabelId};
use crate::middle_end::instructions::{Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::program_config::enabled_optimisations::EnabledOptimisations;
use crate::program_config::enabled_profiling::EnabledProfiling;
use crate::relooper::blocks::{Block, MultipleBlockId};
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};

pub const MAIN_FUNCTION_SOURCE_NAME: &str = "main";

pub fn generate_target_code(
    mut prog: ReloopedProgram,
    enabled_optimisations: &EnabledOptimisations,
    enabled_profiling: &EnabledProfiling,
) -> Result<WasmModule, BackendError> {
    let mut wasm_module = WasmModule::new();
    let mut module_context = ModuleContext::new(enabled_profiling);

    initialise_profiler(&mut module_context, &mut prog);

    let (imported_functions, defined_functions) = separate_imported_and_defined_functions(
        &prog.program_metadata,
        prog.program_blocks.functions,
    );

    module_context.calculate_func_idxs(&imported_functions, &defined_functions);

    let initial_top_of_stack_addr = initialise_memory(
        &mut wasm_module,
        &mut module_context,
        &prog.program_metadata,
    );

    let mut func_idx_to_type_idx_map: HashMap<FuncIdx, TypeIdx> = HashMap::new();
    let mut func_idx_to_body_code_map: HashMap<FuncIdx, WasmExpression> = HashMap::new();

    //
    // global instrs
    //

    // create a wasm function for the global instructions
    let mut global_wasm_instrs = Vec::new();
    let global_instrs_func_idx = module_context.new_defined_func_idx();

    // (i32, i32) -> i32
    let global_wasm_function_type = WasmFunctionType {
        param_types: vec![
            ValType::NumType(NumType::I32),
            ValType::NumType(NumType::I32),
        ],
        result_types: vec![ValType::NumType(NumType::I32)],
    };
    let global_wasm_function_type_idx = wasm_module.insert_type(global_wasm_function_type);

    func_idx_to_type_idx_map.insert(
        global_instrs_func_idx.to_owned(),
        global_wasm_function_type_idx,
    );

    // initialise the frame pointer, and set previous frame ptr value to NULL
    // address operand
    load_stack_ptr(&mut global_wasm_instrs);
    // value to store
    global_wasm_instrs.push(WasmInstruction::I32Const { n: 0 });
    global_wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg::zero(),
    });
    // set frame ptr to start of this frame
    set_frame_ptr_to_stack_ptr(&mut global_wasm_instrs);

    let mut global_var_addrs = HashMap::new();
    if let Some(global_block) = prog.program_blocks.global_instrs {
        global_var_addrs = allocate_global_vars(
            &global_block,
            initial_top_of_stack_addr,
            &mut global_wasm_instrs,
            &module_context,
            &prog.program_metadata,
        );

        // let var_offsets = allocate_local_vars(
        //     &global_block,
        //     &mut global_wasm_instrs,
        //     Box::new(IrType::Function(Box::new(IrType::Void), Vec::new(), false)), // global instructions have a void function type, so don't allocate any space for a return value
        //     Vec::new(),                                                            // no parameters
        //     &prog.program_metadata,
        // );

        let mut global_context = FunctionContext::global_context(global_var_addrs.to_owned());

        global_wasm_instrs.append(&mut convert_block_to_wasm(
            global_block,
            &mut global_context,
            &module_context,
            &prog.program_metadata,
        ));
    }

    // call main() after global instructions -- set up its stack frame
    //
    // store frame ptr at start of stack frame
    load_stack_ptr(&mut global_wasm_instrs);
    // value to store: current value of frame ptr
    load_frame_ptr(&mut global_wasm_instrs);
    // store frame ptr
    global_wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg::zero(),
    });

    // set frame ptr to point at new stack frame
    set_frame_ptr_to_stack_ptr(&mut global_wasm_instrs);

    // increment stack ptr, also leaving space for i32 return value
    let i32_byte_size = IrType::I32
        .get_byte_size(&prog.program_metadata)
        .get_compile_time_value()
        .unwrap();
    increment_stack_ptr_by_known_offset(
        PTR_SIZE + i32_byte_size as u32,
        &mut global_wasm_instrs,
        &module_context,
    );

    // store params argc and argv in main()'s stack frame
    //
    // address operand for where to store argc param
    load_stack_ptr(&mut global_wasm_instrs);
    // load argc
    global_wasm_instrs.push(WasmInstruction::LocalGet {
        local_idx: LocalIdx { x: 0 },
    });
    // store
    global_wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg::zero(),
    });
    // increment stack ptr
    increment_stack_ptr_by_known_offset(
        i32_byte_size as u32,
        &mut global_wasm_instrs,
        &module_context,
    );

    // address operand for where to store argv param
    load_stack_ptr(&mut global_wasm_instrs);
    // load argv
    global_wasm_instrs.push(WasmInstruction::LocalGet {
        local_idx: LocalIdx { x: 1 },
    });
    // store
    global_wasm_instrs.push(WasmInstruction::I32Store {
        mem_arg: MemArg::zero(),
    });
    // increment stack ptr
    increment_stack_ptr_by_known_offset(
        i32_byte_size as u32,
        &mut global_wasm_instrs,
        &module_context,
    );

    // call main()
    match prog
        .program_metadata
        .function_ids
        .get(MAIN_FUNCTION_SOURCE_NAME)
    {
        None => {
            // error: main function must be defined
            return Err(BackendError::NoMainFunctionDefined);
        }
        Some(main_fun_id) => {
            let main_func_idx = module_context
                .fun_id_to_func_idx_map
                .get(main_fun_id)
                .unwrap();
            global_wasm_instrs.push(WasmInstruction::Call {
                func_idx: main_func_idx.to_owned(),
            });
        }
    }

    // load and return result i32
    //
    // address operand for loading return value: frame ptr is still pointing at main() stack frame,
    //     return value is just above previous frame ptr
    load_frame_ptr(&mut global_wasm_instrs);
    global_wasm_instrs.push(WasmInstruction::I32Const { n: PTR_SIZE as i32 });
    global_wasm_instrs.push(WasmInstruction::I32Add);
    // load return value onto stack
    global_wasm_instrs.push(WasmInstruction::I32Load {
        mem_arg: MemArg::zero(),
    });
    // return from program
    global_wasm_instrs.push(WasmInstruction::Return);

    func_idx_to_body_code_map.insert(
        global_instrs_func_idx.to_owned(),
        WasmExpression {
            instrs: global_wasm_instrs,
        },
    );

    // export this function from wasm module
    let main_export = WasmExport {
        name: MAIN_FUNCTION_EXPORT_NAME.to_owned(),
        export_descriptor: ExportDescriptor::Func {
            func_idx: global_instrs_func_idx,
        },
    };
    wasm_module.exports_section.exports.push(main_export);

    // end global instrs

    // insert empty function type to module
    let empty_type = WasmFunctionType {
        param_types: Vec::new(),
        result_types: Vec::new(),
    };
    let empty_type_idx = wasm_module.insert_type(empty_type);

    for (fun_id, function) in defined_functions {
        let wasm_func_idx = module_context.fun_id_to_func_idx_map.get(&fun_id).unwrap();

        // all functions have empty type, because params/result are stored in stack frame
        func_idx_to_type_idx_map.insert(wasm_func_idx.to_owned(), empty_type_idx.to_owned());

        if let Some(mut block) = function.block {
            let mut function_wasm_instrs = Vec::new();

            let var_offsets = allocate_local_vars(
                &mut block,
                &mut function_wasm_instrs,
                function.type_info,
                function.param_var_mappings,
                &module_context,
                &mut prog.program_metadata,
                enabled_optimisations,
            );

            let mut function_context = FunctionContext::new(
                var_offsets,
                global_var_addrs.to_owned(),
                function.label_variable.unwrap(),
            );

            function_wasm_instrs.append(&mut convert_block_to_wasm(
                block,
                &mut function_context,
                &module_context,
                &prog.program_metadata,
            ));

            func_idx_to_body_code_map.insert(
                wasm_func_idx.to_owned(),
                WasmExpression {
                    instrs: function_wasm_instrs,
                },
            );
        } else {
            // empty function body
            func_idx_to_body_code_map.insert(
                wasm_func_idx.to_owned(),
                WasmExpression { instrs: Vec::new() },
            );
        }
    }

    wasm_module.insert_defined_functions(
        func_idx_to_body_code_map,
        func_idx_to_type_idx_map,
        &module_context,
    );

    let mut imported_func_idx_to_type_idx_map: HashMap<FuncIdx, TypeIdx> = HashMap::new();
    let mut imported_func_idx_to_name_map: HashMap<FuncIdx, String> = HashMap::new();

    for (fun_id, fun_name, _) in imported_functions {
        let wasm_func_idx = module_context.fun_id_to_func_idx_map.get(&fun_id).unwrap();

        // imported functions also have the empty type, cos they obey the same calling convention
        imported_func_idx_to_type_idx_map
            .insert(wasm_func_idx.to_owned(), empty_type_idx.to_owned());

        imported_func_idx_to_name_map.insert(wasm_func_idx.to_owned(), fun_name);
    }
    wasm_module.insert_imported_functions(
        imported_func_idx_to_type_idx_map,
        imported_func_idx_to_name_map,
        &module_context,
    );

    Ok(wasm_module)
}

fn separate_imported_and_defined_functions(
    prog_metadata: &ProgramMetadata,
    functions: HashMap<FunId, ReloopedFunction>,
) -> (
    Vec<(FunId, String, ReloopedFunction)>,
    Vec<(FunId, ReloopedFunction)>,
) {
    let imported_function_names = get_imported_function_names();

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
    block: Block,
    function_context: &mut FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) -> Vec<WasmInstruction> {
    let mut wasm_instrs = Vec::new();

    match block {
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
                        *next,
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
            let mut loop_instrs =
                convert_block_to_wasm(*inner, function_context, module_context, prog_metadata);
            // explicit branch back to the start of the loop
            loop_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: 0 },
            });
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
                        *next,
                        function_context,
                        module_context,
                        prog_metadata,
                    ));
                }
            }
        }
        Block::Multiple {
            id,
            pre_handled_blocks_instrs: _,
            handled_blocks,
            next,
        } => {
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
                        *next,
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
    prog_metadata: &ProgramMetadata,
) {
    match instr {
        Instruction::SimpleAssignment(_, dest, src) => {
            // load src onto wasm stack
            let mut load_src_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                src,
                dest_type,
                &mut load_src_instrs,
                function_context,
                prog_metadata,
            );

            // store to dest
            store_var(
                dest,
                load_src_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::LoadFromAddress(_, dest, src) => {
            // load the ptr address onto wasm stack
            let mut load_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                src,
                dest_type.to_owned(),
                &mut load_instrs,
                function_context,
                prog_metadata,
            );

            // load the value at that pointer, which should have the same type as dest
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
        Instruction::StoreToAddress(_, dest, src) => {
            // store the value of src to the location pointed to by dest
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            let inner_dest_type = dest_type.dereference_pointer_type().unwrap();

            // load the value of dest - the address operand
            load_var(dest, wasm_instrs, function_context, prog_metadata);

            // load the value to store
            load_src(
                src,
                inner_dest_type.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );

            store(inner_dest_type, wasm_instrs);
        }
        Instruction::DeclareVariable(..) | Instruction::ReferenceVariable(..) => {
            // no instructions to generate here
        }
        Instruction::AllocateVariable(_, dest, byte_size) => {
            // allocate byte_size many bytes on the stack, and set dest to be a pointer to there
            //
            // store the current stack pointer to dest
            load_var_address(&dest, wasm_instrs, function_context, prog_metadata);

            // let mut load_stack_ptr_instrs = Vec::new();
            load_stack_ptr(wasm_instrs);

            store(
                IrType::PointerTo(Box::new(prog_metadata.get_var_type(&dest).unwrap())),
                wasm_instrs,
            );

            // store_var(
            //     dest,
            //     load_stack_ptr_instrs,
            //     wasm_instrs,
            //     function_context,
            //     prog_metadata,
            // );

            let mut load_byte_size_instrs = Vec::new();

            // load the number of bytes to allocate
            load_src(
                byte_size,
                IrType::I32,
                &mut load_byte_size_instrs,
                function_context,
                prog_metadata,
            );

            // increment stack pointer
            increment_stack_ptr_dynamic(load_byte_size_instrs, wasm_instrs, module_context);
        }
        Instruction::AddressOf(_, dest, src) => {
            // store the address of src in dest
            // src can only be a Var, not a Constant
            let src_var = match src {
                Src::Var(var_id) => var_id,
                Src::Constant(_) | Src::StoreAddressVar(_) | Src::Fun(_) => {
                    unreachable!()
                }
            };

            let mut temp_instrs = Vec::new();
            load_var_address(&src_var, &mut temp_instrs, function_context, prog_metadata);

            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::BitwiseNot(_, dest, src) => {
            // bitwise not is implemented as XORing with -1
            //
            // load src
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();

            let mut temp_instrs = Vec::new();
            load_src(
                src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // bitwise not
            match dest_type {
                IrType::I8 | IrType::U8 | IrType::I16 | IrType::U16 | IrType::I32 | IrType::U32 => {
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
        Instruction::LogicalNot(_, dest, src) => {
            // logical not is implemented as a test-for-zero and set
            //
            // load src
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();

            let mut temp_instrs = Vec::new();
            load_src(
                src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // logical not
            // test if src is zero
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_) => {
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
                blocktype: BlockType::ValType(ValType::NumType(NumType::I32)),
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
        Instruction::Mult(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // mult
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Mul);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Mul);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Mul);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Mul);
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
        Instruction::Div(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // div
            match dest_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32DivS)
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32DivU);
                }
                IrType::I64 => temp_instrs.push(WasmInstruction::I64DivS),
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64DivU);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Div);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Div);
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
        Instruction::Mod(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // remainder
            match dest_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32RemS)
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32RemU);
                }
                IrType::I64 => temp_instrs.push(WasmInstruction::I64RemS),
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64RemU);
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
        Instruction::Add(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // add
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Add);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Add);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Add);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Add);
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
        Instruction::Sub(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // sub
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Sub);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Sub);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Sub);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Sub);
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
        Instruction::LeftShift(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // shift left
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Shl);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Shl);
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
        Instruction::RightShift(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            debug!("dest type: {}", dest_type);

            // shift right
            match dest_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32ShrS)
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32ShrU);
                }
                IrType::I64 => temp_instrs.push(WasmInstruction::I64ShrS),
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64ShrU);
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
        Instruction::BitwiseAnd(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // bitwise AND
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32And);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64And);
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
        Instruction::BitwiseOr(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // bitwise OR
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Or);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Or);
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
        Instruction::BitwiseXor(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // bitwise XOR
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Xor);
                }
                IrType::I64 | IrType::U64 => {
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
        Instruction::LogicalAnd(_, dest, left_src, right_src) => {
            // logical AND:
            //   load left src, test eq zero -> int 0/1
            //   load right src, test eq zero -> int 0/1
            //   bitwise AND
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // test it left_src is zero
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
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
            // if so, push 0, else 1
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::ValType(ValType::NumType(NumType::I32)),
                if_instrs: vec![WasmInstruction::I32Const { n: 0 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 1 }],
            });

            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // test it right_src is zero
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
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
            // if so, push 0, else 1
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::ValType(ValType::NumType(NumType::I32)),
                if_instrs: vec![WasmInstruction::I32Const { n: 0 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 1 }],
            });

            // bitwise AND
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32And);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64And);
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
        Instruction::LogicalOr(_, dest, left_src, right_src) => {
            // logical OR:
            //   load left src, test eq zero -> int 0/1
            //   load right src, test eq zero -> int 0/1
            //   bitwise OR
            let mut temp_instrs = Vec::new();
            // load srcs onto wasm stack
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();

            load_src(
                left_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // test it left_src is zero
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
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
            // if so, push 0, else 1
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::ValType(ValType::NumType(NumType::I32)),
                if_instrs: vec![WasmInstruction::I32Const { n: 0 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 1 }],
            });

            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // test it right_src is zero
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
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
            // if so, push 0, else 1
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::ValType(ValType::NumType(NumType::I32)),
                if_instrs: vec![WasmInstruction::I32Const { n: 0 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 1 }],
            });

            // bitwise OR
            match dest_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Or);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Or);
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
        Instruction::LessThan(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let src_type = get_src_type_preferably_from_var(&left_src, &right_src, prog_metadata);
            // load srcs onto wasm stack
            load_src(
                left_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // less than
            match src_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32LtS);
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32LtU);
                }
                IrType::I64 => {
                    temp_instrs.push(WasmInstruction::I64LtS);
                }
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64LtU);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Lt);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Lt);
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
        Instruction::GreaterThan(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let src_type = get_src_type_preferably_from_var(&left_src, &right_src, prog_metadata);
            // load srcs onto wasm stack
            load_src(
                left_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // greater than
            match src_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32GtS);
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32GtU);
                }
                IrType::I64 => {
                    temp_instrs.push(WasmInstruction::I64GtS);
                }
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64GtU);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Gt);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Gt);
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
        Instruction::LessThanEq(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let src_type = get_src_type_preferably_from_var(&left_src, &right_src, prog_metadata);
            // load srcs onto wasm stack
            load_src(
                left_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // less than or equal
            match src_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32LeS);
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32LeU);
                }
                IrType::I64 => {
                    temp_instrs.push(WasmInstruction::I64LeS);
                }
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64LeU);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Le);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Le);
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
        Instruction::GreaterThanEq(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let src_type = get_src_type_preferably_from_var(&left_src, &right_src, prog_metadata);
            // load srcs onto wasm stack
            load_src(
                left_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // greater than or equal
            match src_type {
                IrType::I8 | IrType::I16 | IrType::I32 => {
                    temp_instrs.push(WasmInstruction::I32GeS);
                }
                IrType::U8
                | IrType::U16
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32GeU);
                }
                IrType::I64 => {
                    temp_instrs.push(WasmInstruction::I64GeS);
                }
                IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64GeU);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Ge);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Ge);
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
        Instruction::Equal(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let src_type = get_src_type_preferably_from_var(&left_src, &right_src, prog_metadata);

            // load srcs onto wasm stack
            load_src(
                left_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // equal
            match src_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Eq);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Eq);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Eq);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Eq);
                }
                t => {
                    println!("{t:?}");
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
        Instruction::NotEqual(_, dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let src_type = get_src_type_preferably_from_var(&left_src, &right_src, prog_metadata);
            // load srcs onto wasm stack
            load_src(
                left_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );

            // not equal
            match src_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    temp_instrs.push(WasmInstruction::I32Ne);
                }
                IrType::I64 | IrType::U64 => {
                    temp_instrs.push(WasmInstruction::I64Ne);
                }
                IrType::F32 => {
                    temp_instrs.push(WasmInstruction::F32Ne);
                }
                IrType::F64 => {
                    temp_instrs.push(WasmInstruction::F64Ne);
                }
                t => {
                    println!("{t:?}");
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
        Instruction::Call(_, dest, fun_id, params) => {
            let callee_function_type = prog_metadata.function_types.get(&fun_id).unwrap();

            set_up_new_stack_frame(
                callee_function_type,
                params,
                wasm_instrs,
                function_context,
                module_context,
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
                module_context,
                prog_metadata,
            );
        }
        Instruction::TailCall(_, fun_id, params) => {
            let callee_function_type = prog_metadata.function_types.get(&fun_id).unwrap();

            overwrite_current_stack_frame_with_new_stack_frame(
                callee_function_type,
                params,
                wasm_instrs,
                function_context,
                module_context,
                prog_metadata,
            );

            wasm_instrs.push(WasmInstruction::Call {
                func_idx: module_context
                    .fun_id_to_func_idx_map
                    .get(&fun_id)
                    .unwrap()
                    .to_owned(),
            });

            wasm_instrs.push(WasmInstruction::Return);
        }
        Instruction::Ret(_, return_value_src) => {
            if let Some(return_value_src) = return_value_src {
                // store return value into stack frame
                //
                // calculate address of return value in stack frame
                load_frame_ptr(wasm_instrs);
                wasm_instrs.push(WasmInstruction::I32Const { n: PTR_SIZE as i32 });
                wasm_instrs.push(WasmInstruction::I32Add);

                let return_type = return_value_src.get_type(prog_metadata).unwrap();

                // load return value to store in stack frame
                load_src(
                    return_value_src,
                    return_type.to_owned(),
                    wasm_instrs,
                    function_context,
                    prog_metadata,
                );

                store(return_type, wasm_instrs);
            }

            wasm_instrs.push(WasmInstruction::Return);
        }
        Instruction::Label(..) => {
            // labels don't do anything anymore at this stage, so just ignore them
        }
        Instruction::Br(..) | Instruction::BrIfEq(..) | Instruction::BrIfNotEq(..) => {
            unreachable!("Br instructions have all been replaced by this point")
        }
        Instruction::PointerToStringLiteral(_, dest, str_literal_id) => {
            let ptr_value = module_context
                .string_literal_id_to_ptr_map
                .get(&str_literal_id)
                .unwrap();
            info!("string literal ptr: {}", ptr_value);
            let temp_instrs = vec![WasmInstruction::I32Const {
                n: ptr_value.to_owned() as i32,
            }];
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I8toI16(_, dest, src)
        | Instruction::I8toU16(_, dest, src)
        | Instruction::U8toI16(_, dest, src)
        | Instruction::U8toU16(_, dest, src)
        | Instruction::I16toI32(_, dest, src)
        | Instruction::U16toI32(_, dest, src)
        | Instruction::I16toU32(_, dest, src)
        | Instruction::U16toU32(_, dest, src)
        | Instruction::I32toU32(_, dest, src)
        | Instruction::I32toI8(_, dest, src)
        | Instruction::U32toI8(_, dest, src)
        | Instruction::I32toU8(_, dest, src)
        | Instruction::U32toU8(_, dest, src)
        | Instruction::I64toU64(_, dest, src)
        | Instruction::U32toPtr(_, dest, src)
        | Instruction::I32toPtr(_, dest, src)
        | Instruction::PtrToI32(_, dest, src) => {
            // for all of these, dest and src get loaded to wasm stack as the same type, so this works
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            load_src(
                src,
                dest_type,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I32toU64(_, dest, src) | Instruction::I32toI64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            // load src as i64
            match src {
                Src::Var(var_id) => {
                    load_var_address(&var_id, &mut temp_instrs, function_context, prog_metadata);
                    // load i32 into an i64
                    temp_instrs.push(WasmInstruction::I64Load32S {
                        mem_arg: MemArg::zero(),
                    });
                }
                Src::Constant(constant) => {
                    load_constant(constant, IrType::I64, &mut temp_instrs);
                }
                Src::StoreAddressVar(_) | Src::Fun(_) => {
                    unreachable!()
                }
            }
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::U32toU64(_, dest, src) | Instruction::U32toI64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            // load src as i64
            match src {
                Src::Var(var_id) => {
                    load_var_address(&var_id, &mut temp_instrs, function_context, prog_metadata);
                    // load u32 into an i64
                    temp_instrs.push(WasmInstruction::I64Load32U {
                        mem_arg: MemArg::zero(),
                    });
                }
                Src::Constant(constant) => {
                    load_constant(constant, IrType::I64, &mut temp_instrs);
                }
                Src::StoreAddressVar(_) | Src::Fun(_) => {
                    unreachable!()
                }
            }
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I64toI32(_, dest, src)
        | Instruction::U64toI32(_, dest, src)
        | Instruction::I64toI8(_, dest, src)
        | Instruction::U64toI8(_, dest, src)
        | Instruction::I64toU8(_, dest, src)
        | Instruction::U64toU8(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::I64,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert i64 to i32
            temp_instrs.push(WasmInstruction::I32WrapI64);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::U32toF32(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::U32,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert u32 to f32
            temp_instrs.push(WasmInstruction::F32ConvertI32U);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I32toF32(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::I32,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert i32 to f32
            temp_instrs.push(WasmInstruction::F32ConvertI32S);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::U64toF32(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::U64,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert u64 to f32
            temp_instrs.push(WasmInstruction::F32ConvertI64U);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I64toF32(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::I64,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert i64 to f32
            temp_instrs.push(WasmInstruction::F32ConvertI64S);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::U32toF64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::U32,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert u32 to f64
            temp_instrs.push(WasmInstruction::F64ConvertI32U);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I32toF64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::I32,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert i32 to f64
            temp_instrs.push(WasmInstruction::F64ConvertI32S);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::U64toF64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::U64,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert u64 to f64
            temp_instrs.push(WasmInstruction::F64ConvertI64U);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::I64toF64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::I64,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert i64 to f64
            temp_instrs.push(WasmInstruction::F64ConvertI64S);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::F32toF64(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::F32,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert f32 to f64
            temp_instrs.push(WasmInstruction::F64PromoteF32);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::F64toI32(_, dest, src) => {
            let mut temp_instrs = Vec::new();
            load_src(
                src,
                IrType::F64,
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // convert f64 to i32
            temp_instrs.push(WasmInstruction::I32TruncF64S);
            store_var(
                dest,
                temp_instrs,
                wasm_instrs,
                function_context,
                prog_metadata,
            );
        }
        Instruction::Nop(..) => {
            // do nothing
        }
        Instruction::Break(_, loop_block_id) => {
            // get depth of block to br out of
            let depth = function_context.get_depth_of_block(&loop_block_id).unwrap();
            wasm_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: depth },
            });
        }
        Instruction::Continue(_, loop_block_id) => {
            // get depth of loop to br to start of
            let depth = function_context.get_depth_of_loop(&loop_block_id).unwrap();
            wasm_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: depth },
            });
        }
        Instruction::EndHandledBlock(_, multiple_block_id) => {
            // get depth of if to br out of
            let depth = function_context
                .get_depth_of_if(&multiple_block_id)
                .unwrap();
            wasm_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: depth },
            });
        }
        Instruction::IfEqElse(_, left_src, right_src, true_instrs, false_instrs) => {
            // load operands for comparison
            let src_type = left_src.get_type(prog_metadata).unwrap();
            //todo check both left and right types, in case one is a constant and the other is a var with a type we know
            load_src(
                left_src,
                src_type.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );
            // test for equality
            match src_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    wasm_instrs.push(WasmInstruction::I32Eq);
                }
                IrType::I64 | IrType::U64 => {
                    wasm_instrs.push(WasmInstruction::I64Eq);
                }
                IrType::F32 => {
                    wasm_instrs.push(WasmInstruction::F32Eq);
                }
                IrType::F64 => {
                    wasm_instrs.push(WasmInstruction::F64Eq);
                }
                t => {
                    println!("{t:?}");
                    unreachable!()
                }
            }

            function_context
                .control_flow_stack
                .push(ControlFlowElement::UnlabelledIf);

            let mut if_instrs = Vec::new();
            for instr in true_instrs {
                convert_ir_instr_to_wasm(
                    instr,
                    &mut if_instrs,
                    function_context,
                    module_context,
                    prog_metadata,
                );
            }

            let mut else_instrs = Vec::new();
            for instr in false_instrs {
                convert_ir_instr_to_wasm(
                    instr,
                    &mut else_instrs,
                    function_context,
                    module_context,
                    prog_metadata,
                );
            }

            wasm_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs,
                else_instrs,
            });

            function_context.control_flow_stack.pop();
        }
        Instruction::IfNotEqElse(_, left_src, right_src, true_instrs, false_instrs) => {
            // load operands for comparison
            let src_type = left_src.get_type(prog_metadata).unwrap();
            //todo check both left and right types, in case one is a constant and the other is a var with a type we know
            load_src(
                left_src,
                src_type.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );
            load_src(
                right_src,
                src_type.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );
            // test for equality
            match src_type {
                IrType::I8
                | IrType::U8
                | IrType::I16
                | IrType::U16
                | IrType::I32
                | IrType::U32
                | IrType::PointerTo(_)
                | IrType::ArrayOf(_, _) => {
                    wasm_instrs.push(WasmInstruction::I32Ne);
                }
                IrType::I64 | IrType::U64 => {
                    wasm_instrs.push(WasmInstruction::I64Ne);
                }
                IrType::F32 => {
                    wasm_instrs.push(WasmInstruction::F32Ne);
                }
                IrType::F64 => {
                    wasm_instrs.push(WasmInstruction::F64Ne);
                }
                t => {
                    println!("{t:?}");
                    unreachable!()
                }
            }

            function_context
                .control_flow_stack
                .push(ControlFlowElement::UnlabelledIf);

            let mut if_instrs = Vec::new();
            for instr in true_instrs {
                convert_ir_instr_to_wasm(
                    instr,
                    &mut if_instrs,
                    function_context,
                    module_context,
                    prog_metadata,
                );
            }

            let mut else_instrs = Vec::new();
            for instr in false_instrs {
                convert_ir_instr_to_wasm(
                    instr,
                    &mut else_instrs,
                    function_context,
                    module_context,
                    prog_metadata,
                );
            }

            wasm_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs,
                else_instrs,
            });

            function_context.control_flow_stack.pop();
        }
    }
}

fn convert_handled_blocks(
    mut handled_blocks: VecDeque<Block>,
    multiple_block_id: MultipleBlockId,
    function_context: &mut FunctionContext,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
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
    prog_metadata: &ProgramMetadata,
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

fn get_src_type_preferably_from_var(
    left_src: &Src,
    right_src: &Src,
    prog_metadata: &ProgramMetadata,
) -> IrType {
    match left_src {
        Src::Var(var_id) => prog_metadata.get_var_type(var_id).unwrap(),
        Src::Constant(_) => match right_src {
            Src::Var(var_id) => prog_metadata.get_var_type(var_id).unwrap(),
            Src::Constant(constant) => constant.get_type(None),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
