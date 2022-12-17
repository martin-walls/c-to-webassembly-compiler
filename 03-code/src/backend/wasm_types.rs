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
