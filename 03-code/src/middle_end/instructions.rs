use crate::middle_end::ids::{FunId, LabelId, StringLiteralId, VarId};
use crate::middle_end::ir::Program;
use crate::middle_end::ir_types::IrType;
use crate::middle_end::middle_end_error::{MiddleEndError, TypeError};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i128),
    Float(f64),
}

impl Constant {
    pub fn get_type(&self) -> Box<IrType> {
        match self {
            Constant::Int(i) => match i {
                0..=255 => Box::new(IrType::U8),
                -128..=127 => Box::new(IrType::I8),
                0..=65535 => Box::new(IrType::U16),
                -32_768..=32_767 => Box::new(IrType::I16),
                0..=4_294_967_296 => Box::new(IrType::U32),
                -2_147_483_648..=2_147_483_647 => Box::new(IrType::I32),
                0..=18_446_744_073_709_551_615 => Box::new(IrType::U64),
                _ => Box::new(IrType::I64),
            },
            Constant::Float(_) => Box::new(IrType::F64),
        }
    }
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

impl Src {
    pub fn get_type(&self, prog: &Box<Program>) -> Result<Box<IrType>, MiddleEndError> {
        match self {
            Src::Var(var) => prog.get_var_type(var),
            Src::Constant(c) => Ok(c.get_type()),
            Src::Fun(fun_id) => prog.get_fun_type(fun_id),
        }
    }
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
    Call(Dest, Src, Vec<Src>), // probably will use call_indirect in wasm to call the function
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

    // integer promotion
    I8toI32(Dest, Src),
    U8toI32(Dest, Src),
    I16toI32(Dest, Src),
    U16toI32(Dest, Src),

    I32toU32(Dest, Src),

    // promotion to unsigned long
    I32toU64(Dest, Src),
    U32toU64(Dest, Src),
    I64toU64(Dest, Src),

    // promotion to long
    I32toI64(Dest, Src),
    U32toI64(Dest, Src),

    // integer to float
    U32toF32(Dest, Src),
    I32toF32(Dest, Src),
    U64toF32(Dest, Src),
    I64toF32(Dest, Src),
    U32toF64(Dest, Src),
    I32toF64(Dest, Src),
    U64toF64(Dest, Src),
    I64toF64(Dest, Src),

    // float promotion
    F32toF64(Dest, Src),

    Nop,
}

impl Instruction {
    pub fn get_conversion_instr(
        src: Src,
        src_type: Box<IrType>,
        dest: Dest,
        dest_type: Box<IrType>,
    ) -> Result<Self, MiddleEndError> {
        if src_type == dest_type {
            return Ok(Instruction::Nop);
        }
        match (*src_type, *dest_type) {
            (IrType::I8, IrType::I32) => Ok(Instruction::I8toI32(dest, src)),
            (IrType::U8, IrType::I32) => Ok(Instruction::U8toI32(dest, src)),
            (IrType::I16, IrType::I32) => Ok(Instruction::I16toI32(dest, src)),
            (IrType::U16, IrType::I32) => Ok(Instruction::U16toI32(dest, src)),
            (IrType::Function(_, _), IrType::PointerTo(_)) => Ok(Instruction::AddressOf(dest, src)),
            (IrType::ArrayOf(_, _), IrType::PointerTo(_)) => Ok(Instruction::AddressOf(dest, src)),
            (s, d) => Err(MiddleEndError::TypeError(TypeError::TypeConversionError(
                "Cannot convert type",
                Box::new(s),
                Some(Box::new(d)),
            ))),
        }
    }
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
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}
