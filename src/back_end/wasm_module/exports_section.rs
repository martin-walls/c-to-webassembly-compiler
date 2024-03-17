use crate::back_end::integer_encoding::encode_unsigned_int;
use crate::back_end::to_bytes::ToBytes;
use crate::back_end::vector_encoding::encode_vector;
use crate::back_end::wasm_indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx};
use crate::back_end::wasm_module::module::encode_section;

pub struct ExportsSection {
    pub exports: Vec<WasmExport>,
}

impl ExportsSection {
    pub fn new() -> Self {
        ExportsSection {
            exports: Vec::new(),
        }
    }
}

impl ToBytes for ExportsSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.exports);

        encode_section(0x07, body_bytes)
    }
}

pub struct WasmExport {
    pub name: String,
    pub export_descriptor: ExportDescriptor,
}

impl ToBytes for WasmExport {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // export name
        let mut name_bytes = self.name.as_bytes().to_vec();
        // string length
        bytes.append(&mut encode_unsigned_int(name_bytes.len() as u128));
        bytes.append(&mut name_bytes);

        // export descriptor
        bytes.append(&mut self.export_descriptor.to_bytes());

        bytes
    }
}

pub enum ExportDescriptor {
    Func { func_idx: FuncIdx },
    Table { table_idx: TableIdx },
    Mem { mem_idx: MemIdx },
    Global { global_idx: GlobalIdx },
}

impl ToBytes for ExportDescriptor {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ExportDescriptor::Func { func_idx } => {
                let mut bytes = vec![0x00];
                bytes.append(&mut func_idx.to_bytes());
                bytes
            }
            ExportDescriptor::Table { table_idx } => {
                let mut bytes = vec![0x01];
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            ExportDescriptor::Mem { mem_idx } => {
                let mut bytes = vec![0x02];
                bytes.append(&mut mem_idx.to_bytes());
                bytes
            }
            ExportDescriptor::Global { global_idx } => {
                let mut bytes = vec![0x03];
                bytes.append(&mut global_idx.to_bytes());
                bytes
            }
        }
    }
}
