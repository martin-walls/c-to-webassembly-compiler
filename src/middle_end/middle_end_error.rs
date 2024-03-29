use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

use crate::middle_end::ids::{FunId, VarId};
use crate::middle_end::ir_types::IrType;

#[derive(Debug)]
pub enum MiddleEndError {
    BreakOutsideLoopOrSwitchContext,
    ContinueOutsideLoopContext,
    CaseOutsideSwitchContext,
    DefaultOutsideSwitchContext,
    MultipleDefaultCasesInSwitch,
    NoCaseBlockToPushInstructionTo,
    /// in theory this should never occur
    LoopNestingError,
    UndeclaredIdentifier(String),
    UndeclaredType(String),
    UndeclaredEnumTag(String),
    UndeclaredStructTag(String),
    UndeclaredUnionTag(String),
    InvalidLValue,
    DuplicateFunctionDeclaration(String),
    DuplicateTypeDeclaration(String),
    DuplicateEnumConstantDeclaration(String),
    InvalidDeclaration,
    InvalidAbstractDeclarator,
    InvalidConstantExpression,
    InvalidFunctionDeclaration,
    InvalidTypedefDeclaration,
    InvalidInitialiserExpression,
    UnnamedStructMember,
    DuplicateStructMember,
    StructMemberNotFound(String),
    UnnamedUnionMember,
    DuplicateUnionMember,
    UnionMemberNotFound(String),
    /// in theory this should never occur because of global scope
    ScopeError,
    /// in theory shouldn't happen
    RedeclaredVarType(VarId),
    /// in theory shouldn't happen
    TypeNotFound,
    FunctionNotFoundForId(FunId),
    InvalidAssignment,
    AttemptToModifyNonLValue,
    UndefinedArraySize,
    ArrayMemberSizeNotKnownAtCompileTime,
    UndefinedStructMemberSize,
    UndefinedUnionMemberSize,
    CantEvaluateAtCompileTime,
    UnwrapNonFunctionType,
    DereferenceNonPointerType(IrType),
    InvalidOperation(&'static str),
    TypeConversionError(&'static str, IrType, Option<IrType>),
    UnwrapNonArrayType(IrType),
    UnwrapNonStructType(IrType),
    AssignNonAggregateValueToAggregateType,
    MismatchedArrayInitialiserLength,
    ByteSizeNotKnownAtCompileTime,
    AttemptToStoreToNonVariable,
    NoMainFunctionDefined,
    UnwrapNonVarSrc,
}

impl fmt::Display for MiddleEndError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MiddleEndError::BreakOutsideLoopOrSwitchContext => {
                write!(
                    f,
                    "Illegal break statement found outside loop or switch context"
                )
            }
            MiddleEndError::ContinueOutsideLoopContext => {
                write!(f, "Illegal continue statement found outside loop context")
            }
            MiddleEndError::CaseOutsideSwitchContext => {
                write!(f, "Illegal case statement found outside switch context")
            }
            MiddleEndError::DefaultOutsideSwitchContext => {
                write!(f, "Illegal default case statement outside switch context")
            }
            MiddleEndError::MultipleDefaultCasesInSwitch => {
                write!(
                    f,
                    "Multiple default case statements found inside switch context"
                )
            }
            MiddleEndError::LoopNestingError => {
                write!(f, "Loop nesting error when converting to IR")
            }
            MiddleEndError::UndeclaredIdentifier(name) => {
                write!(f, "Use of undeclared identifier: \"{name}\"")
            }
            MiddleEndError::UndeclaredType(name) => {
                write!(f, "Use of undeclared type: \"{name}\"")
            }
            MiddleEndError::InvalidLValue => {
                write!(f, "Invalid LValue used")
            }
            MiddleEndError::InvalidAbstractDeclarator => {
                write!(f, "Invalid abstract declarator")
            }
            MiddleEndError::InvalidConstantExpression => {
                write!(f, "Invalid constant expression")
            }
            MiddleEndError::InvalidFunctionDeclaration => {
                write!(f, "Invalid function declaration")
            }
            MiddleEndError::ScopeError => {
                write!(f, "Scoping error")
            }
            MiddleEndError::InvalidInitialiserExpression => {
                write!(f, "Invalid initialiser expression")
            }
            MiddleEndError::DuplicateTypeDeclaration(t) => {
                write!(f, "Duplicate typedef declaration: \"{t}\"")
            }
            MiddleEndError::InvalidTypedefDeclaration => {
                write!(f, "Invalid typedef declaration")
            }
            MiddleEndError::DuplicateFunctionDeclaration(name) => {
                write!(f, "Duplicate function declaration: \"{name}\"")
            }
            MiddleEndError::InvalidDeclaration => {
                write!(f, "Invalid declaration")
            }
            MiddleEndError::UnnamedStructMember => {
                write!(f, "Unnamed struct members are not allowed")
            }
            MiddleEndError::DuplicateStructMember => {
                write!(f, "Duplicate struct member")
            }
            MiddleEndError::RedeclaredVarType(var) => {
                write!(f, "Type for {var} was declared twice in IR")
            }
            MiddleEndError::TypeNotFound => {
                write!(f, "Type was not found in IR")
            }
            e => {
                write!(f, "Middle end error: {e:?}")
            }
        }
    }
}

impl Error for MiddleEndError {}
