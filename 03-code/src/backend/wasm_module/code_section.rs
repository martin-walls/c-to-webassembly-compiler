use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_instructions::WasmExpression;
use crate::backend::wasm_module::module::encode_section;
use crate::backend::wasm_types::ValType;

pub struct CodeSection {
    pub function_bodies: Vec<WasmFunctionCode>,
}

impl CodeSection {
    pub fn new() -> Self {
        CodeSection {
            function_bodies: Vec::new(),
        }
    }
}

impl ToBytes for CodeSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.function_bodies);

        encode_section(0x0a, body_bytes)
    }
}

pub struct WasmFunctionCode {
    pub local_declarations: Vec<LocalDeclaration>,
    pub function_body: WasmExpression,
}

impl ToBytes for WasmFunctionCode {
    fn to_bytes(&self) -> Vec<u8> {
        let mut body_bytes = encode_vector(&self.local_declarations);
        body_bytes.append(&mut self.function_body.to_bytes());

        let mut bytes = Vec::new();
        // function code size
        bytes.append(&mut encode_unsigned_int(body_bytes.len() as u128));
        bytes.append(&mut body_bytes);

        bytes
    }
}

pub struct LocalDeclaration {
    pub count: u32,
    pub value_type: ValType,
}

impl ToBytes for LocalDeclaration {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = encode_unsigned_int(self.count as u128);
        bytes.append(&mut self.value_type.to_bytes());
        bytes
    }
}
