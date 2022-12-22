use crate::backend::float_encoding::encode_float;
use crate::backend::integer_encoding::{encode_signed_int, encode_unsigned_int};
use crate::backend::to_bytes::ToBytes;
use crate::backend::vector_encoding::encode_vector;
use crate::backend::wasm_indices::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, TableIdx, TypeIdx,
};
use crate::backend::wasm_types::{RefType, ValType};
use crate::middle_end::instructions::Instruction;

#[derive(Debug)]
pub struct WasmExpression {
    pub instrs: Vec<WasmInstruction>,
}

impl ToBytes for WasmExpression {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instr in &self.instrs {
            bytes.append(&mut instr.to_bytes());
        }
        // instructions followed by explicit `end`
        bytes.push(0x0b);
        bytes
    }
}

#[derive(Debug)]
pub enum BlockType {
    None,
    ValType(ValType),
    TypeIndex(i32),
}

impl ToBytes for BlockType {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            BlockType::None => {
                vec![0x40]
            }
            BlockType::ValType(val_type) => val_type.to_bytes(),
            BlockType::TypeIndex(i) => encode_signed_int(*i as i128),
        }
    }
}

#[derive(Debug)]
pub struct MemArg {
    /// Alignment hint. Memory accesses are more efficient if they line up with the alignment
    /// hint, but will still work if not.
    pub align: u32,
    /// Constant offset to add to the memory operand, to get the effective address.
    pub offset: u32,
}

impl MemArg {
    pub fn zero() -> Self {
        MemArg {
            align: 0,
            offset: 0,
        }
    }
}

impl ToBytes for MemArg {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = encode_unsigned_int(self.align as u128);
        bytes.append(&mut encode_unsigned_int(self.offset as u128));
        bytes
    }
}

/// See https://webassembly.github.io/spec/core/binary/instructions.html
#[derive(Debug)]
pub enum WasmInstruction {
    // Control instructions
    Unreachable,
    Nop,
    Block {
        blocktype: BlockType,
        instrs: Vec<WasmInstruction>,
    },
    Loop {
        blocktype: BlockType,
        instrs: Vec<WasmInstruction>,
    },
    IfElse {
        blocktype: BlockType,
        if_instrs: Vec<WasmInstruction>,
        else_instrs: Vec<WasmInstruction>,
    },

    Br {
        label_idx: LabelIdx,
    },
    BrIf {
        label_idx: LabelIdx,
    },
    BrTable {
        labels: Vec<LabelIdx>,
        label_idx: LabelIdx,
    },
    Return,
    Call {
        func_idx: FuncIdx,
    },
    CallIndirect {
        type_idx: TypeIdx,
        table_idx: TableIdx,
    },

    // Reference instructions
    RefNull {
        ref_type: RefType,
    },
    RefIsNull,
    RefFunc {
        func_idx: FuncIdx,
    },

    // Parametric instructions
    Drop,
    Select,
    SelectTyped {
        types: Vec<ValType>,
    },

    // Variable instructions
    LocalGet {
        local_idx: LocalIdx,
    },
    LocalSet {
        local_idx: LocalIdx,
    },
    LocalTee {
        local_idx: LocalIdx,
    },
    GlobalGet {
        global_idx: GlobalIdx,
    },
    GlobalSet {
        global_idx: GlobalIdx,
    },

    // Table instructions
    TableGet {
        table_idx: TableIdx,
    },
    TableSet {
        table_idx: TableIdx,
    },
    TableInit {
        elem_idx: ElemIdx,
        table_idx: TableIdx,
    },
    ElemDrop {
        elem_idx: ElemIdx,
    },
    TableCopy {
        table_idx1: TableIdx,
        table_idx2: TableIdx,
    },
    TableGrow {
        table_idx: TableIdx,
    },
    TableSize {
        table_idx: TableIdx,
    },
    TableFill {
        table_idx: TableIdx,
    },

    // Memory instructions
    I32Load {
        mem_arg: MemArg,
    },
    I64Load {
        mem_arg: MemArg,
    },
    F32Load {
        mem_arg: MemArg,
    },
    F64Load {
        mem_arg: MemArg,
    },
    I32Load8S {
        mem_arg: MemArg,
    },
    I32Load8U {
        mem_arg: MemArg,
    },
    I32Load16S {
        mem_arg: MemArg,
    },
    I32Load16U {
        mem_arg: MemArg,
    },
    I64Load8S {
        mem_arg: MemArg,
    },
    I64Load8U {
        mem_arg: MemArg,
    },
    I64Load16S {
        mem_arg: MemArg,
    },
    I64Load16U {
        mem_arg: MemArg,
    },
    I64Load32S {
        mem_arg: MemArg,
    },
    I64Load32U {
        mem_arg: MemArg,
    },

    I32Store {
        mem_arg: MemArg,
    },
    I64Store {
        mem_arg: MemArg,
    },
    F32Store {
        mem_arg: MemArg,
    },
    F64Store {
        mem_arg: MemArg,
    },
    I32Store8 {
        mem_arg: MemArg,
    },
    I32Store16 {
        mem_arg: MemArg,
    },
    I64Store8 {
        mem_arg: MemArg,
    },
    I64Store16 {
        mem_arg: MemArg,
    },
    I64Store32 {
        mem_arg: MemArg,
    },

    MemorySize,
    MemoryGrow,
    MemoryInit {
        data_idx: DataIdx,
    },
    DataDrop {
        data_idx: DataIdx,
    },
    MemoryCopy,
    MemoryFill,

    // Numeric instructions
    I32Const {
        n: i32,
    },
    I64Const {
        n: i64,
    },
    F32Const {
        z: f32,
    },
    F64Const {
        z: f64,
    },

    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    I32Clz,
    I32Ctz,
    I32PopCnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32RotL,
    I32RotR,

    I64Clz,
    I64Ctz,
    I64PopCnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64RotL,
    I64RotR,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32CopySign,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64CopySign,

    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TruncSatF64S,
    I64TruncSatF64U,
}

impl ToBytes for WasmInstruction {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            WasmInstruction::Unreachable => vec![0x00],
            WasmInstruction::Nop => vec![0x01],
            WasmInstruction::Block { blocktype, instrs } => {
                let mut bytes = vec![0x02];
                bytes.append(&mut blocktype.to_bytes());
                for instr in instrs {
                    bytes.append(&mut instr.to_bytes());
                }
                bytes.push(0x0b);
                bytes
            }
            WasmInstruction::Loop { blocktype, instrs } => {
                let mut bytes = vec![0x03];
                bytes.append(&mut blocktype.to_bytes());
                for instr in instrs {
                    bytes.append(&mut instr.to_bytes());
                }
                bytes.push(0x0b);
                bytes
            }
            WasmInstruction::IfElse {
                blocktype,
                if_instrs,
                else_instrs,
            } => {
                let mut bytes = vec![0x04];
                bytes.append(&mut blocktype.to_bytes());
                for instr in if_instrs {
                    bytes.append(&mut instr.to_bytes());
                }
                if !else_instrs.is_empty() {
                    bytes.push(0x05);
                    for instr in else_instrs {
                        bytes.append(&mut instr.to_bytes());
                    }
                }
                bytes.push(0x0b);
                bytes
            }
            WasmInstruction::Br { label_idx } => {
                let mut bytes = vec![0x0C];
                bytes.append(&mut label_idx.to_bytes());
                bytes
            }
            WasmInstruction::BrIf { label_idx } => {
                let mut bytes = vec![0x0D];
                bytes.append(&mut label_idx.to_bytes());
                bytes
            }
            WasmInstruction::BrTable { labels, label_idx } => {
                let mut bytes = vec![0x0E];
                bytes.append(&mut encode_vector(labels));
                bytes.append(&mut label_idx.to_bytes());
                bytes
            }
            WasmInstruction::Return => vec![0x0F],
            WasmInstruction::Call { func_idx } => {
                let mut bytes = vec![0x10];
                bytes.append(&mut func_idx.to_bytes());
                bytes
            }
            WasmInstruction::CallIndirect {
                type_idx,
                table_idx,
            } => {
                let mut bytes = vec![0x11];
                bytes.append(&mut type_idx.to_bytes());
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            WasmInstruction::RefNull { ref_type } => {
                let mut bytes = vec![0xD0];
                bytes.append(&mut ref_type.to_bytes());
                bytes
            }
            WasmInstruction::RefIsNull => vec![0xD1],
            WasmInstruction::RefFunc { func_idx } => {
                let mut bytes = vec![0xD2];
                bytes.append(&mut func_idx.to_bytes());
                bytes
            }
            WasmInstruction::Drop => vec![0x1A],
            WasmInstruction::Select => vec![0x1B],
            WasmInstruction::SelectTyped { types } => {
                let mut bytes = vec![0x1C];
                bytes.append(&mut encode_vector(types));
                bytes
            }
            WasmInstruction::LocalGet { local_idx } => {
                let mut bytes = vec![0x20];
                bytes.append(&mut local_idx.to_bytes());
                bytes
            }
            WasmInstruction::LocalSet { local_idx } => {
                let mut bytes = vec![0x21];
                bytes.append(&mut local_idx.to_bytes());
                bytes
            }
            WasmInstruction::LocalTee { local_idx } => {
                let mut bytes = vec![0x22];
                bytes.append(&mut local_idx.to_bytes());
                bytes
            }
            WasmInstruction::GlobalGet { global_idx } => {
                let mut bytes = vec![0x23];
                bytes.append(&mut global_idx.to_bytes());
                bytes
            }
            WasmInstruction::GlobalSet { global_idx } => {
                let mut bytes = vec![0x24];
                bytes.append(&mut global_idx.to_bytes());
                bytes
            }

            WasmInstruction::TableGet { table_idx } => {
                let mut bytes = vec![0x25];
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            WasmInstruction::TableSet { table_idx } => {
                let mut bytes = vec![0x26];
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            WasmInstruction::TableInit {
                elem_idx,
                table_idx,
            } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(12));
                bytes.append(&mut elem_idx.to_bytes());
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            WasmInstruction::ElemDrop { elem_idx } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(13));
                bytes.append(&mut elem_idx.to_bytes());
                bytes
            }
            WasmInstruction::TableCopy {
                table_idx1,
                table_idx2,
            } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(14));
                bytes.append(&mut table_idx1.to_bytes());
                bytes.append(&mut table_idx2.to_bytes());
                bytes
            }
            WasmInstruction::TableGrow { table_idx } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(15));
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            WasmInstruction::TableSize { table_idx } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(16));
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }
            WasmInstruction::TableFill { table_idx } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(17));
                bytes.append(&mut table_idx.to_bytes());
                bytes
            }

            WasmInstruction::I32Load { mem_arg } => {
                let mut bytes = vec![0x28];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load { mem_arg } => {
                let mut bytes = vec![0x29];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::F32Load { mem_arg } => {
                let mut bytes = vec![0x2A];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::F64Load { mem_arg } => {
                let mut bytes = vec![0x2B];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Load8S { mem_arg } => {
                let mut bytes = vec![0x2C];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Load8U { mem_arg } => {
                let mut bytes = vec![0x2D];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Load16S { mem_arg } => {
                let mut bytes = vec![0x2E];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Load16U { mem_arg } => {
                let mut bytes = vec![0x2F];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load8S { mem_arg } => {
                let mut bytes = vec![0x30];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load8U { mem_arg } => {
                let mut bytes = vec![0x31];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load16S { mem_arg } => {
                let mut bytes = vec![0x32];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load16U { mem_arg } => {
                let mut bytes = vec![0x33];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load32S { mem_arg } => {
                let mut bytes = vec![0x34];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Load32U { mem_arg } => {
                let mut bytes = vec![0x35];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Store { mem_arg } => {
                let mut bytes = vec![0x36];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Store { mem_arg } => {
                let mut bytes = vec![0x37];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::F32Store { mem_arg } => {
                let mut bytes = vec![0x38];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::F64Store { mem_arg } => {
                let mut bytes = vec![0x39];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Store8 { mem_arg } => {
                let mut bytes = vec![0x3A];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I32Store16 { mem_arg } => {
                let mut bytes = vec![0x3B];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Store8 { mem_arg } => {
                let mut bytes = vec![0x3C];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Store16 { mem_arg } => {
                let mut bytes = vec![0x3D];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::I64Store32 { mem_arg } => {
                let mut bytes = vec![0x3E];
                bytes.append(&mut mem_arg.to_bytes());
                bytes
            }
            WasmInstruction::MemorySize => {
                vec![0x3F, 0x00]
            }
            WasmInstruction::MemoryGrow => {
                vec![0x40, 0x00]
            }
            WasmInstruction::MemoryInit { data_idx } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(8));
                bytes.append(&mut data_idx.to_bytes());
                bytes.push(0x00);
                bytes
            }
            WasmInstruction::DataDrop { data_idx } => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(9));
                bytes.append(&mut data_idx.to_bytes());
                bytes
            }
            WasmInstruction::MemoryCopy => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(10));
                bytes.push(0x00);
                bytes.push(0x00);
                bytes
            }
            WasmInstruction::MemoryFill => {
                let mut bytes = vec![0xFC];
                bytes.append(&mut encode_unsigned_int(11));
                bytes.push(0x00);
                bytes
            }
            WasmInstruction::I32Const { n } => {
                let mut bytes = vec![0x41];
                bytes.append(&mut encode_signed_int(*n as i128));
                bytes
            }
            WasmInstruction::I64Const { n } => {
                let mut bytes = vec![0x42];
                bytes.append(&mut encode_signed_int(*n as i128));
                bytes
            }
            WasmInstruction::F32Const { z } => {
                let mut bytes = vec![0x43];
                bytes.append(&mut encode_float(*z as f64));
                bytes
            }
            WasmInstruction::F64Const { z } => {
                let mut bytes = vec![0x44];
                bytes.append(&mut encode_float(*z));
                bytes
            }

            WasmInstruction::I32Eqz => {
                vec![0x45]
            }
            WasmInstruction::I32Eq => {
                vec![0x46]
            }
            WasmInstruction::I32Ne => {
                vec![0x47]
            }
            WasmInstruction::I32LtS => {
                vec![0x48]
            }
            WasmInstruction::I32LtU => {
                vec![0x49]
            }
            WasmInstruction::I32GtS => {
                vec![0x4A]
            }
            WasmInstruction::I32GtU => {
                vec![0x4b]
            }
            WasmInstruction::I32LeS => {
                vec![0x4c]
            }
            WasmInstruction::I32LeU => {
                vec![0x4d]
            }
            WasmInstruction::I32GeS => {
                vec![0x4e]
            }
            WasmInstruction::I32GeU => {
                vec![0x4f]
            }

            WasmInstruction::I64Eqz => {
                vec![0x50]
            }
            WasmInstruction::I64Eq => {
                vec![0x51]
            }
            WasmInstruction::I64Ne => {
                vec![0x52]
            }
            WasmInstruction::I64LtS => {
                vec![0x53]
            }
            WasmInstruction::I64LtU => {
                vec![0x54]
            }
            WasmInstruction::I64GtS => {
                vec![0x55]
            }
            WasmInstruction::I64GtU => {
                vec![0x56]
            }
            WasmInstruction::I64LeS => {
                vec![0x57]
            }
            WasmInstruction::I64LeU => {
                vec![0x58]
            }
            WasmInstruction::I64GeS => {
                vec![0x59]
            }
            WasmInstruction::I64GeU => {
                vec![0x5a]
            }

            WasmInstruction::F32Eq => {
                vec![0x5b]
            }
            WasmInstruction::F32Ne => {
                vec![0x5c]
            }
            WasmInstruction::F32Lt => {
                vec![0x5d]
            }
            WasmInstruction::F32Gt => {
                vec![0x5e]
            }
            WasmInstruction::F32Le => {
                vec![0x5f]
            }
            WasmInstruction::F32Ge => {
                vec![0x60]
            }

            WasmInstruction::F64Eq => {
                vec![0x61]
            }
            WasmInstruction::F64Ne => {
                vec![0x62]
            }
            WasmInstruction::F64Lt => {
                vec![0x63]
            }
            WasmInstruction::F64Gt => {
                vec![0x64]
            }
            WasmInstruction::F64Le => {
                vec![0x65]
            }
            WasmInstruction::F64Ge => {
                vec![0x66]
            }

            WasmInstruction::I32Clz => {
                vec![0x67]
            }
            WasmInstruction::I32Ctz => {
                vec![0x68]
            }
            WasmInstruction::I32PopCnt => {
                vec![0x69]
            }
            WasmInstruction::I32Add => {
                vec![0x6a]
            }
            WasmInstruction::I32Sub => {
                vec![0x6b]
            }
            WasmInstruction::I32Mul => {
                vec![0x6c]
            }
            WasmInstruction::I32DivS => {
                vec![0x6d]
            }
            WasmInstruction::I32DivU => {
                vec![0x6e]
            }
            WasmInstruction::I32RemS => {
                vec![0x6f]
            }
            WasmInstruction::I32RemU => {
                vec![0x70]
            }
            WasmInstruction::I32And => {
                vec![0x71]
            }
            WasmInstruction::I32Or => {
                vec![0x72]
            }
            WasmInstruction::I32Xor => {
                vec![0x73]
            }
            WasmInstruction::I32Shl => {
                vec![0x74]
            }
            WasmInstruction::I32ShrS => {
                vec![0x75]
            }
            WasmInstruction::I32ShrU => {
                vec![0x76]
            }
            WasmInstruction::I32RotL => {
                vec![0x77]
            }
            WasmInstruction::I32RotR => {
                vec![0x78]
            }

            WasmInstruction::I64Clz => {
                vec![0x79]
            }
            WasmInstruction::I64Ctz => {
                vec![0x7a]
            }
            WasmInstruction::I64PopCnt => {
                vec![0x7b]
            }
            WasmInstruction::I64Add => {
                vec![0x7c]
            }
            WasmInstruction::I64Sub => {
                vec![0x7d]
            }
            WasmInstruction::I64Mul => {
                vec![0x7e]
            }
            WasmInstruction::I64DivS => {
                vec![0x7f]
            }
            WasmInstruction::I64DivU => {
                vec![0x80]
            }
            WasmInstruction::I64RemS => {
                vec![0x81]
            }
            WasmInstruction::I64RemU => {
                vec![0x82]
            }
            WasmInstruction::I64And => {
                vec![0x83]
            }
            WasmInstruction::I64Or => {
                vec![0x84]
            }
            WasmInstruction::I64Xor => {
                vec![0x85]
            }
            WasmInstruction::I64Shl => {
                vec![0x86]
            }
            WasmInstruction::I64ShrS => {
                vec![0x87]
            }
            WasmInstruction::I64ShrU => {
                vec![0x88]
            }
            WasmInstruction::I64RotL => {
                vec![0x89]
            }
            WasmInstruction::I64RotR => {
                vec![0x8a]
            }

            WasmInstruction::F32Abs => {
                vec![0x8b]
            }
            WasmInstruction::F32Neg => {
                vec![0x8c]
            }
            WasmInstruction::F32Ceil => {
                vec![0x8d]
            }
            WasmInstruction::F32Floor => {
                vec![0x8e]
            }
            WasmInstruction::F32Trunc => {
                vec![0x8f]
            }
            WasmInstruction::F32Nearest => {
                vec![0x90]
            }
            WasmInstruction::F32Sqrt => {
                vec![0x91]
            }
            WasmInstruction::F32Add => {
                vec![0x92]
            }
            WasmInstruction::F32Sub => {
                vec![0x93]
            }
            WasmInstruction::F32Mul => {
                vec![0x94]
            }
            WasmInstruction::F32Div => {
                vec![0x95]
            }
            WasmInstruction::F32Min => {
                vec![0x96]
            }
            WasmInstruction::F32Max => {
                vec![0x97]
            }
            WasmInstruction::F32CopySign => {
                vec![0x98]
            }

            WasmInstruction::F64Abs => {
                vec![0x99]
            }
            WasmInstruction::F64Neg => {
                vec![0x9a]
            }
            WasmInstruction::F64Ceil => {
                vec![0x9b]
            }
            WasmInstruction::F64Floor => {
                vec![0x9c]
            }
            WasmInstruction::F64Trunc => {
                vec![0x9d]
            }
            WasmInstruction::F64Nearest => {
                vec![0x9e]
            }
            WasmInstruction::F64Sqrt => {
                vec![0x9f]
            }
            WasmInstruction::F64Add => {
                vec![0xa0]
            }
            WasmInstruction::F64Sub => {
                vec![0xa1]
            }
            WasmInstruction::F64Mul => {
                vec![0xa2]
            }
            WasmInstruction::F64Div => {
                vec![0xa3]
            }
            WasmInstruction::F64Min => {
                vec![0xa4]
            }
            WasmInstruction::F64Max => {
                vec![0xa5]
            }
            WasmInstruction::F64CopySign => {
                vec![0xa6]
            }

            WasmInstruction::I32WrapI64 => {
                vec![0xa7]
            }
            WasmInstruction::I32TruncF32S => {
                vec![0xa8]
            }
            WasmInstruction::I32TruncF32U => {
                vec![0xa9]
            }
            WasmInstruction::I32TruncF64S => {
                vec![0xaa]
            }
            WasmInstruction::I32TruncF64U => {
                vec![0xab]
            }
            WasmInstruction::I64ExtendI32S => {
                vec![0xac]
            }
            WasmInstruction::I64ExtendI32U => {
                vec![0xad]
            }
            WasmInstruction::I64TruncF32S => {
                vec![0xae]
            }
            WasmInstruction::I64TruncF32U => {
                vec![0xaf]
            }
            WasmInstruction::I64TruncF64S => {
                vec![0xb0]
            }
            WasmInstruction::I64TruncF64U => {
                vec![0xb1]
            }
            WasmInstruction::F32ConvertI32S => {
                vec![0xb2]
            }
            WasmInstruction::F32ConvertI32U => {
                vec![0xb3]
            }
            WasmInstruction::F32ConvertI64S => {
                vec![0xb4]
            }
            WasmInstruction::F32ConvertI64U => {
                vec![0xb5]
            }
            WasmInstruction::F32DemoteF64 => {
                vec![0xb6]
            }
            WasmInstruction::F64ConvertI32S => {
                vec![0xb7]
            }
            WasmInstruction::F64ConvertI32U => {
                vec![0xb8]
            }
            WasmInstruction::F64ConvertI64S => {
                vec![0xb9]
            }
            WasmInstruction::F64ConvertI64U => {
                vec![0xba]
            }
            WasmInstruction::F64PromoteF32 => {
                vec![0xbb]
            }
            WasmInstruction::I32ReinterpretF32 => {
                vec![0xbc]
            }
            WasmInstruction::I64ReinterpretF64 => {
                vec![0xbd]
            }
            WasmInstruction::F32ReinterpretI32 => {
                vec![0xbe]
            }
            WasmInstruction::F64ReinterpretI64 => {
                vec![0xbf]
            }

            WasmInstruction::I32Extend8S => {
                vec![0xc0]
            }
            WasmInstruction::I32Extend16S => {
                vec![0xc1]
            }
            WasmInstruction::I64Extend8S => {
                vec![0xc2]
            }
            WasmInstruction::I64Extend16S => {
                vec![0xc3]
            }
            WasmInstruction::I64Extend32S => {
                vec![0xc4]
            }

            WasmInstruction::I32TruncSatF32S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(0));
                v
            }
            WasmInstruction::I32TruncSatF32U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(1));
                v
            }
            WasmInstruction::I32TruncSatF64S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(2));
                v
            }
            WasmInstruction::I32TruncSatF64U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(3));
                v
            }
            WasmInstruction::I64TruncSatF32S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(4));
                v
            }
            WasmInstruction::I64TruncSatF32U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(5));
                v
            }
            WasmInstruction::I64TruncSatF64S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(6));
                v
            }
            WasmInstruction::I64TruncSatF64U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(7));
                v
            }
        }
    }
}
