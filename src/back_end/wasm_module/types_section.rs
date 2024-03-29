use crate::back_end::to_bytes::ToBytes;
use crate::back_end::vector_encoding::encode_vector;
use crate::back_end::wasm_module::module::encode_section;
use crate::back_end::wasm_types::ValType;

pub struct TypesSection {
    pub function_types: Vec<WasmFunctionType>,
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
        let body_bytes = encode_vector(&self.function_types);

        encode_section(0x01, body_bytes)
    }
}

pub struct WasmFunctionType {
    pub param_types: Vec<ValType>,
    pub result_types: Vec<ValType>,
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
