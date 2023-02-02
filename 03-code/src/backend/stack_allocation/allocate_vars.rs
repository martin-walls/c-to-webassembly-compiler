use crate::backend::memory_constants::PTR_SIZE;
use crate::backend::stack_allocation::naive_allocation::{
    naive_allocate_global_vars, naive_allocate_local_vars,
};
use crate::backend::stack_allocation::optimised_allocation::optimised_allocate_local_vars;
use crate::backend::stack_frame_operations::increment_stack_ptr_by_known_offset;
use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::wasm_instructions::WasmInstruction;
use crate::enabled_optimisations::EnabledOptimisations;
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::relooper::blocks::Block;
use std::collections::{HashMap, HashSet};

pub type VariableAllocationMap = HashMap<VarId, u32>;

pub fn allocate_local_vars(
    block: &Box<Block>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_type: Box<IrType>,
    fun_param_var_mappings: Vec<VarId>,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
    enabled_optimisations: &EnabledOptimisations,
) -> VariableAllocationMap {
    if enabled_optimisations.is_stack_allocation_optimisation_enabled() {
        optimised_allocate_local_vars(
            block,
            wasm_instrs,
            fun_type,
            fun_param_var_mappings,
            module_context,
            prog_metadata,
        )
    } else {
        naive_allocate_local_vars(
            block,
            wasm_instrs,
            fun_type,
            fun_param_var_mappings,
            module_context,
            prog_metadata,
        )
    }
}

pub fn allocate_global_vars(
    block: &Box<Block>,
    initial_top_of_stack_addr: u32,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> VariableAllocationMap {
    naive_allocate_global_vars(
        block,
        initial_top_of_stack_addr,
        wasm_instrs,
        module_context,
        prog_metadata,
    )
}
