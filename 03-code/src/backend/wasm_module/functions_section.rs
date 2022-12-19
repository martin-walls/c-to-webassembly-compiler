use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_indices::TypeIdx;

pub struct FunctionsSection {
    function_type_idxs: Vec<TypeIdx>,
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
        let mut body_bytes = encode_vector(&self.function_type_idxs);

        let mut bytes = Vec::new();
        // section code
        bytes.push(0x03);
        // section size
        bytes.append(&mut encode_unsigned_int(body_bytes.len() as u128));
        // body
        bytes.append(&mut body_bytes);

        bytes
    }
}
