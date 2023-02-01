use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::Block;
use std::collections::HashMap;

pub fn get_vars_from_block(
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
                    | Instruction::DeclareVariable(dest)
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
                    | Instruction::F64toI32(dest, _)
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
                    | Instruction::I32toPtr(dest, _)
                    | Instruction::PtrToI32(dest, _) => {
                        let dest_type = prog_metadata.get_var_type(dest).unwrap();
                        vars.insert(dest.to_owned(), dest_type);
                        // vars.push((dest.to_owned(), dest_type));
                    }
                    Instruction::TailCall(_, _)
                    | Instruction::Ret(_)
                    | Instruction::Label(_)
                    | Instruction::Br(_)
                    | Instruction::BrIfEq(_, _, _)
                    | Instruction::BrIfNotEq(_, _, _)
                    | Instruction::Nop
                    | Instruction::Break(_)
                    | Instruction::Continue(_)
                    | Instruction::EndHandledBlock(_)
                    | Instruction::IfEqElse(_, _, _, _)
                    | Instruction::IfNotEqElse(_, _, _, _) => {}
                }
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_from_block(&next, prog_metadata));
                    vars
                }
            }
        }
        Block::Loop { inner, next, .. } => {
            let mut inner_block_vars = get_vars_from_block(&inner, prog_metadata);
            match next {
                None => inner_block_vars,
                Some(next) => {
                    inner_block_vars.extend(get_vars_from_block(&next, prog_metadata));
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
                vars.extend(get_vars_from_block(&handled, prog_metadata));
            }

            match next {
                None => vars,
                Some(next) => {
                    vars.extend(get_vars_from_block(&next, prog_metadata));
                    vars
                }
            }
        }
    }
}
