use crate::backend::dataflow_analysis::flowgraph::generate_flowgraph;
use crate::backend::dataflow_analysis::live_variable_analysis::live_variable_analysis;
use crate::backend::stack_allocation::allocate_vars::VariableAllocationMap;
use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::wasm_instructions::WasmInstruction;
use crate::middle_end::ids::VarId;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::Block;
use log::debug;
use std::collections::HashMap;

pub fn optimised_allocate_local_vars(
    block: &Box<Block>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_type: Box<IrType>,
    fun_param_var_mappings: Vec<VarId>,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> VariableAllocationMap {
    let flowgraph = generate_flowgraph(block);

    let live = live_variable_analysis(&flowgraph);

    debug!("{:#?}", live);

    todo!()
}
