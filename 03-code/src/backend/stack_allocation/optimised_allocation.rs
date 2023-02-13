use std::collections::{HashMap, HashSet};

use log::debug;

use crate::backend::dataflow_analysis::clash_graph::{generate_clash_graph, ClashGraph};
use crate::backend::dataflow_analysis::dead_code_analysis::remove_dead_vars;
use crate::backend::stack_allocation::allocate_vars::VariableAllocationMap;
use crate::backend::stack_allocation::clash_interval_var_locations::ClashIntervalVarLocations;
use crate::backend::stack_allocation::get_vars_from_block::get_vars_from_block;
use crate::backend::stack_allocation::interval_tree_var_locations::IntervalTreeVarLocations;
use crate::backend::stack_allocation::naive_var_locations::NaiveVarLocations;
use crate::backend::stack_allocation::var_locations::{VarLocation, VarLocations};
use crate::backend::stack_frame_operations::increment_stack_ptr_by_known_offset;
use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::wasm_instructions::WasmInstruction;
use crate::middle_end::ids::VarId;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::Block;

type VarAndByteSizePair = (VarId, u64);
type VarAllocationStack = Vec<VarAndByteSizePair>;

pub fn optimised_allocate_local_vars(
    block: &mut Block,
    param_vars_not_to_allocate_again: &Vec<VarId>,
    start_offset: u32,
    var_offsets: VariableAllocationMap,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
    prog_metadata: &mut ProgramMetadata,
) -> VariableAllocationMap {
    remove_dead_vars(block, prog_metadata);

    debug!("removed dead vars: {}", block);

    let clash_graph = generate_clash_graph(block);

    debug!("{}", clash_graph);

    // get all vars used in this block -- all the variables to allocate
    let mut vars_to_allocate = get_vars_from_block(block, prog_metadata);
    // remove param vars, cos we don't need to allocate them again
    for param_var in param_vars_not_to_allocate_again {
        vars_to_allocate.remove(param_var);
    }

    // pop vars from clash graph onto a stack
    let mut var_allocation_stack: VarAllocationStack = Vec::new();

    let mut temp_clash_graph = clash_graph.to_owned();

    while let Some(var_and_type) =
        pop_smallest_least_clashed_var(&mut vars_to_allocate, &mut temp_clash_graph, prog_metadata)
    {
        // var_allocation_stack.push(var_and_type);

        // if we make it FIFO, we'll allocate vars with the least clashes first.
        // Because we always allocate vars to the lowest possible addr given
        // the constraints of the existing allocations, this will put vars
        // with less clashes in lower addrs, so allow more overlap there
        var_allocation_stack.insert(0, var_and_type);
    }

    debug!("var allocation stack:");
    for var in &var_allocation_stack {
        debug!("  {}", var.0);
    }

    let var_locations = allocate_vars_from_stack(var_allocation_stack, &clash_graph, prog_metadata);

    let (var_offsets, total_offset) =
        calculate_var_offsets(var_locations, var_offsets, start_offset);

    increment_stack_ptr_by_known_offset(total_offset, wasm_instrs, module_context);

    var_offsets
}

/// Finds the var with the least number of clashes, breaking ties by the
/// smallest byte size. Removes that var from the vars left to allocate, and
/// from the clash graph.
/// If no vars are left, returns None.
fn pop_smallest_least_clashed_var(
    vars_left_to_allocate: &mut HashMap<VarId, IrType>,
    clash_graph: &mut ClashGraph,
    prog_metadata: &ProgramMetadata,
) -> Option<VarAndByteSizePair> {
    let mut min_var = None;
    let mut min_var_clash_count = 0;
    let mut min_var_byte_size = 0;

    for (var, var_type) in vars_left_to_allocate.iter() {
        let clash_count = clash_graph.count_clashes(var);
        let byte_size = var_type
            .get_byte_size(prog_metadata)
            .get_compile_time_value()
            .unwrap();

        if min_var.is_none()
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

    min_var.map(|min_var| (min_var, min_var_byte_size))
}

fn allocate_vars_from_stack(
    mut var_allocation_stack: VarAllocationStack,
    clash_graph: &ClashGraph,
    prog_metadata: &ProgramMetadata,
) -> HashSet<VarLocation> {
    let mut var_locations = ClashIntervalVarLocations::new();

    // allocate vars in order of the stack
    while let Some((var, byte_size)) = var_allocation_stack.pop() {
        // don't allocate the null dest
        if let Some(null_dest) = &prog_metadata.null_dest_var {
            if var == *null_dest {
                continue;
            }
        }
        debug!("allocating var {var}");

        // find the lowest addr where this var fits without clashing
        let lowest_possible_location =
            var_locations.find_lowest_non_clashing_location_for_var(var, byte_size, clash_graph);
        // allocate the var in the location we found
        var_locations.insert(lowest_possible_location, &clash_graph);
    }

    var_locations.into_hashset()
}

fn calculate_var_offsets(
    var_locations: HashSet<VarLocation>,
    mut var_offsets: VariableAllocationMap,
    start_offset: u32,
) -> (VariableAllocationMap, u32) {
    let mut total_offset = 0;
    for var_location in var_locations {
        if var_location.end_exclusive() > total_offset {
            total_offset = var_location.end_exclusive();
        }
        let offset = var_location.start + start_offset;
        debug!("offset {}: {}", var_location.var, offset);
        var_offsets.insert(var_location.var, offset);
    }
    (var_offsets, total_offset)
}
