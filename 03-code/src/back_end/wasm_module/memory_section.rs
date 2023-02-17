use crate::back_end::to_bytes::ToBytes;
use crate::back_end::vector_encoding::encode_vector;
use crate::back_end::wasm_module::module::encode_section;
use crate::back_end::wasm_types::MemoryType;

pub struct MemorySection {
    pub memory_types: Vec<MemoryType>,
}

impl MemorySection {
    pub fn new() -> Self {
        MemorySection {
            memory_types: Vec::new(),
        }
    }
}

impl ToBytes for MemorySection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.memory_types);

        encode_section(0x05, body_bytes)
    }
}
