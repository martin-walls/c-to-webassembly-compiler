use std::collections::HashMap;

use crate::back_end::memory_constants::PTR_SIZE;
use crate::back_end::stack_allocation::naive_allocation::{
    naive_allocate_global_vars, naive_allocate_local_vars,
};
use crate::back_end::stack_allocation::optimised_allocation::optimised_allocate_local_vars;
use crate::back_end::target_code_generation_context::ModuleContext;
use crate::back_end::wasm_instructions::WasmInstruction;
use crate::middle_end::ids::VarId;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::program_config::enabled_optimisations::EnabledOptimisations;
use crate::relooper::blocks::Block;

pub type VariableAllocationMap = HashMap<VarId, u32>;

pub fn allocate_local_vars(
    block: &mut Block,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_type: IrType,
    fun_param_var_mappings: Vec<VarId>,
    module_context: &ModuleContext,
    prog_metadata: &mut ProgramMetadata,
    enabled_optimisations: &EnabledOptimisations,
) -> VariableAllocationMap {
    let mut var_offsets: VariableAllocationMap = HashMap::new();
    let mut offset = PTR_SIZE;

    let (return_type, param_types) = match fun_type {
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
        var_offsets.insert(param_var_id.to_owned(), offset);
        offset += param_byte_size as u32;
    }

    if enabled_optimisations.is_stack_allocation_optimisation_enabled() {
        optimised_allocate_local_vars(
            block,
            &fun_param_var_mappings,
            offset,
            var_offsets,
            wasm_instrs,
            module_context,
            prog_metadata,
        )
    } else {
        naive_allocate_local_vars(
            block,
            &fun_param_var_mappings,
            offset,
            var_offsets,
            wasm_instrs,
            module_context,
            prog_metadata,
        )
    }
}

pub fn allocate_global_vars(
    block: &Block,
    initial_top_of_stack_addr: u32,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) -> VariableAllocationMap {
    naive_allocate_global_vars(
        block,
        initial_top_of_stack_addr,
        wasm_instrs,
        module_context,
        prog_metadata,
    )
}
