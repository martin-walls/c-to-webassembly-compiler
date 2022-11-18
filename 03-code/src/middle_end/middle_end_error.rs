use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum MiddleEndError {
    BreakOutsideLoopOrSwitchContext,
    ContinueOutsideLoopContext,
    CaseOutsideSwitchContext,
    DefaultOutsideSwitchContext,
    MultipleDefaultCasesInSwitch,
    /// in theory this should never occur
    LoopNestingError,
    UndeclaredIdentifier(String),
    UndeclaredType(String),
    InvalidLValue,
    InvalidFunctionCall,
    DuplicateDeclaration(String),
    InvalidAbstractDeclarator,
    InvalidConstantExpression,
    InvalidFunctionDeclaration,
    InvalidInitialiserExpression,
    /// in theory this should never occur because of global scope
    ScopeError,
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
                write!(f, "Use of undeclared identifier: \"{}\"", name)
            }
            MiddleEndError::UndeclaredType(name) => {
                write!(f, "Use of undeclared type: \"{}\"", name)
            }
            MiddleEndError::InvalidLValue => {
                write!(f, "Invalid LValue used")
            }
            MiddleEndError::InvalidFunctionCall => {
                write!(f, "Invalid function call")
            }
            MiddleEndError::DuplicateDeclaration(name) => {
                write!(f, "Duplicate declaration: \"{}\"", name)
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
        }
    }
}

impl Error for MiddleEndError {}
