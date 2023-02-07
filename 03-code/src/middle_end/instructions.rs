use std::fmt;
use std::fmt::Formatter;

use crate::middle_end::ids::{FunId, InstructionId, LabelId, StringLiteralId, ValueType, VarId};
use crate::middle_end::ir::{Program, ProgramMetadata};
use crate::middle_end::ir_types::IrType;
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::relooper::blocks::{LoopBlockId, MultipleBlockId};

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
                    0..=65_535 => Box::new(IrType::U16),
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

    pub fn get_type_minimum_i32(&self) -> Box<IrType> {
        match self {
            Constant::Int(i) => match i {
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
    StoreAddressVar(VarId),
    Constant(Constant),
    Fun(FunId),
}

impl Src {
    pub fn get_type(
        &self,
        prog_metadata: &Box<ProgramMetadata>,
    ) -> Result<Box<IrType>, MiddleEndError> {
        match self {
            Src::Var(var) | Src::StoreAddressVar(var) => prog_metadata.get_var_type(var),
            Src::Constant(c) => Ok(c.get_type(None)),
            Src::Fun(fun_id) => prog_metadata.get_fun_type(fun_id),
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

    pub fn unwrap_var(&self) -> Result<VarId, MiddleEndError> {
        match self {
            Src::Var(var_id) | Src::StoreAddressVar(var_id) => Ok(var_id.to_owned()),
            _ => Err(MiddleEndError::UnwrapNonVarSrc),
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
    SimpleAssignment(InstructionId, Dest, Src),

    LoadFromAddress(InstructionId, Dest, Src),
    // addr <- x
    StoreToAddress(InstructionId, Dest, Src),

    DeclareVariable(InstructionId, Dest),
    AllocateVariable(InstructionId, Dest, Src),

    // Unary operations
    // t = <op> a
    AddressOf(InstructionId, Dest, Src),
    BitwiseNot(InstructionId, Dest, Src),
    LogicalNot(InstructionId, Dest, Src),
    // Binary operations
    // t = a <op> b
    Mult(InstructionId, Dest, Src, Src),
    Div(InstructionId, Dest, Src, Src),
    Mod(InstructionId, Dest, Src, Src),
    Add(InstructionId, Dest, Src, Src),
    Sub(InstructionId, Dest, Src, Src),
    LeftShift(InstructionId, Dest, Src, Src),
    RightShift(InstructionId, Dest, Src, Src),
    BitwiseAnd(InstructionId, Dest, Src, Src),
    BitwiseOr(InstructionId, Dest, Src, Src),
    BitwiseXor(InstructionId, Dest, Src, Src),
    LogicalAnd(InstructionId, Dest, Src, Src),
    LogicalOr(InstructionId, Dest, Src, Src),

    // comparison
    LessThan(InstructionId, Dest, Src, Src),
    GreaterThan(InstructionId, Dest, Src, Src),
    LessThanEq(InstructionId, Dest, Src, Src),
    GreaterThanEq(InstructionId, Dest, Src, Src),
    Equal(InstructionId, Dest, Src, Src),
    NotEqual(InstructionId, Dest, Src, Src),

    // control flow
    Call(InstructionId, Dest, FunId, Vec<Src>),
    TailCall(InstructionId, FunId, Vec<Src>),
    Ret(InstructionId, Option<Src>),
    Label(InstructionId, LabelId),
    Br(InstructionId, LabelId),
    BrIfEq(InstructionId, Src, Src, LabelId),
    BrIfNotEq(InstructionId, Src, Src, LabelId),
    PointerToStringLiteral(InstructionId, Dest, StringLiteralId),

    // char promotions
    I8toI16(InstructionId, Dest, Src),
    I8toU16(InstructionId, Dest, Src),
    U8toI16(InstructionId, Dest, Src),
    U8toU16(InstructionId, Dest, Src),

    // promotion to signed int
    I16toI32(InstructionId, Dest, Src),
    U16toI32(InstructionId, Dest, Src),

    // promotion to unsigned int
    I16toU32(InstructionId, Dest, Src),
    U16toU32(InstructionId, Dest, Src),
    I32toU32(InstructionId, Dest, Src),

    // promotion to unsigned long
    I32toU64(InstructionId, Dest, Src),
    U32toU64(InstructionId, Dest, Src),
    I64toU64(InstructionId, Dest, Src),

    // promotion to long
    I32toI64(InstructionId, Dest, Src),
    U32toI64(InstructionId, Dest, Src),

    // integer to float
    U32toF32(InstructionId, Dest, Src),
    I32toF32(InstructionId, Dest, Src),
    U64toF32(InstructionId, Dest, Src),
    I64toF32(InstructionId, Dest, Src),
    // integer to double
    U32toF64(InstructionId, Dest, Src),
    I32toF64(InstructionId, Dest, Src),
    U64toF64(InstructionId, Dest, Src),
    I64toF64(InstructionId, Dest, Src),

    // float promotion
    F32toF64(InstructionId, Dest, Src),

    // double to int
    F64toI32(InstructionId, Dest, Src),

    // integer truncation
    I32toI8(InstructionId, Dest, Src),
    U32toI8(InstructionId, Dest, Src),
    I64toI8(InstructionId, Dest, Src),
    U64toI8(InstructionId, Dest, Src),

    I32toU8(InstructionId, Dest, Src),
    U32toU8(InstructionId, Dest, Src),
    I64toU8(InstructionId, Dest, Src),
    U64toU8(InstructionId, Dest, Src),

    I64toI32(InstructionId, Dest, Src),
    U64toI32(InstructionId, Dest, Src),

    // cast to pointer
    U32toPtr(InstructionId, Dest, Src),
    I32toPtr(InstructionId, Dest, Src),
    PtrToI32(InstructionId, Dest, Src),

    Nop(InstructionId),

    // relooper control flow (never generated by AST to IR conversion, from relooper output only)
    Break(InstructionId, LoopBlockId),
    Continue(InstructionId, LoopBlockId),
    EndHandledBlock(InstructionId, MultipleBlockId),

    // if ... else ... end
    IfEqElse(InstructionId, Src, Src, Vec<Instruction>, Vec<Instruction>),
    IfNotEqElse(InstructionId, Src, Src, Vec<Instruction>, Vec<Instruction>),
}

impl Instruction {
    pub fn has_side_effect(&self) -> bool {
        match self {
            Instruction::Call(..) | Instruction::TailCall(..) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::SimpleAssignment(id, dest, src) => {
                write!(f, "[{}] {} = {}", id, dest, src)
            }
            Instruction::AddressOf(id, dest, src) => {
                write!(f, "[{}] {} = &{}", id, dest, src)
            }
            Instruction::LoadFromAddress(id, dest, src) => {
                write!(f, "[{}] {} = *{}", id, dest, src)
            }
            Instruction::StoreToAddress(id, dest, src) => {
                write!(f, "[{}] *{} <- {}", id, dest, src)
            }
            Instruction::AllocateVariable(id, dest, size) => {
                write!(f, "[{}] allocate {} bytes for {}", id, size, dest)
            }
            Instruction::BitwiseNot(id, dest, src) => {
                write!(f, "[{}] {} = ~{}", id, dest, src)
            }
            Instruction::LogicalNot(id, dest, src) => {
                write!(f, "[{}] {} = !{}", id, dest, src)
            }
            Instruction::Mult(id, dest, left, right) => {
                write!(f, "[{}] {} = {} * {}", id, dest, left, right)
            }
            Instruction::Div(id, dest, left, right) => {
                write!(f, "[{}] {} = {} / {}", id, dest, left, right)
            }
            Instruction::Mod(id, dest, left, right) => {
                write!(f, "[{}] {} = {} % {}", id, dest, left, right)
            }
            Instruction::Add(id, dest, left, right) => {
                write!(f, "[{}] {} = {} + {}", id, dest, left, right)
            }
            Instruction::Sub(id, dest, left, right) => {
                write!(f, "[{}] {} = {} - {}", id, dest, left, right)
            }
            Instruction::LeftShift(id, dest, left, right) => {
                write!(f, "[{}] {} = {} << {}", id, dest, left, right)
            }
            Instruction::RightShift(id, dest, left, right) => {
                write!(f, "[{}] {} = {} >> {}", id, dest, left, right)
            }
            Instruction::BitwiseAnd(id, dest, left, right) => {
                write!(f, "[{}] {} = {} & {}", id, dest, left, right)
            }
            Instruction::BitwiseOr(id, dest, left, right) => {
                write!(f, "[{}] {} = {} | {}", id, dest, left, right)
            }
            Instruction::BitwiseXor(id, dest, left, right) => {
                write!(f, "[{}] {} = {} ^ {}", id, dest, left, right)
            }
            Instruction::LogicalAnd(id, dest, left, right) => {
                write!(f, "[{}] {} = {} && {}", id, dest, left, right)
            }
            Instruction::LogicalOr(id, dest, left, right) => {
                write!(f, "[{}] {} = {} || {}", id, dest, left, right)
            }
            Instruction::LessThan(id, dest, left, right) => {
                write!(f, "[{}] {} = {} < {}", id, dest, left, right)
            }
            Instruction::GreaterThan(id, dest, left, right) => {
                write!(f, "[{}] {} = {} > {}", id, dest, left, right)
            }
            Instruction::LessThanEq(id, dest, left, right) => {
                write!(f, "[{}] {} = {} <= {}", id, dest, left, right)
            }
            Instruction::GreaterThanEq(id, dest, left, right) => {
                write!(f, "[{}] {} = {} >= {}", id, dest, left, right)
            }
            Instruction::Equal(id, dest, left, right) => {
                write!(f, "[{}] {} = {} == {}", id, dest, left, right)
            }
            Instruction::NotEqual(id, dest, left, right) => {
                write!(f, "[{}] {} = {} != {}", id, dest, left, right)
            }
            Instruction::Call(id, dest, fun, params) => {
                write!(f, "[{}] {} = call {}(", id, dest, fun)?;
                if !params.is_empty() {
                    for param in &params[..params.len() - 1] {
                        write!(f, "{}, ", param)?;
                    }
                    write!(f, "{}", params[params.len() - 1])?;
                }
                write!(f, ")")
            }
            Instruction::TailCall(id, fun, params) => {
                write!(f, "[{}] tail-call {}(", id, fun)?;
                if !params.is_empty() {
                    for param in &params[..params.len() - 1] {
                        write!(f, "{}, ", param)?;
                    }
                    write!(f, "{}", params[params.len() - 1])?;
                }
                write!(f, ")")
            }
            Instruction::Ret(id, src) => match src {
                None => {
                    write!(f, "[{}] return", id)
                }
                Some(src) => {
                    write!(f, "[{}] return {}", id, src)
                }
            },
            Instruction::Label(id, label) => {
                write!(f, "[{}] {}:", id, label)
            }
            Instruction::Br(id, label) => {
                write!(f, "[{}] goto {}", id, label)
            }
            Instruction::BrIfEq(id, left, right, label) => {
                write!(f, "[{}] if {} == {} goto {}", id, left, right, label)
            }
            Instruction::BrIfNotEq(id, left, right, label) => {
                write!(f, "[{}] if {} != {} goto {}", id, left, right, label)
            }
            Instruction::PointerToStringLiteral(id, dest, string_id) => {
                write!(
                    f,
                    "[{}] {} = pointer to string literal {}",
                    id, dest, string_id
                )
            }
            Instruction::I8toI16(id, dest, src) | Instruction::U8toI16(id, dest, src) => {
                write!(f, "[{}] {} = (I16) {}", id, dest, src)
            }
            Instruction::I8toU16(id, dest, src) | Instruction::U8toU16(id, dest, src) => {
                write!(f, "[{}] {} = (U16) {}", id, dest, src)
            }
            Instruction::I16toI32(id, dest, src) | Instruction::U16toI32(id, dest, src) => {
                write!(f, "[{}] {} = (I32) {}", id, dest, src)
            }
            Instruction::I16toU32(id, dest, src)
            | Instruction::U16toU32(id, dest, src)
            | Instruction::I32toU32(id, dest, src) => {
                write!(f, "[{}] {} = (U32) {}", id, dest, src)
            }
            Instruction::I32toI64(id, dest, src) | Instruction::U32toI64(id, dest, src) => {
                write!(f, "[{}] {} = (I64) {}", id, dest, src)
            }
            Instruction::I32toU64(id, dest, src)
            | Instruction::U32toU64(id, dest, src)
            | Instruction::I64toU64(id, dest, src) => {
                write!(f, "[{}] {} = (U64) {}", id, dest, src)
            }
            Instruction::U32toF32(id, dest, src)
            | Instruction::I32toF32(id, dest, src)
            | Instruction::U64toF32(id, dest, src)
            | Instruction::I64toF32(id, dest, src) => {
                write!(f, "[{}] {} = (F32) {}", id, dest, src)
            }
            Instruction::U32toF64(id, dest, src)
            | Instruction::I32toF64(id, dest, src)
            | Instruction::U64toF64(id, dest, src)
            | Instruction::I64toF64(id, dest, src)
            | Instruction::F32toF64(id, dest, src) => {
                write!(f, "[{}] {} = (F64) {}", id, dest, src)
            }
            Instruction::Nop(id) => {
                write!(f, "[{}] Nop", id)
            }
            Instruction::Break(id, loop_block_id) => {
                write!(f, "[{}] break {}", id, loop_block_id)
            }
            Instruction::Continue(id, loop_block_id) => {
                write!(f, "[{}] continue {}", id, loop_block_id)
            }
            Instruction::EndHandledBlock(id, multiple_block_id) => {
                write!(f, "[{}] endHandled {}", id, multiple_block_id)
            }
            Instruction::IfEqElse(id, src1, src2, if_block, else_block) => {
                write!(f, "[{}] if {} == {} {{ ", id, src1, src2)?;
                for instr in if_block {
                    write!(f, "{}; ", instr)?;
                }
                write!(f, "}} else {{ ")?;
                for instr in else_block {
                    write!(f, "{}; ", instr)?;
                }
                write!(f, "}}")
            }
            Instruction::IfNotEqElse(id, src1, src2, if_block, else_block) => {
                write!(f, "[{}] if {} != {} {{ ", id, src1, src2)?;
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
