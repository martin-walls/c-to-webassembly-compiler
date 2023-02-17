use crate::back_end::to_bytes::ToBytes;
use crate::back_end::wasm_indices::FuncIdx;
use crate::back_end::wasm_module::module::encode_section;

pub struct StartSection {
    pub start_func_idx: Option<FuncIdx>,
}

impl StartSection {
    pub fn new() -> Self {
        StartSection {
            start_func_idx: None,
        }
    }
}

impl ToBytes for StartSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = match &self.start_func_idx {
            None => Vec::new(),
            Some(func_idx) => func_idx.to_bytes(),
        };

        encode_section(0x08, body_bytes)
    }
}
