use crate::back_end::integer_encoding::encode_unsigned_int;
use crate::back_end::to_bytes::ToBytes;
use crate::back_end::vector_encoding::encode_vector;
use crate::back_end::wasm_indices::TypeIdx;
use crate::back_end::wasm_module::module::encode_section;
use crate::back_end::wasm_types::{GlobalType, MemoryType, TableType};

pub struct ImportsSection {
    pub imports: Vec<WasmImport>,
}

impl ImportsSection {
    pub fn new() -> Self {
        ImportsSection {
            imports: Vec::new(),
        }
    }
}

impl ToBytes for ImportsSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.imports);

        encode_section(0x02, body_bytes)
    }
}

pub struct WasmImport {
    pub module_name: String,
    pub field_name: String,
    pub import_descriptor: ImportDescriptor,
}

impl ToBytes for WasmImport {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // module name
        let mut module_name_bytes = self.module_name.as_bytes().to_vec();
        // string length
        bytes.append(&mut encode_unsigned_int(module_name_bytes.len() as u128));
        bytes.append(&mut module_name_bytes);

        // field name
        let mut field_name_bytes = self.field_name.as_bytes().to_vec();
        // string length
        bytes.append(&mut encode_unsigned_int(field_name_bytes.len() as u128));
        bytes.append(&mut field_name_bytes);

        // import descriptor
        bytes.append(&mut self.import_descriptor.to_bytes());

        bytes
    }
}

pub enum ImportDescriptor {
    Func { func_type_idx: TypeIdx },
    Table { table_type: TableType },
    Mem { mem_type: MemoryType },
    Global { global_type: GlobalType },
}

impl ToBytes for ImportDescriptor {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ImportDescriptor::Func { func_type_idx } => {
                let mut bytes = vec![0x00];
                bytes.append(&mut func_type_idx.to_bytes());
                bytes
            }
            ImportDescriptor::Table { table_type } => {
                let mut bytes = vec![0x01];
                bytes.append(&mut table_type.to_bytes());
                bytes
            }
            ImportDescriptor::Mem { mem_type } => {
                let mut bytes = vec![0x02];
                bytes.append(&mut mem_type.to_bytes());
                bytes
            }
            ImportDescriptor::Global { global_type } => {
                let mut bytes = vec![0x03];
                bytes.append(&mut global_type.to_bytes());
                bytes
            }
        }
    }
}
