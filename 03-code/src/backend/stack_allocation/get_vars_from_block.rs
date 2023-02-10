use std::collections::HashMap;

use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::Block;

pub fn get_vars_from_block(
    block: &Block,
    prog_metadata: &ProgramMetadata,
) -> HashMap<VarId, IrType> {
    match block {
        Block::Simple { internal, next } => {
            let mut vars = get_vars_from_instrs(&internal.instrs, prog_metadata);

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_from_block(next, prog_metadata));
                    vars
                }
            }
        }
        Block::Loop { inner, next, .. } => {
            let mut inner_block_vars = get_vars_from_block(inner, prog_metadata);
            match next {
                None => inner_block_vars,
                Some(next) => {
                    inner_block_vars.extend(get_vars_from_block(next, prog_metadata));
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
                vars.extend(get_vars_from_block(handled, prog_metadata));
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_from_block(next, prog_metadata));
                    vars
                }
            }
        }
    }
}

fn get_vars_from_instrs(
    instrs: &Vec<Instruction>,
    prog_metadata: &ProgramMetadata,
) -> HashMap<VarId, IrType> {
    let mut vars = HashMap::new();

    for instr in instrs {
        match instr {
            Instruction::SimpleAssignment(_, dest, _)
            | Instruction::LoadFromAddress(_, dest, _)
            | Instruction::StoreToAddress(_, dest, _)
            | Instruction::DeclareVariable(_, dest)
            | Instruction::AllocateVariable(_, dest, _)
            | Instruction::AddressOf(_, dest, _)
            | Instruction::BitwiseNot(_, dest, _)
            | Instruction::LogicalNot(_, dest, _)
            | Instruction::Mult(_, dest, _, _)
            | Instruction::Div(_, dest, _, _)
            | Instruction::Mod(_, dest, _, _)
            | Instruction::Add(_, dest, _, _)
            | Instruction::Sub(_, dest, _, _)
            | Instruction::LeftShift(_, dest, _, _)
            | Instruction::RightShift(_, dest, _, _)
            | Instruction::BitwiseAnd(_, dest, _, _)
            | Instruction::BitwiseOr(_, dest, _, _)
            | Instruction::BitwiseXor(_, dest, _, _)
            | Instruction::LogicalAnd(_, dest, _, _)
            | Instruction::LogicalOr(_, dest, _, _)
            | Instruction::LessThan(_, dest, _, _)
            | Instruction::GreaterThan(_, dest, _, _)
            | Instruction::LessThanEq(_, dest, _, _)
            | Instruction::GreaterThanEq(_, dest, _, _)
            | Instruction::Equal(_, dest, _, _)
            | Instruction::NotEqual(_, dest, _, _)
            | Instruction::Call(_, dest, _, _)
            | Instruction::PointerToStringLiteral(_, dest, _)
            | Instruction::I8toI16(_, dest, _)
            | Instruction::I8toU16(_, dest, _)
            | Instruction::U8toI16(_, dest, _)
            | Instruction::U8toU16(_, dest, _)
            | Instruction::I16toI32(_, dest, _)
            | Instruction::U16toI32(_, dest, _)
            | Instruction::I16toU32(_, dest, _)
            | Instruction::U16toU32(_, dest, _)
            | Instruction::I32toU32(_, dest, _)
            | Instruction::I32toU64(_, dest, _)
            | Instruction::U32toU64(_, dest, _)
            | Instruction::I64toU64(_, dest, _)
            | Instruction::I32toI64(_, dest, _)
            | Instruction::U32toI64(_, dest, _)
            | Instruction::U32toF32(_, dest, _)
            | Instruction::I32toF32(_, dest, _)
            | Instruction::U64toF32(_, dest, _)
            | Instruction::I64toF32(_, dest, _)
            | Instruction::U32toF64(_, dest, _)
            | Instruction::I32toF64(_, dest, _)
            | Instruction::U64toF64(_, dest, _)
            | Instruction::I64toF64(_, dest, _)
            | Instruction::F32toF64(_, dest, _)
            | Instruction::F64toI32(_, dest, _)
            | Instruction::I32toI8(_, dest, _)
            | Instruction::U32toI8(_, dest, _)
            | Instruction::I64toI8(_, dest, _)
            | Instruction::U64toI8(_, dest, _)
            | Instruction::I32toU8(_, dest, _)
            | Instruction::U32toU8(_, dest, _)
            | Instruction::I64toU8(_, dest, _)
            | Instruction::U64toU8(_, dest, _)
            | Instruction::I64toI32(_, dest, _)
            | Instruction::U64toI32(_, dest, _)
            | Instruction::U32toPtr(_, dest, _)
            | Instruction::I32toPtr(_, dest, _)
            | Instruction::PtrToI32(_, dest, _) => {
                if prog_metadata.is_var_the_null_dest(dest) {
                    continue;
                }
                let dest_type = prog_metadata.get_var_type(dest).unwrap();
                vars.insert(dest.to_owned(), dest_type);
                // vars.push((dest.to_owned(), dest_type));
            }
            Instruction::TailCall(..)
            | Instruction::Ret(..)
            | Instruction::Label(..)
            | Instruction::Br(..)
            | Instruction::BrIfEq(..)
            | Instruction::BrIfNotEq(..)
            | Instruction::Nop(..)
            | Instruction::Break(..)
            | Instruction::Continue(..)
            | Instruction::EndHandledBlock(..)
            | Instruction::ReferenceVariable(..) => {}
            Instruction::IfEqElse(_, _, _, instrs1, instrs2)
            | Instruction::IfNotEqElse(_, _, _, instrs1, instrs2) => {
                vars.extend(get_vars_from_instrs(instrs1, prog_metadata));
                vars.extend(get_vars_from_instrs(instrs2, prog_metadata));
            }
        }
    }

    vars
}
