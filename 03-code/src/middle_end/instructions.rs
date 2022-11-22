use crate::middle_end::ids::{FunId, LabelId, StringLiteralId, VarId};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i128),
    Float(f64),
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Int(i) => {
                write!(f, "{}", i)
            }
            Constant::Float(fl) => {
                write!(f, "{}", fl)
            }
        }
    }
}

pub type Dest = VarId;

#[derive(Debug, Clone)]
pub enum Src {
    Var(VarId),
    Constant(Constant),
    Fun(FunId),
}

impl fmt::Display for Src {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Src::Var(var) => {
                write!(f, "{}", var)
            }
            Src::Constant(c) => {
                write!(f, "{}", c)
            }
            Src::Fun(fun) => {
                write!(f, "{}", fun)
            }
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    // t = a
    SimpleAssignment(Dest, Src),
    // Unary operations
    // t = <op> a
    AddressOf(Dest, Src),
    Dereference(Dest, Src),
    BitwiseNot(Dest, Src),
    LogicalNot(Dest, Src),
    // Binary operations
    // t = a <op> b
    Mult(Dest, Src, Src),
    Div(Dest, Src, Src),
    Mod(Dest, Src, Src),
    Add(Dest, Src, Src), // todo if adding to a pointer, add by the size of the object it points to
    Sub(Dest, Src, Src),
    LeftShift(Dest, Src, Src),
    RightShift(Dest, Src, Src),
    BitwiseAnd(Dest, Src, Src),
    BitwiseOr(Dest, Src, Src),
    BitwiseXor(Dest, Src, Src),
    LogicalAnd(Dest, Src, Src),
    LogicalOr(Dest, Src, Src),

    // comparison
    LessThan(Dest, Src, Src),
    GreaterThan(Dest, Src, Src),
    LessThanEq(Dest, Src, Src),
    GreaterThanEq(Dest, Src, Src),
    Equal(Dest, Src, Src),
    NotEqual(Dest, Src, Src),

    // control flow
    Call(Dest, Src, Vec<Src>),
    Ret(Option<Src>),
    Label(LabelId),
    Br(LabelId),
    BrIfEq(Src, Src, LabelId),
    BrIfNotEq(Src, Src, LabelId),
    BrIfGT(Src, Src, LabelId),
    BrIfLT(Src, Src, LabelId),
    BrIfGE(Src, Src, LabelId),
    BrIfLE(Src, Src, LabelId),

    PointerToStringLiteral(Dest, StringLiteralId),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::SimpleAssignment(dest, src) => {
                write!(f, "{} = {}", dest, src)
            }
            Instruction::AddressOf(dest, src) => {
                write!(f, "{} = &{}", dest, src)
            }
            Instruction::Dereference(dest, src) => {
                write!(f, "{} = *{}", dest, src)
            }
            Instruction::BitwiseNot(dest, src) => {
                write!(f, "{} = ~{}", dest, src)
            }
            Instruction::LogicalNot(dest, src) => {
                write!(f, "{} = !{}", dest, src)
            }
            Instruction::Mult(dest, left, right) => {
                write!(f, "{} = {} * {}", dest, left, right)
            }
            Instruction::Div(dest, left, right) => {
                write!(f, "{} = {} / {}", dest, left, right)
            }
            Instruction::Mod(dest, left, right) => {
                write!(f, "{} = {} % {}", dest, left, right)
            }
            Instruction::Add(dest, left, right) => {
                write!(f, "{} = {} + {}", dest, left, right)
            }
            Instruction::Sub(dest, left, right) => {
                write!(f, "{} = {} - {}", dest, left, right)
            }
            Instruction::LeftShift(dest, left, right) => {
                write!(f, "{} = {} << {}", dest, left, right)
            }
            Instruction::RightShift(dest, left, right) => {
                write!(f, "{} = {} >> {}", dest, left, right)
            }
            Instruction::BitwiseAnd(dest, left, right) => {
                write!(f, "{} = {} & {}", dest, left, right)
            }
            Instruction::BitwiseOr(dest, left, right) => {
                write!(f, "{} = {} | {}", dest, left, right)
            }
            Instruction::BitwiseXor(dest, left, right) => {
                write!(f, "{} = {} ^ {}", dest, left, right)
            }
            Instruction::LogicalAnd(dest, left, right) => {
                write!(f, "{} = {} && {}", dest, left, right)
            }
            Instruction::LogicalOr(dest, left, right) => {
                write!(f, "{} = {} || {}", dest, left, right)
            }
            Instruction::LessThan(dest, left, right) => {
                write!(f, "{} = {} < {}", dest, left, right)
            }
            Instruction::GreaterThan(dest, left, right) => {
                write!(f, "{} = {} > {}", dest, left, right)
            }
            Instruction::LessThanEq(dest, left, right) => {
                write!(f, "{} = {} <= {}", dest, left, right)
            }
            Instruction::GreaterThanEq(dest, left, right) => {
                write!(f, "{} = {} >= {}", dest, left, right)
            }
            Instruction::Equal(dest, left, right) => {
                write!(f, "{} = {} == {}", dest, left, right)
            }
            Instruction::NotEqual(dest, left, right) => {
                write!(f, "{} = {} != {}", dest, left, right)
            }
            Instruction::Call(dest, fun, params) => {
                write!(f, "{} = call {}(", dest, fun)?;
                for param in &params[..params.len() - 1] {
                    write!(f, "{}, ", param)?;
                }
                write!(f, "{})", params[params.len() - 1])
            }
            Instruction::Ret(src) => match src {
                None => {
                    write!(f, "return")
                }
                Some(src) => {
                    write!(f, "return {}", src)
                }
            },
            Instruction::Label(label) => {
                write!(f, "{}:", label)
            }
            Instruction::Br(label) => {
                write!(f, "goto {}", label)
            }
            Instruction::BrIfEq(left, right, label) => {
                write!(f, "if {} == {} goto {}", left, right, label)
            }
            Instruction::BrIfNotEq(left, right, label) => {
                write!(f, "if {} != {} goto {}", left, right, label)
            }
            Instruction::BrIfGT(left, right, label) => {
                write!(f, "if {} > {} goto {}", left, right, label)
            }
            Instruction::BrIfLT(left, right, label) => {
                write!(f, "if {} < {} goto {}", left, right, label)
            }
            Instruction::BrIfGE(left, right, label) => {
                write!(f, "if {} >= {} goto {}", left, right, label)
            }
            Instruction::BrIfLE(left, right, label) => {
                write!(f, "if {} <= {} goto {}", left, right, label)
            }
            Instruction::PointerToStringLiteral(dest, string_id) => {
                write!(f, "{} = pointer to string literal {}", dest, string_id)
            }
        }
    }
}
