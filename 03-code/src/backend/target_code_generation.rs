use log::info;
use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};

use crate::backend::allocate_local_vars::allocate_local_vars;
use crate::backend::backend_error::BackendError;
use crate::backend::memory_operations::{load, load_src, load_var, store, store_var};
use crate::backend::stack_frame_operations::{
    increment_stack_ptr_by_known_offset, increment_stack_ptr_dynamic, load_frame_ptr,
    load_stack_ptr, pop_stack_frame, set_frame_ptr_to_stack_ptr, set_up_new_stack_frame,
};
use crate::backend::target_code_generation_context::{
    ControlFlowElement, FunctionContext, ModuleContext,
};
use crate::backend::wasm_indices::{FuncIdx, LabelIdx, LocalIdx, MemIdx, TypeIdx};
use crate::backend::wasm_instructions::{BlockType, MemArg, WasmExpression, WasmInstruction};
use crate::backend::wasm_module::data_section::DataSegment;
use crate::backend::wasm_module::exports_section::{ExportDescriptor, WasmExport};
use crate::backend::wasm_module::module::WasmModule;
use crate::backend::wasm_module::types_section::WasmFunctionType;
use crate::backend::wasm_types::{Limits, MemoryType, NumType, ValType};
use crate::middle_end::ids::{FunId, Id, LabelId, VarId};
use crate::middle_end::instructions::{Instruction, Src};
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::{Block, LoopBlockId, MultipleBlockId};
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};

pub const PTR_SIZE: u32 = 4;

pub const FRAME_PTR_ADDR: u32 = 0;
pub const STACK_PTR_ADDR: u32 = FRAME_PTR_ADDR + PTR_SIZE;

pub const MAIN_FUNCTION_SOURCE_NAME: &str = "main";
pub const MAIN_FUNCTION_EXPORT_NAME: &str = "main";
pub const MEMORY_EXPORT_NAME: &str = "memory";

pub fn generate_target_code(prog: ReloopedProgram) -> Result<WasmModule, BackendError> {
    let mut wasm_module = WasmModule::new();

    let (imported_functions, defined_functions) = separate_imported_and_defined_functions(
        &prog.program_metadata,
        prog.program_blocks.functions,
    );

    let mut module_context = ModuleContext::new();

    module_context.calculate_func_idxs(&imported_functions, &defined_functions);

    initialise_memory(
        &mut wasm_module,
        &mut module_context,
        &prog.program_metadata,
    );

    // insert empty function type to module
    let empty_type = WasmFunctionType {
        param_types: Vec::new(),
        result_types: Vec::new(),
    };
    let empty_type_idx = wasm_module.insert_type(empty_type);

    let mut func_idx_to_type_idx_map: HashMap<FuncIdx, TypeIdx> = HashMap::new();
    let mut func_idx_to_body_code_map: HashMap<FuncIdx, WasmExpression> = HashMap::new();

    for (fun_id, function) in defined_functions {
        let wasm_func_idx = module_context.fun_id_to_func_idx_map.get(&fun_id).unwrap();

        // all functions have empty type, because params/result are stored in stack frame
        func_idx_to_type_idx_map.insert(wasm_func_idx.to_owned(), empty_type_idx.to_owned());

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

    if let Some(global_block) = prog.program_blocks.global_instrs {
        let var_offsets = allocate_local_vars(
            &global_block,
            &mut global_wasm_instrs,
            Box::new(IrType::Function(Box::new(IrType::Void), Vec::new(), false)), // global instructions have a void function type, so don't allocate any space for a return value
            Vec::new(),                                                            // no parameters
            &prog.program_metadata,
        );

        let mut function_context = FunctionContext::new(
            var_offsets,
            VarId::initial_id(), // global instructions don't have any control flow, so just put a dummy var here
        );

        global_wasm_instrs.append(&mut convert_block_to_wasm(
            global_block,
            &mut function_context,
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
    increment_stack_ptr_by_known_offset(PTR_SIZE + i32_byte_size as u32, &mut global_wasm_instrs);

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
    increment_stack_ptr_by_known_offset(i32_byte_size as u32, &mut global_wasm_instrs);

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
    increment_stack_ptr_by_known_offset(i32_byte_size as u32, &mut global_wasm_instrs);

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

fn initialise_memory(
    wasm_module: &mut WasmModule,
    module_context: &mut ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) {
    // ------------------------------------------------
    // | FP | SP | String literals | ...stack frames...
    // ------------------------------------------------
    let mut data: Vec<u8> = Vec::new();
    // temp placeholder values for frame ptr and stack ptr
    for _ in 0..2 {
        // for each ptr
        for _ in 0..PTR_SIZE {
            // allocate PTR_SIZE bytes
            data.push(0x00);
        }
    }

    // store string literals in memory
    for (string_literal_id, string) in &prog_metadata.string_literals {
        // store the pointer to the string
        let ptr_to_string = data.len();
        info!(
            "Storing string literal {:?} at addr {}",
            string, ptr_to_string
        );
        module_context
            .string_literal_id_to_ptr_map
            .insert(string_literal_id.to_owned(), ptr_to_string as u32);
        // insert the string
        data.append(&mut string.as_bytes().to_vec());
        // null terminate the string
        data.push(0x00);
    }

    // set stack ptr to point at top of stack
    let stack_ptr_value = data.len();
    info!("Setting stack ptr to {}", stack_ptr_value);
    data[PTR_SIZE as usize] = (stack_ptr_value & 0xFF) as u8;
    data[(PTR_SIZE + 1) as usize] = ((stack_ptr_value >> 8) & 0xFF) as u8;
    data[(PTR_SIZE + 2) as usize] = ((stack_ptr_value >> 16) & 0xFF) as u8;
    data[(PTR_SIZE + 3) as usize] = ((stack_ptr_value >> 24) & 0xFF) as u8;

    // insert data segment to module
    let data_segment = DataSegment::ActiveSegmentMemIndexZero {
        offset_expr: WasmExpression {
            instrs: vec![WasmInstruction::I32Const { n: 0 }],
        },
        data,
    };
    wasm_module.data_section.data_segments.push(data_segment);

    // declare memory, with min 1 page, no max
    wasm_module.memory_section.memory_types.push(MemoryType {
        limits: Limits { min: 1, max: None },
    });

    // export memory
    let memory_export = WasmExport {
        name: MEMORY_EXPORT_NAME.to_owned(),
        export_descriptor: ExportDescriptor::Mem {
            mem_idx: MemIdx { x: 0 },
        },
    };
    wasm_module.exports_section.exports.push(memory_export);
}

// fn convert_function_type_to_wasm(ir_type: &Box<IrType>) -> WasmFunctionType {
//     let (fun_return_type, fun_param_types) = match &**ir_type {
//         IrType::Function(return_type, param_types, _) => (return_type, param_types),
//         _ => unreachable!(),
//     };
//
//     let mut param_types = Vec::new();
//
//     for fun_param_type in fun_param_types {
//         match **fun_param_type {
//             IrType::I8
//             | IrType::U8
//             | IrType::I16
//             | IrType::U16
//             | IrType::I32
//             | IrType::U32
//             | IrType::PointerTo(_)
//             | IrType::ArrayOf(_, _)
//             | IrType::Struct(_)
//             | IrType::Union(_) => param_types.push(ValType::NumType(NumType::I32)),
//             IrType::I64 | IrType::U64 => param_types.push(ValType::NumType(NumType::I64)),
//             IrType::F32 => param_types.push(ValType::NumType(NumType::F32)),
//             IrType::F64 => param_types.push(ValType::NumType(NumType::F64)),
//             _ => {
//                 unreachable!()
//             }
//         }
//     }
//
//     let mut result_types = Vec::new();
//     match **fun_return_type {
//         IrType::I8
//         | IrType::U8
//         | IrType::I16
//         | IrType::U16
//         | IrType::I32
//         | IrType::U32
//         | IrType::PointerTo(_)
//         | IrType::ArrayOf(_, _)
//         | IrType::Struct(_)
//         | IrType::Union(_) => result_types.push(ValType::NumType(NumType::I32)),
//         IrType::I64 | IrType::U64 => result_types.push(ValType::NumType(NumType::I64)),
//         IrType::F32 => result_types.push(ValType::NumType(NumType::F32)),
//         IrType::F64 => result_types.push(ValType::NumType(NumType::F64)),
//         IrType::Void => {}
//         _ => {
//             unreachable!()
//         }
//     }
//
//     WasmFunctionType {
//         param_types,
//         result_types,
//     }
// }

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
            let mut loop_instrs =
                convert_block_to_wasm(inner, function_context, module_context, prog_metadata);
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
        Instruction::LoadFromAddress(dest, src) => {
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
        Instruction::StoreToAddress(dest, src) => {
            // store the value of src to the location pointed to by dest
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            let inner_dest_type = dest_type.dereference_pointer_type().unwrap();

            // load the value of dest - the address operand
            load_var(
                dest.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );

            // load the value to store
            load_src(
                src.to_owned(),
                inner_dest_type.to_owned(),
                wasm_instrs,
                function_context,
                prog_metadata,
            );

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
                Box::new(IrType::I32),
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
                    let mut temp_instrs = Vec::new();
                    load_frame_ptr(&mut temp_instrs);
                    temp_instrs.push(WasmInstruction::I32Const {
                        n: *fp_offset as i32,
                    });
                    temp_instrs.push(WasmInstruction::I32Add);
                    store_var(
                        dest,
                        temp_instrs,
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
            match *dest_type {
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
        Instruction::LogicalNot(dest, src) => {
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
            match *dest_type {
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
        Instruction::Mult(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::Div(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::Mod(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::Add(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::Sub(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::LeftShift(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::RightShift(dest, left_src, right_src) => {
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

            // shift right
            match *dest_type {
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
        Instruction::BitwiseAnd(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::BitwiseOr(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::BitwiseXor(dest, left_src, right_src) => {
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
            match *dest_type {
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
        Instruction::LogicalAnd(dest, left_src, right_src) => {
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
            match *dest_type {
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
            // if so, push 1, else 0
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs: vec![WasmInstruction::I32Const { n: 1 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 0 }],
            });

            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // test it right_src is zero
            match *dest_type {
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
            // if so, push 1, else 0
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs: vec![WasmInstruction::I32Const { n: 1 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 0 }],
            });

            // bitwise AND
            match *dest_type {
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
        Instruction::LogicalOr(dest, left_src, right_src) => {
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
            match *dest_type {
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
            // if so, push 1, else 0
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs: vec![WasmInstruction::I32Const { n: 1 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 0 }],
            });

            load_src(
                right_src,
                dest_type.to_owned(),
                &mut temp_instrs,
                function_context,
                prog_metadata,
            );
            // test it right_src is zero
            match *dest_type {
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
            // if so, push 1, else 0
            temp_instrs.push(WasmInstruction::IfElse {
                blocktype: BlockType::None,
                if_instrs: vec![WasmInstruction::I32Const { n: 1 }],
                else_instrs: vec![WasmInstruction::I32Const { n: 0 }],
            });

            // bitwise OR
            match *dest_type {
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
        Instruction::LessThan(dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            // load srcs onto wasm stack
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

            // less than
            match *dest_type {
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
        Instruction::GreaterThan(dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            // load srcs onto wasm stack
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

            // greater than
            match *dest_type {
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
        Instruction::LessThanEq(dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            // load srcs onto wasm stack
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

            // less than or equal
            match *dest_type {
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
        Instruction::GreaterThanEq(dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            // load srcs onto wasm stack
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

            // greater than or equal
            match *dest_type {
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
        Instruction::Equal(dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            // load srcs onto wasm stack
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

            // equal
            match *dest_type {
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
                    println!("{:?}", t);
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
        Instruction::NotEqual(dest, left_src, right_src) => {
            let mut temp_instrs = Vec::new();
            let dest_type = prog_metadata.get_var_type(&dest).unwrap();
            // load srcs onto wasm stack
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

            // not equal
            match *dest_type {
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
                    println!("{:?}", t);
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
        Instruction::Ret(return_value_src) => {
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
        Instruction::Label(_) => {
            // labels don't do anything anymore at this stage, so just ignore them
        }
        Instruction::Br(_) | Instruction::BrIfEq(_, _, _) | Instruction::BrIfNotEq(_, _, _) => {
            unreachable!("Br instructions have all been replaced by this point")
        }
        Instruction::PointerToStringLiteral(dest, str_literal_id) => {
            let ptr_value = module_context
                .string_literal_id_to_ptr_map
                .get(&str_literal_id)
                .unwrap();
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
        Instruction::I8toI16(dest, src)
        | Instruction::I8toU16(dest, src)
        | Instruction::U8toI16(dest, src)
        | Instruction::U8toU16(dest, src)
        | Instruction::I16toI32(dest, src)
        | Instruction::U16toI32(dest, src)
        | Instruction::I16toU32(dest, src)
        | Instruction::U16toU32(dest, src)
        | Instruction::I32toU32(dest, src)
        | Instruction::I32toU64(dest, src)
        | Instruction::U32toU64(dest, src)
        | Instruction::I64toU64(dest, src)
        | Instruction::I32toI64(dest, src)
        | Instruction::U32toI64(dest, src)
        | Instruction::U32toF32(dest, src)
        | Instruction::I32toF32(dest, src)
        | Instruction::U64toF32(dest, src)
        | Instruction::I64toF32(dest, src)
        | Instruction::U32toF64(dest, src)
        | Instruction::I32toF64(dest, src)
        | Instruction::U64toF64(dest, src)
        | Instruction::I64toF64(dest, src)
        | Instruction::F32toF64(dest, src)
        | Instruction::I32toI8(dest, src)
        | Instruction::U32toI8(dest, src)
        | Instruction::I64toI8(dest, src)
        | Instruction::U64toI8(dest, src)
        | Instruction::I32toU8(dest, src)
        | Instruction::U32toU8(dest, src)
        | Instruction::I64toU8(dest, src)
        | Instruction::U64toU8(dest, src)
        | Instruction::I64toI32(dest, src)
        | Instruction::U64toI32(dest, src)
        | Instruction::U32toPtr(dest, src)
        | Instruction::I32toPtr(dest, src) => {
            // todo need to look at the types and insert load/store instructions that match type.
            //      Otherwise wasm complains eg. that i64 load doesn't match i32 store.
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

        Instruction::Nop => {
            // do nothing
        }
        Instruction::Break(loop_block_id) => {
            // get depth of block to br out of
            let depth = function_context.get_depth_of_block(&loop_block_id).unwrap();
            wasm_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: depth },
            });
        }
        Instruction::Continue(loop_block_id) => {
            // get depth of loop to br to start of
            let depth = function_context.get_depth_of_loop(&loop_block_id).unwrap();
            wasm_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: depth },
            });
        }
        Instruction::EndHandledBlock(multiple_block_id) => {
            // get depth of if to br out of
            let depth = function_context
                .get_depth_of_if(&multiple_block_id)
                .unwrap();
            wasm_instrs.push(WasmInstruction::Br {
                label_idx: LabelIdx { l: depth },
            });
        }
        Instruction::IfEqElse(left_src, right_src, true_instrs, false_instrs) => {
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
            match *src_type {
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
                    println!("{:?}", t);
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
        Instruction::IfNotEqElse(left_src, right_src, true_instrs, false_instrs) => {
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
            match *src_type {
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
                    println!("{:?}", t);
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
