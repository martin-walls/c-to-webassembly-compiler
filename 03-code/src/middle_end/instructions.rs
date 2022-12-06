use crate::middle_end::ids::{FunId, LabelId, StringLiteralId, ValueType, VarId};
use crate::middle_end::ir::Program;
use crate::middle_end::ir_types::IrType;
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::relooper::blocks::{LoopBlockId, MultipleBlockId};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i128),
    Float(f64),
}

impl Constant {
    pub fn get_type(&self, expected: Option<Box<IrType>>) -> Box<IrType> {
        match self {
            Constant::Int(i) => {
                if let Some(t) = expected {
                    if t.is_integral_type() {
                        return t;
                    }
                }
                match i {
                    0..=255 => Box::new(IrType::U8),
                    -128..=127 => Box::new(IrType::I8),
                    0..=65535 => Box::new(IrType::U16),
                    -32_768..=32_767 => Box::new(IrType::I16),
                    0..=4_294_967_296 => Box::new(IrType::U32),
                    -2_147_483_648..=2_147_483_647 => Box::new(IrType::I32),
                    0..=18_446_744_073_709_551_615 => Box::new(IrType::U64),
                    _ => Box::new(IrType::I64),
                }
            }
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
    StoreAddressVar(VarId),
    Constant(Constant),
    Fun(FunId),
}

impl Src {
    pub fn get_type(&self, prog: &Box<Program>) -> Result<Box<IrType>, MiddleEndError> {
        match self {
            Src::Var(var) | Src::StoreAddressVar(var) => prog.get_var_type(var),
            Src::Constant(c) => Ok(c.get_type(None)),
            Src::Fun(fun_id) => prog.get_fun_type(fun_id),
        }
    }

    pub fn get_value_type(&self) -> ValueType {
        match self {
            Src::Var(var) | Src::StoreAddressVar(var) => var.get_value_type(),
            Src::Constant(_) => ValueType::RValue,
            Src::Fun(_) => ValueType::RValue,
        }
    }

    pub fn require_function_id(&self) -> Result<FunId, MiddleEndError> {
        match self {
            Src::Fun(fun_id) => Ok(fun_id.to_owned()),
            _ => Err(MiddleEndError::InvalidOperation(
                "Require function name failed",
            )),
        }
    }

    pub fn get_function_return_type(
        &self,
        prog: &Box<Program>,
    ) -> Result<Box<IrType>, MiddleEndError> {
        match self {
            Src::Fun(fun_id) => match *prog.get_fun_type(fun_id)? {
                IrType::Function(ret_type, _, _) => Ok(ret_type),
                _ => Err(MiddleEndError::UnwrapNonFunctionType),
            },
            _ => Err(MiddleEndError::UnwrapNonFunctionType),
        }
    }
}

impl fmt::Display for Src {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Src::Var(var) | Src::StoreAddressVar(var) => {
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

#[derive(Debug, Clone)]
pub enum Instruction {
    // t = a
    SimpleAssignment(Dest, Src),

    LoadFromAddress(Dest, Src),
    // addr <- x
    StoreToAddress(Dest, Src),

    AllocateVariable(Dest, Src),

    // Unary operations
    // t = <op> a
    AddressOf(Dest, Src),
    BitwiseNot(Dest, Src),
    LogicalNot(Dest, Src),
    // Binary operations
    // t = a <op> b
    Mult(Dest, Src, Src),
    Div(Dest, Src, Src),
    Mod(Dest, Src, Src),
    Add(Dest, Src, Src),
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
    Call(Dest, FunId, Vec<Src>),
    Ret(Option<Src>),
    Label(LabelId),
    Br(LabelId),
    BrIfEq(Src, Src, LabelId),
    BrIfNotEq(Src, Src, LabelId),
    PointerToStringLiteral(Dest, StringLiteralId),

    // char promotions
    I8toI16(Dest, Src),
    I8toU16(Dest, Src),
    U8toI16(Dest, Src),
    U8toU16(Dest, Src),

    // promotion to signed int
    I16toI32(Dest, Src),
    U16toI32(Dest, Src),

    // promotion to unsigned int
    I16toU32(Dest, Src),
    U16toU32(Dest, Src),
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
    // integer to double
    U32toF64(Dest, Src),
    I32toF64(Dest, Src),
    U64toF64(Dest, Src),
    I64toF64(Dest, Src),

    // float promotion
    F32toF64(Dest, Src),

    // integer truncation
    I32toI8(Dest, Src),
    U32toI8(Dest, Src),
    I64toI8(Dest, Src),
    U64toI8(Dest, Src),

    I32toU8(Dest, Src),
    U32toU8(Dest, Src),
    I64toU8(Dest, Src),
    U64toU8(Dest, Src),

    I64toI32(Dest, Src),
    U64toI32(Dest, Src),

    // cast to pointer
    U32toPtr(Dest, Src),
    I32toPtr(Dest, Src),

    Nop,

    // relooper control flow (never generated by AST to IR conversion, from relooper output only)
    Break(LoopBlockId),
    Continue(LoopBlockId),
    EndHandledBlock(MultipleBlockId),

    // if ... else ... end
    IfEqElse(Src, Src, Vec<Instruction>, Vec<Instruction>),
    IfNotEqElse(Src, Src, Vec<Instruction>, Vec<Instruction>),
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
            Instruction::LoadFromAddress(dest, src) => {
                write!(f, "{} = *{}", dest, src)
            }
            Instruction::StoreToAddress(dest, src) => {
                write!(f, "*{} <- {}", dest, src)
            }
            Instruction::AllocateVariable(dest, size) => {
                write!(f, "allocate {} bytes for {}", size, dest)
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
            Instruction::PointerToStringLiteral(dest, string_id) => {
                write!(f, "{} = pointer to string literal {}", dest, string_id)
            }
            Instruction::I8toI16(dest, src) | Instruction::U8toI16(dest, src) => {
                write!(f, "{} = (I16) {}", dest, src)
            }
            Instruction::I8toU16(dest, src) | Instruction::U8toU16(dest, src) => {
                write!(f, "{} = (U16) {}", dest, src)
            }
            Instruction::I16toI32(dest, src) | Instruction::U16toI32(dest, src) => {
                write!(f, "{} = (I32) {}", dest, src)
            }
            Instruction::I16toU32(dest, src)
            | Instruction::U16toU32(dest, src)
            | Instruction::I32toU32(dest, src) => {
                write!(f, "{} = (U32) {}", dest, src)
            }
            Instruction::I32toI64(dest, src) | Instruction::U32toI64(dest, src) => {
                write!(f, "{} = (I64) {}", dest, src)
            }
            Instruction::I32toU64(dest, src)
            | Instruction::U32toU64(dest, src)
            | Instruction::I64toU64(dest, src) => {
                write!(f, "{} = (U64) {}", dest, src)
            }
            Instruction::U32toF32(dest, src)
            | Instruction::I32toF32(dest, src)
            | Instruction::U64toF32(dest, src)
            | Instruction::I64toF32(dest, src) => {
                write!(f, "{} = (F32) {}", dest, src)
            }
            Instruction::U32toF64(dest, src)
            | Instruction::I32toF64(dest, src)
            | Instruction::U64toF64(dest, src)
            | Instruction::I64toF64(dest, src)
            | Instruction::F32toF64(dest, src) => {
                write!(f, "{} = (F64) {}", dest, src)
            }
            Instruction::Nop => {
                write!(f, "Nop")
            }
            Instruction::Break(loop_block_id) => {
                write!(f, "break {}", loop_block_id)
            }
            Instruction::Continue(loop_block_id) => {
                write!(f, "continue {}", loop_block_id)
            }
            Instruction::EndHandledBlock(multiple_block_id) => {
                write!(f, "endHandled {}", multiple_block_id)
            }
            Instruction::IfEqElse(src1, src2, if_block, else_block) => {
                write!(f, "if {} == {} {{ ", src1, src2)?;
                for instr in if_block {
                    write!(f, "{}; ", instr)?;
                }
                write!(f, "}} else {{ ")?;
                for instr in else_block {
                    write!(f, "{}; ", instr)?;
                }
                write!(f, "}}")
            }
            Instruction::IfNotEqElse(src1, src2, if_block, else_block) => {
                write!(f, "if {} != {} {{ ", src1, src2)?;
                for instr in if_block {
                    write!(f, "{}; ", instr)?;
                }
                write!(f, "}} else {{ ")?;
                for instr in else_block {
                    write!(f, "{}; ", instr)?;
                }
                write!(f, "}}")
            }
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}
