use crate::backend::dataflow_analysis::clash_graph::{generate_clash_graph, ClashGraph};
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
    mut vars_to_allocate: HashMap<VarId, Box<IrType>>,
    block: &Box<Block>,
    start_offset: u32,
    mut var_offsets: VariableAllocationMap,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_param_var_mappings: Vec<VarId>,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> VariableAllocationMap {
    let flowgraph = generate_flowgraph(block);

    let live_vars = live_variable_analysis(&flowgraph);

    let clash_graph = generate_clash_graph(&live_vars);

    debug!("{}", clash_graph);

    // pop vars from clash graph onto a stack
    let mut var_allocation_stack = Vec::new();

    let mut temp_clash_graph = clash_graph.to_owned();

    while let Some(var) =
        pop_smallest_least_clashed_var(&mut vars_to_allocate, &mut temp_clash_graph, prog_metadata)
    {
        var_allocation_stack.push(var);
    }

    debug!("var allocation stack:");
    for var in &var_allocation_stack {
        debug!("  {}", var);
    }

    // allocate vars in order of the stack
    while let Some(var) = var_allocation_stack.pop() {
        // todo allocate var
    }

    todo!()
}

/// Finds the var with the least number of clashes, breaking ties by the
/// smallest byte size. Removes that var from the vars left to allocate, and
/// from the clash graph.
/// If no vars are left, returns None.
fn pop_smallest_least_clashed_var(
    vars_left_to_allocate: &mut HashMap<VarId, Box<IrType>>,
    clash_graph: &mut ClashGraph,
    prog_metadata: &Box<ProgramMetadata>,
) -> Option<VarId> {
    let mut min_var = None;
    let mut min_var_clash_count = 0;
    let mut min_var_byte_size = 0;

    for (var, var_type) in vars_left_to_allocate.iter() {
        let clash_count = clash_graph.count_clashes(var);
        let byte_size = var_type
            .get_byte_size(prog_metadata)
            .get_compile_time_value()
            .unwrap();

        if min_var == None
            || clash_count < min_var_clash_count
            || (clash_count == min_var_clash_count && byte_size < min_var_byte_size)
        {
            min_var = Some(var.to_owned());
            min_var_clash_count = clash_count;
            min_var_byte_size = byte_size;
        }
    }

    if let Some(min_var) = &min_var {
        vars_left_to_allocate.remove(min_var);
        clash_graph.remove_var(min_var);
    }

    min_var
}
