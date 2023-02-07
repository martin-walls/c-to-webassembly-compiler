use std::collections::HashSet;

use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::{Instruction, Src};

pub fn def_set(instr: &Instruction) -> HashSet<VarId> {
    let mut defined_vars = HashSet::new();
    match instr {
        Instruction::SimpleAssignment(_, dest, _)
        | Instruction::LoadFromAddress(_, dest, _)
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
            defined_vars.insert(dest.to_owned());
        }
        _ => {}
    }

    defined_vars
}

pub fn ref_set(instr: &Instruction) -> HashSet<VarId> {
    let mut referenced_vars = HashSet::new();

    match instr {
        Instruction::SimpleAssignment(_, _, src)
        | Instruction::LoadFromAddress(_, _, src)
        | Instruction::AllocateVariable(_, _, src)
        | Instruction::AddressOf(_, _, src)
        | Instruction::BitwiseNot(_, _, src)
        | Instruction::LogicalNot(_, _, src)
        | Instruction::BrIfEq(_, _, src, _)
        | Instruction::BrIfNotEq(_, _, src, _)
        | Instruction::I8toI16(_, _, src)
        | Instruction::I8toU16(_, _, src)
        | Instruction::U8toI16(_, _, src)
        | Instruction::U8toU16(_, _, src)
        | Instruction::I16toI32(_, _, src)
        | Instruction::U16toI32(_, _, src)
        | Instruction::I16toU32(_, _, src)
        | Instruction::U16toU32(_, _, src)
        | Instruction::I32toU32(_, _, src)
        | Instruction::I32toU64(_, _, src)
        | Instruction::U32toU64(_, _, src)
        | Instruction::I64toU64(_, _, src)
        | Instruction::I32toI64(_, _, src)
        | Instruction::U32toI64(_, _, src)
        | Instruction::U32toF32(_, _, src)
        | Instruction::I32toF32(_, _, src)
        | Instruction::U64toF32(_, _, src)
        | Instruction::I64toF32(_, _, src)
        | Instruction::U32toF64(_, _, src)
        | Instruction::I32toF64(_, _, src)
        | Instruction::U64toF64(_, _, src)
        | Instruction::I64toF64(_, _, src)
        | Instruction::F32toF64(_, _, src)
        | Instruction::F64toI32(_, _, src)
        | Instruction::I32toI8(_, _, src)
        | Instruction::U32toI8(_, _, src)
        | Instruction::I64toI8(_, _, src)
        | Instruction::U64toI8(_, _, src)
        | Instruction::I32toU8(_, _, src)
        | Instruction::U32toU8(_, _, src)
        | Instruction::I64toU8(_, _, src)
        | Instruction::U64toU8(_, _, src)
        | Instruction::I64toI32(_, _, src)
        | Instruction::U64toI32(_, _, src)
        | Instruction::U32toPtr(_, _, src)
        | Instruction::I32toPtr(_, _, src)
        | Instruction::PtrToI32(_, _, src) => match src {
            Src::Var(var) | Src::StoreAddressVar(var) => {
                referenced_vars.insert(var.to_owned());
            }
            Src::Constant(_) => {}
            Src::Fun(_) => {}
        },
        Instruction::StoreToAddress(_, src1, src2) => {
            referenced_vars.insert(src1.to_owned());
            match src2 {
                Src::Var(var) | Src::StoreAddressVar(var) => {
                    referenced_vars.insert(var.to_owned());
                }
                _ => {}
            }
        }
        Instruction::Mult(_, _, src1, src2)
        | Instruction::Div(_, _, src1, src2)
        | Instruction::Mod(_, _, src1, src2)
        | Instruction::Add(_, _, src1, src2)
        | Instruction::Sub(_, _, src1, src2)
        | Instruction::LeftShift(_, _, src1, src2)
        | Instruction::RightShift(_, _, src1, src2)
        | Instruction::BitwiseAnd(_, _, src1, src2)
        | Instruction::BitwiseOr(_, _, src1, src2)
        | Instruction::BitwiseXor(_, _, src1, src2)
        | Instruction::LogicalAnd(_, _, src1, src2)
        | Instruction::LogicalOr(_, _, src1, src2)
        | Instruction::LessThan(_, _, src1, src2)
        | Instruction::GreaterThan(_, _, src1, src2)
        | Instruction::LessThanEq(_, _, src1, src2)
        | Instruction::GreaterThanEq(_, _, src1, src2)
        | Instruction::Equal(_, _, src1, src2)
        | Instruction::NotEqual(_, _, src1, src2)
        | Instruction::IfEqElse(_, src1, src2, _, _)
        | Instruction::IfNotEqElse(_, src1, src2, _, _) => {
            match src1 {
                Src::Var(var) | Src::StoreAddressVar(var) => {
                    referenced_vars.insert(var.to_owned());
                }
                _ => {}
            }
            match src2 {
                Src::Var(var) | Src::StoreAddressVar(var) => {
                    referenced_vars.insert(var.to_owned());
                }
                _ => {}
            }
        }
        Instruction::Call(_, _, _, srcs) | Instruction::TailCall(_, _, srcs) => {
            for src in srcs {
                match src {
                    Src::Var(var) | Src::StoreAddressVar(var) => {
                        referenced_vars.insert(var.to_owned());
                    }
                    _ => {}
                }
            }
        }
        Instruction::Ret(_, src) => {
            if let Some(src) = src {
                match src {
                    Src::Var(var) | Src::StoreAddressVar(var) => {
                        referenced_vars.insert(var.to_owned());
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    referenced_vars
}
