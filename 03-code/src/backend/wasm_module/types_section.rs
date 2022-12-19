use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_types::ValType;

pub struct TypesSection {
    function_types: Vec<WasmFunctionType>,
}

impl TypesSection {
    pub fn new() -> Self {
        TypesSection {
            function_types: Vec::new(),
        }
    }
}

impl ToBytes for TypesSection {
    fn to_bytes(&self) -> Vec<u8> {
        // body of section
        let mut body_bytes = encode_vector(&self.function_types);

        let mut bytes = Vec::new();
        // section code
        bytes.push(0x01);
        // section size
        bytes.append(&mut encode_unsigned_int(body_bytes.len() as u128));
        // body
        bytes.append(&mut body_bytes);

        bytes
    }
}

pub struct WasmFunctionType {
    param_types: Vec<ValType>,
    result_types: Vec<ValType>,
}

impl ToBytes for WasmFunctionType {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // it's a function type
        bytes.push(0x60);

        // vector of parameter types
        bytes.append(&mut encode_vector(&self.param_types));

        // vector of result types
        bytes.append(&mut encode_vector(&self.result_types));

        bytes
    }
}
