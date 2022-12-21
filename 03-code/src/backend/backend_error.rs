use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum BackendError {
    NoMainFunctionDefined,
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::NoMainFunctionDefined => {
                write!(f, "Program must define a \"main\" function")
            }
            e => write!(f, "Backend error: {:?}", e),
        }
    }
}

impl Error for BackendError {}
