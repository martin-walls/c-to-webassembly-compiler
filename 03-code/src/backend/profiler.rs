use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::wasm_instructions::WasmInstruction;
use crate::middle_end::ir_types::IrType;
use crate::relooper::relooper::{ReloopedFunction, ReloopedProgram};

const LOG_STACK_PTR_IMPORT_NAME: &str = "log_stack_ptr";

pub fn initialise_profiler(module_context: &mut ModuleContext, prog: &mut ReloopedProgram) {
    // initialise stack pointer logging
    if module_context
        .enabled_profiling
        .is_stack_ptr_logging_enabled()
    {
        // declare new fun for log fun
        let log_stack_ptr_fun_id = prog
            .program_metadata
            .new_fun_declaration(LOG_STACK_PTR_IMPORT_NAME.to_owned())
            .unwrap();

        // insert function stub to program
        prog.program_blocks.functions.insert(
            log_stack_ptr_fun_id.to_owned(),
            ReloopedFunction {
                block: None,
                label_variable: None,
                type_info: IrType::Function(Box::new(IrType::Void), Vec::new(), false),
                param_var_mappings: Vec::new(),
                body_is_defined: false,
            },
        );

        // store fun id in module context
        module_context.log_stack_ptr_fun_id = Some(log_stack_ptr_fun_id);
    }
}

pub fn log_stack_ptr(wasm_instrs: &mut Vec<WasmInstruction>, module_context: &ModuleContext) {
    // check if stack pointer logging is enabled
    if !module_context
        .enabled_profiling
        .is_stack_ptr_logging_enabled()
    {
        return;
    }
    if let Some(log_stack_ptr_fun_id) = &module_context.log_stack_ptr_fun_id {
        let log_stack_ptr_func_idx = module_context
            .fun_id_to_func_idx_map
            .get(log_stack_ptr_fun_id)
            .unwrap();
        wasm_instrs.push(WasmInstruction::Call {
            func_idx: log_stack_ptr_func_idx.to_owned(),
        })
    }
}
