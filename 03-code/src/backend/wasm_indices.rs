use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;

pub trait WasmIdx {
    fn initial_idx() -> Self;
    fn next_idx(&self) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeIdx {
    x: u32,
}

impl WasmIdx for TypeIdx {
    fn initial_idx() -> Self {
        TypeIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        TypeIdx { x: self.x + 1 }
    }
}

impl ToBytes for TypeIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableIdx {
    x: u32,
}

impl WasmIdx for TableIdx {
    fn initial_idx() -> Self {
        TableIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        TableIdx { x: self.x + 1 }
    }
}

impl ToBytes for TableIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemIdx {
    x: u32,
}

impl WasmIdx for MemIdx {
    fn initial_idx() -> Self {
        MemIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        MemIdx { x: self.x + 1 }
    }
}

impl ToBytes for MemIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElemIdx {
    x: u32,
}

impl WasmIdx for ElemIdx {
    fn initial_idx() -> Self {
        ElemIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        ElemIdx { x: self.x + 1 }
    }
}

impl ToBytes for ElemIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataIdx {
    x: u32,
}

impl WasmIdx for DataIdx {
    fn initial_idx() -> Self {
        DataIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        DataIdx { x: self.x + 1 }
    }
}

impl ToBytes for DataIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuncIdx {
    x: u32,
}

impl WasmIdx for FuncIdx {
    fn initial_idx() -> Self {
        FuncIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        FuncIdx { x: self.x + 1 }
    }
}

impl ToBytes for FuncIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalIdx {
    x: u32,
}

impl WasmIdx for LocalIdx {
    fn initial_idx() -> Self {
        LocalIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        LocalIdx { x: self.x + 1 }
    }
}

impl ToBytes for LocalIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalIdx {
    x: u32,
}

impl WasmIdx for GlobalIdx {
    fn initial_idx() -> Self {
        GlobalIdx { x: 0 }
    }

    fn next_idx(&self) -> Self {
        GlobalIdx { x: self.x + 1 }
    }
}

impl ToBytes for GlobalIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.x as u128)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelIdx {
    l: u32,
}

impl WasmIdx for LabelIdx {
    fn initial_idx() -> Self {
        LabelIdx { l: 0 }
    }

    fn next_idx(&self) -> Self {
        LabelIdx { l: self.l + 1 }
    }
}

impl ToBytes for LabelIdx {
    fn to_bytes(&self) -> Vec<u8> {
        encode_unsigned_int(self.l as u128)
    }
}
