use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_module::module::encode_section;
use crate::backend::wasm_types::TableType;

pub struct TablesSection {
    table_types: Vec<TableType>,
}

impl TablesSection {
    pub fn new() -> Self {
        TablesSection {
            table_types: Vec::new(),
        }
    }
}

impl ToBytes for TablesSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.table_types);

        encode_section(0x04, body_bytes)
    }
}
