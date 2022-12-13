use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::wasm_instructions::ToBytes;

pub struct TypeIdx {
    x: u32,
}

impl ToBytes for TypeIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct TableIdx {
    x: u32,
}

impl ToBytes for TableIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct ElemIdx {
    x: u32,
}

impl ToBytes for ElemIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct DataIdx {
    x: u32,
}

impl ToBytes for DataIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct FuncIdx {
    x: u32,
}

impl ToBytes for FuncIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct LocalIdx {
    x: u32,
}

impl ToBytes for LocalIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct GlobalIdx {
    x: u32,
}

impl ToBytes for GlobalIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

pub struct LabelIdx {
    l: u32,
}

impl ToBytes for LabelIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.l as u128)
    }
}
