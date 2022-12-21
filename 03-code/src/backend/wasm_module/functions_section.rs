use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_indices::TypeIdx;
use crate::backend::wasm_module::module::encode_section;

pub struct FunctionsSection {
    pub function_type_idxs: Vec<TypeIdx>,
}

impl FunctionsSection {
    pub fn new() -> Self {
        FunctionsSection {
            function_type_idxs: Vec::new(),
        }
    }
}

impl ToBytes for FunctionsSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.function_type_idxs);

        encode_section(0x03, body_bytes)
    }
}
