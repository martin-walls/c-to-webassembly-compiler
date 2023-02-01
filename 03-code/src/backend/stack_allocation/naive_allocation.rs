use crate::backend::memory_constants::PTR_SIZE;
use crate::backend::stack_allocation::get_vars_from_block::get_vars_from_block;
use crate::backend::stack_frame_operations::increment_stack_ptr_by_known_offset;
use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::wasm_instructions::WasmInstruction;
use crate::middle_end::ids::VarId;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::relooper::blocks::Block;
use std::collections::{HashMap, HashSet};

pub fn naive_allocate_local_vars(
    block: &Box<Block>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_type: Box<IrType>,
    fun_param_var_mappings: Vec<VarId>,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, u32> {
    let block_vars = get_vars_from_block(block, prog_metadata);

    let mut var_offsets = HashMap::new();
    let mut offset = PTR_SIZE;
    // only increment the stack ptr by the size we allocated for local vars, not params and return value
    let mut stack_ptr_increment = 0;

    let (return_type, param_types) = match *fun_type {
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

    let mut param_var_ids = HashSet::new();

    // calculate offset of each param variable
    for param_i in 0..param_types.len() {
        let param_type = param_types.get(param_i).unwrap();
        let param_var_id = fun_param_var_mappings.get(param_i).unwrap();
        param_var_ids.insert(param_var_id.to_owned());
        let param_byte_size = match param_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(size) => size,
            TypeSize::Runtime(_) => {
                unreachable!()
            }
        };
        var_offsets.insert(param_var_id.to_owned(), offset);
        offset += param_byte_size as u32;
    }

    // calculate offset of each local variable
    for (var_id, var_type) in block_vars {
        // check if the variable is also a parameter and has therefore
        // already been allocated
        if param_var_ids.contains(&var_id) {
            continue;
        }
        let byte_size = match var_type.get_byte_size(prog_metadata) {
            TypeSize::CompileTime(byte_size) => byte_size,
            TypeSize::Runtime(_) => {
                // we shouldn't be trying to allocate a variable with runtime-known byte size
                // on the stack here. its space is allocated with the AllocateVariable instruction,
                // with just a pointer on the stack here
                unreachable!()
            }
        };
        var_offsets.insert(var_id, offset);
        offset += byte_size as u32;
        stack_ptr_increment += byte_size as u32;
    }

    // update stack pointer to after allocated vars
    increment_stack_ptr_by_known_offset(stack_ptr_increment, wasm_instrs, module_context);

    var_offsets
}

pub fn naive_allocate_global_vars(
    block: &Box<Block>,
    initial_top_of_stack_addr: u32,
    wasm_instrs: &mut Vec<WasmInstruction>,
    module_context: &ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, u32> {
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
