use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;

#[derive(Debug)]
pub enum ValType {
    NumType(NumType),
    RefType(RefType),
}

impl ToBytes for ValType {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ValType::NumType(t) => t.to_bytes(),
            ValType::RefType(t) => t.to_bytes(),
        }
    }
}

#[derive(Debug)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

impl ToBytes for NumType {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            NumType::I32 => {
                vec![0x7f]
            }
            NumType::I64 => {
                vec![0x7e]
            }
            NumType::F32 => {
                vec![0x7d]
            }
            NumType::F64 => {
                vec![0x7c]
            }
        }
    }
}

#[derive(Debug)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl ToBytes for RefType {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            RefType::FuncRef => {
                vec![0x70]
            }
            RefType::ExternRef => {
                vec![0x6f]
            }
        }
    }
}

pub struct TableType {
    pub element_ref_type: RefType,
    pub limits: Limits,
}

impl ToBytes for TableType {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.element_ref_type.to_bytes();
        bytes.append(&mut self.limits.to_bytes());
        bytes
    }
}

pub struct MemoryType {
    /// Memory limits in units of page size
    pub limits: Limits,
}

impl ToBytes for MemoryType {
    fn to_bytes(&self) -> Vec<u8> {
        self.limits.to_bytes()
    }
}

pub struct GlobalType {
    pub value_type: ValType,
    pub is_mutable: bool,
}

impl ToBytes for GlobalType {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.value_type.to_bytes();
        match self.is_mutable {
            false => bytes.push(0x00),
            true => bytes.push(0x01),
        }
        bytes
    }
}

pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl ToBytes for Limits {
    fn to_bytes(&self) -> Vec<u8> {
        match self.max {
            None => {
                let mut bytes = vec![0x00];
                bytes.append(&mut encode_unsigned_int(self.min as u128));
                bytes
            }
            Some(max) => {
                let mut bytes = vec![0x01];
                bytes.append(&mut encode_unsigned_int(self.min as u128));
                bytes.append(&mut encode_unsigned_int(max as u128));
                bytes
            }
        }
    }
}
