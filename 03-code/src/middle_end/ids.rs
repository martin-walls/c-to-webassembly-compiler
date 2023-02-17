use std::fmt;
use std::fmt::Formatter;

use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstructionId(u64);

impl Id for InstructionId {
    fn initial_id() -> Self {
        InstructionId(0)
    }

    fn next_id(&self) -> Self {
        InstructionId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for InstructionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    LValue,
    RValue,
    None,
}

impl ValueType {
    pub fn is_lvalue(&self) -> bool {
        match self {
            ValueType::LValue => true,
            ValueType::RValue | ValueType::None => false,
        }
    }

    pub fn is_rvalue(&self) -> bool {
        match self {
            ValueType::RValue => true,
            ValueType::LValue | ValueType::None => false,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::LValue => {
                write!(f, "lvalue")
            }
            ValueType::RValue => {
                write!(f, "rvalue")
            }
            ValueType::None => {
                write!(f, "no value type")
            }
        }
    }
}

/// A variable in the IR
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarId(u64, ValueType);

impl Id for VarId {
    fn initial_id() -> Self {
        VarId(0, ValueType::None)
    }

    fn next_id(&self) -> Self {
        VarId(self.0 + 1, self.1.to_owned())
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl VarId {
    pub fn set_value_type(&mut self, value_type: ValueType) {
        self.1 = value_type;
    }

    pub fn get_value_type(&self) -> ValueType {
        self.1.to_owned()
    }
}

impl fmt::Display for VarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunId(u64);

impl Id for FunId {
    fn initial_id() -> Self {
        FunId(0)
    }

    fn next_id(&self) -> Self {
        FunId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for FunId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "f{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelId(u64);

impl Id for LabelId {
    fn initial_id() -> Self {
        LabelId(0)
    }

    fn next_id(&self) -> Self {
        LabelId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for LabelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "l{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringLiteralId(u64);

impl Id for StringLiteralId {
    fn initial_id() -> Self {
        StringLiteralId(0)
    }

    fn next_id(&self) -> Self {
        StringLiteralId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for StringLiteralId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "s{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructId(u64);

impl Id for StructId {
    fn initial_id() -> Self {
        StructId(0)
    }

    fn next_id(&self) -> Self {
        StructId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for StructId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "struct{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionId(u64);

impl Id for UnionId {
    fn initial_id() -> Self {
        UnionId(0)
    }

    fn next_id(&self) -> Self {
        UnionId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for UnionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "union{}", self.0)
    }
}
