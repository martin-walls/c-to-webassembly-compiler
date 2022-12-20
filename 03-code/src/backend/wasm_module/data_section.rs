use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_indices::MemIdx;
use crate::backend::wasm_instructions::WasmExpression;
use crate::backend::wasm_module::module::encode_section;

pub struct DataSection {
    data_segments: Vec<DataSegment>,
}

impl DataSection {
    pub fn new() -> Self {
        DataSection {
            data_segments: Vec::new(),
        }
    }
}

impl ToBytes for DataSection {
    fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = encode_vector(&self.data_segments);

        encode_section(0x0b, body_bytes)
    }
}

pub enum DataSegment {
    ActiveSegmentIndexZero {
        offset_expr: WasmExpression,
        data: Vec<u8>,
    },
    PassiveSegment {
        data: Vec<u8>,
    },
    ActiveSegmentExplicitIndex {
        memory_idx: MemIdx,
        offset_expr: WasmExpression,
        data: Vec<u8>,
    },
}

impl ToBytes for DataSegment {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            DataSegment::ActiveSegmentIndexZero { offset_expr, data } => {
                let mut bytes = encode_unsigned_int(0);
                bytes.append(&mut offset_expr.to_bytes());
                bytes.append(&mut data.clone());
                bytes
            }
            DataSegment::PassiveSegment { data } => {
                let mut bytes = encode_unsigned_int(1);
                bytes.append(&mut data.clone());
                bytes
            }
            DataSegment::ActiveSegmentExplicitIndex {
                memory_idx,
                offset_expr,
                data,
            } => {
                let mut bytes = encode_unsigned_int(2);
                bytes.append(&mut memory_idx.to_bytes());
                bytes.append(&mut offset_expr.to_bytes());
                bytes.append(&mut data.clone());
                bytes
            }
        }
    }
}
