use crate::backend::memory_constants::PTR_SIZE;
use crate::backend::stack_frame_operations::increment_stack_ptr_by_known_offset;
use crate::backend::wasm_instructions::WasmInstruction;
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::relooper::blocks::Block;
use std::collections::{HashMap, HashSet};

pub fn allocate_local_vars(
    block: &Box<Block>,
    wasm_instrs: &mut Vec<WasmInstruction>,
    fun_type: Box<IrType>,
    fun_param_var_mappings: Vec<VarId>,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, u32> {
    let block_vars = get_vars_with_types(block, prog_metadata);

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
    increment_stack_ptr_by_known_offset(stack_ptr_increment, wasm_instrs);

    var_offsets
}

pub fn allocate_global_vars(
    block: &Box<Block>,
    initial_top_of_stack_addr: u32,
    wasm_instrs: &mut Vec<WasmInstruction>,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, u32> {
    let global_vars = get_vars_with_types(block, prog_metadata);

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

    increment_stack_ptr_by_known_offset(stack_ptr_increment, wasm_instrs);

    var_addrs
}

fn get_vars_with_types(
    block: &Box<Block>,
    prog_metadata: &Box<ProgramMetadata>,
) -> HashMap<VarId, Box<IrType>> {
    match &**block {
        Block::Simple { internal, next } => {
            let mut vars = HashMap::new();

            for instr in &internal.instrs {
                match instr {
                    Instruction::SimpleAssignment(dest, _)
                    | Instruction::LoadFromAddress(dest, _)
                    | Instruction::StoreToAddress(dest, _)
                    | Instruction::AllocateVariable(dest, _)
                    | Instruction::AddressOf(dest, _)
                    | Instruction::BitwiseNot(dest, _)
                    | Instruction::LogicalNot(dest, _)
                    | Instruction::Mult(dest, _, _)
                    | Instruction::Div(dest, _, _)
                    | Instruction::Mod(dest, _, _)
                    | Instruction::Add(dest, _, _)
                    | Instruction::Sub(dest, _, _)
                    | Instruction::LeftShift(dest, _, _)
                    | Instruction::RightShift(dest, _, _)
                    | Instruction::BitwiseAnd(dest, _, _)
                    | Instruction::BitwiseOr(dest, _, _)
                    | Instruction::BitwiseXor(dest, _, _)
                    | Instruction::LogicalAnd(dest, _, _)
                    | Instruction::LogicalOr(dest, _, _)
                    | Instruction::LessThan(dest, _, _)
                    | Instruction::GreaterThan(dest, _, _)
                    | Instruction::LessThanEq(dest, _, _)
                    | Instruction::GreaterThanEq(dest, _, _)
                    | Instruction::Equal(dest, _, _)
                    | Instruction::NotEqual(dest, _, _)
                    | Instruction::Call(dest, _, _)
                    | Instruction::PointerToStringLiteral(dest, _)
                    | Instruction::I8toI16(dest, _)
                    | Instruction::I8toU16(dest, _)
                    | Instruction::U8toI16(dest, _)
                    | Instruction::U8toU16(dest, _)
                    | Instruction::I16toI32(dest, _)
                    | Instruction::U16toI32(dest, _)
                    | Instruction::I16toU32(dest, _)
                    | Instruction::U16toU32(dest, _)
                    | Instruction::I32toU32(dest, _)
                    | Instruction::I32toU64(dest, _)
                    | Instruction::U32toU64(dest, _)
                    | Instruction::I64toU64(dest, _)
                    | Instruction::I32toI64(dest, _)
                    | Instruction::U32toI64(dest, _)
                    | Instruction::U32toF32(dest, _)
                    | Instruction::I32toF32(dest, _)
                    | Instruction::U64toF32(dest, _)
                    | Instruction::I64toF32(dest, _)
                    | Instruction::U32toF64(dest, _)
                    | Instruction::I32toF64(dest, _)
                    | Instruction::U64toF64(dest, _)
                    | Instruction::I64toF64(dest, _)
                    | Instruction::F32toF64(dest, _)
                    | Instruction::I32toI8(dest, _)
                    | Instruction::U32toI8(dest, _)
                    | Instruction::I64toI8(dest, _)
                    | Instruction::U64toI8(dest, _)
                    | Instruction::I32toU8(dest, _)
                    | Instruction::U32toU8(dest, _)
                    | Instruction::I64toU8(dest, _)
                    | Instruction::U64toU8(dest, _)
                    | Instruction::I64toI32(dest, _)
                    | Instruction::U64toI32(dest, _)
                    | Instruction::U32toPtr(dest, _)
                    | Instruction::I32toPtr(dest, _) => {
                        let dest_type = prog_metadata.get_var_type(dest).unwrap();
                        vars.insert(dest.to_owned(), dest_type);
                        // vars.push((dest.to_owned(), dest_type));
                    }
                    _ => {}
                }
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_with_types(&next, prog_metadata));
                    vars
                }
            }
        }
        Block::Loop { inner, next, .. } => {
            let mut inner_block_vars = get_vars_with_types(&inner, prog_metadata);
            match next {
                None => inner_block_vars,
                Some(next) => {
                    inner_block_vars.extend(get_vars_with_types(&next, prog_metadata));
                    inner_block_vars
                }
            }
        }
        Block::Multiple {
            handled_blocks,
            next,
            ..
        } => {
            let mut vars = HashMap::new();

            for handled in handled_blocks {
                vars.extend(get_vars_with_types(&handled, prog_metadata));
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_with_types(&next, prog_metadata));
                    vars
                }
            }
        }
    }
}
