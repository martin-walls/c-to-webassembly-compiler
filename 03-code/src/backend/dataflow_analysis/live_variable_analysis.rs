use crate::backend::dataflow_analysis::flowgraph::{Flowgraph, InstructionId};
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::{Instruction, Src};
use std::collections::{HashMap, HashSet};

pub type LiveVariableMap = HashMap<InstructionId, HashSet<VarId>>;

pub fn live_variable_analysis(flowgraph: &Flowgraph) -> LiveVariableMap {
    // for every instr, which vars are live at that point
    let mut live: LiveVariableMap = HashMap::new();

    let mut changes = true;
    while changes {
        changes = false;

        for (instr_id, instr) in &flowgraph.instrs {
            // U_{s in succ} live(s)
            let mut out_live: HashSet<VarId> = HashSet::new();
            for successor in flowgraph.successors.get(instr_id).unwrap() {
                out_live.extend(live.get(successor).unwrap_or(&HashSet::new()).to_owned());
            }

            // \ def(n)
            for def_var in def_set(instr) {
                out_live.remove(&def_var);
            }

            // U ref(n)
            for ref_var in ref_set(instr) {
                out_live.insert(ref_var);
            }

            let prev_live = live.insert(instr_id.to_owned(), out_live.to_owned());

            match prev_live {
                None => {
                    changes = true;
                }
                Some(prev_live_vars) => {
                    if prev_live_vars != out_live {
                        changes = true
                    }
                }
            }
        }
    }

    live
}

fn def_set(instr: &Instruction) -> HashSet<VarId> {
    let mut defined_vars = HashSet::new();
    match instr {
        Instruction::SimpleAssignment(dest, _)
        | Instruction::LoadFromAddress(dest, _)
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
            defined_vars.insert(dest.to_owned());
        }
        _ => {}
    }

    defined_vars
}

fn ref_set(instr: &Instruction) -> HashSet<VarId> {
    let mut referenced_vars = HashSet::new();

    match instr {
        Instruction::SimpleAssignment(_, src)
        | Instruction::LoadFromAddress(_, src)
        | Instruction::AllocateVariable(_, src)
        | Instruction::AddressOf(_, src)
        | Instruction::BitwiseNot(_, src)
        | Instruction::LogicalNot(_, src)
        | Instruction::BrIfEq(_, src, _)
        | Instruction::BrIfNotEq(_, src, _)
        | Instruction::I8toI16(_, src)
        | Instruction::I8toU16(_, src)
        | Instruction::U8toI16(_, src)
        | Instruction::U8toU16(_, src)
        | Instruction::I16toI32(_, src)
        | Instruction::U16toI32(_, src)
        | Instruction::I16toU32(_, src)
        | Instruction::U16toU32(_, src)
        | Instruction::I32toU32(_, src)
        | Instruction::I32toU64(_, src)
        | Instruction::U32toU64(_, src)
        | Instruction::I64toU64(_, src)
        | Instruction::I32toI64(_, src)
        | Instruction::U32toI64(_, src)
        | Instruction::U32toF32(_, src)
        | Instruction::I32toF32(_, src)
        | Instruction::U64toF32(_, src)
        | Instruction::I64toF32(_, src)
        | Instruction::U32toF64(_, src)
        | Instruction::I32toF64(_, src)
        | Instruction::U64toF64(_, src)
        | Instruction::I64toF64(_, src)
        | Instruction::F32toF64(_, src)
        | Instruction::F64toI32(_, src)
        | Instruction::I32toI8(_, src)
        | Instruction::U32toI8(_, src)
        | Instruction::I64toI8(_, src)
        | Instruction::U64toI8(_, src)
        | Instruction::I32toU8(_, src)
        | Instruction::U32toU8(_, src)
        | Instruction::I64toU8(_, src)
        | Instruction::U64toU8(_, src)
        | Instruction::I64toI32(_, src)
        | Instruction::U64toI32(_, src)
        | Instruction::U32toPtr(_, src)
        | Instruction::I32toPtr(_, src)
        | Instruction::PtrToI32(_, src) => match src {
            Src::Var(var) | Src::StoreAddressVar(var) => {
                referenced_vars.insert(var.to_owned());
            }
            Src::Constant(_) => {}
            Src::Fun(_) => {}
        },
        Instruction::StoreToAddress(src1, src2) => {
            referenced_vars.insert(src1.to_owned());
            match src2 {
                Src::Var(var) | Src::StoreAddressVar(var) => {
                    referenced_vars.insert(var.to_owned());
                }
                _ => {}
            }
        }
        Instruction::Mult(_, src1, src2)
        | Instruction::Div(_, src1, src2)
        | Instruction::Mod(_, src1, src2)
        | Instruction::Add(_, src1, src2)
        | Instruction::Sub(_, src1, src2)
        | Instruction::LeftShift(_, src1, src2)
        | Instruction::RightShift(_, src1, src2)
        | Instruction::BitwiseAnd(_, src1, src2)
        | Instruction::BitwiseOr(_, src1, src2)
        | Instruction::BitwiseXor(_, src1, src2)
        | Instruction::LogicalAnd(_, src1, src2)
        | Instruction::LogicalOr(_, src1, src2)
        | Instruction::LessThan(_, src1, src2)
        | Instruction::GreaterThan(_, src1, src2)
        | Instruction::LessThanEq(_, src1, src2)
        | Instruction::GreaterThanEq(_, src1, src2)
        | Instruction::Equal(_, src1, src2)
        | Instruction::NotEqual(_, src1, src2)
        | Instruction::IfEqElse(src1, src2, _, _)
        | Instruction::IfNotEqElse(src1, src2, _, _) => {
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
        Instruction::Call(_, _, srcs) | Instruction::TailCall(_, srcs) => {
            for src in srcs {
                match src {
                    Src::Var(var) | Src::StoreAddressVar(var) => {
                        referenced_vars.insert(var.to_owned());
                    }
                    _ => {}
                }
            }
        }
        Instruction::Ret(src) => {
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
