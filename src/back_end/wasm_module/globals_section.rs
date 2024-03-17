use crate::back_end::to_bytes::ToBytes;
use crate::back_end::vector_encoding::encode_vector;
use crate::back_end::wasm_instructions::WasmExpression;
use crate::back_end::wasm_module::module::encode_section;
use crate::back_end::wasm_types::GlobalType;

pub struct GlobalsSection {
    globals: Vec<WasmGlobal>,
}

impl GlobalsSection {
    pub fn new() -> Self {
        GlobalsSection {
            globals: Vec::new(),
        }
    }
}

impl ToBytes for GlobalsSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.globals);

        encode_section(0x06, body_bytes)
    }
}

pub struct WasmGlobal {
    global_type: GlobalType,
    init_expr: WasmExpression,
}

impl ToBytes for WasmGlobal {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.global_type.to_bytes();
        bytes.append(&mut self.init_expr.to_bytes());
        bytes
    }
}
