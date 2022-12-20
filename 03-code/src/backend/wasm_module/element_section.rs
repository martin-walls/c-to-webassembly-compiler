use crate::backend::to_bytes::ToBytes;
use crate::backend::wasm_module::module::encode_section;

pub struct ElementSection {}

impl ElementSection {
    pub fn new() -> Self {
        ElementSection {}
    }
}

impl ToBytes for ElementSection {
    fn to_bytes(&self) -> Vec<u8> {
        encode_section(0x09, Vec::new())
    }
}
