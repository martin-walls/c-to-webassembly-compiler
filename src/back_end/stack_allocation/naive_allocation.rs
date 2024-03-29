use std::collections::HashMap;

use crate::back_end::stack_allocation::allocate_vars::VariableAllocationMap;
use crate::back_end::stack_allocation::get_vars_from_block::get_vars_from_block;
use crate::back_end::stack_frame_operations::increment_stack_ptr_by_known_offset;
use crate::back_end::target_code_generation_context::ModuleContext;
use crate::back_end::wasm_instructions::WasmInstruction;
use crate::middle_end::ids::VarId;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::TypeSize;
use crate::relooper::blocks::Block;

pub fn naive_allocate_local_vars(
    block: &Block,
    param_vars_not_to_allocate_again: &Vec<VarId>,
    start_offset: u32,
    mut var_offsets: VariableAllocationMap,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) -> VariableAllocationMap {
    // get all vars used in this block -- all the variables to allocate
    let mut vars = get_vars_from_block(block, prog_metadata);
    // remove param vars, cos we don't need to allocate them again
    for param_var in param_vars_not_to_allocate_again {
        vars.remove(param_var);
    }

    let mut offset = 0;

    // calculate offset of each local variable
    for (var_id, var_type) in vars {
        let byte_size = match var_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(byte_size) => byte_size,
            TypeSize::Runtime(_) => {
                // we shouldn't be trying to allocate a variable with runtime-known byte size
                // on the stack here. its space is allocated with the AllocateVariable instruction,
                // with just a pointer on the stack here
                unreachable!()
            }
        };
        var_offsets.insert(var_id, start_offset + offset);
        offset += byte_size as u32;
    }

    // update stack pointer to after allocated vars
    increment_stack_ptr_by_known_offset(offset, wasm_instrs, module_context);

    var_offsets
}

pub fn naive_allocate_global_vars(
    block: &Block,
    initial_top_of_stack_addr: u32,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
    prog_metadata: &ProgramMetadata,
) -> VariableAllocationMap {
    let global_vars = get_vars_from_block(block, prog_metadata);

    let mut var_addrs = HashMap::new();
    let mut addr = initial_top_of_stack_addr;
    // how much to increment the stack pointer by when we've allocated all vars
    let mut stack_ptr_increment = 0;

    // calculate addr of each global var
    for (var_id, var_type) in global_vars {
        let byte_size = match var_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(byte_size) => byte_size,
            TypeSize::Runtime(_) => {
                unreachable!()
            }
        };
        var_addrs.insert(var_id, addr);
        addr += byte_size as u32;
        stack_ptr_increment += byte_size as u32;
    }

    increment_stack_ptr_by_known_offset(stack_ptr_increment, wasm_instrs, module_context);

    var_addrs
}
