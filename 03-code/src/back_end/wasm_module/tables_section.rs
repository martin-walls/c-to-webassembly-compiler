use crate::back_end::to_bytes::ToBytes;
use crate::back_end::vector_encoding::encode_vector;
use crate::back_end::wasm_module::module::encode_section;
use crate::back_end::wasm_types::TableType;

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
