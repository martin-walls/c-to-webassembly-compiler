use crate::backend::integer_encoding::encode_unsigned_int;

/// See https://webassembly.github.io/spec/core/binary/instructions.html
pub enum WasmOpcode {
    // Control instructions
    Unreachable,
    Nop,
    Block,
    Loop,
    If,
    Else,
    End,
    Br,
    BrIf,
    BrTable,
    Return,
    Call,
    CallIndirect,

    // Reference instructions
    RefNull,
    RefIsNull,
    RefFunc,

    // Parametric instructions
    Drop,
    Select,
    SelectTyped,

    // Variable instructions
    LocalGet,
    LocalSet,
    LocalTee,
    GlobalGet,
    GlobalSet,

    // Table instructions
    TableGet,
    TableSet,
    TableInit,
    ElemDrop,
    TableCopy,
    TableGrow,
    TableSize,
    TableFill,

    // Memory instructions
    I32Load,
    I64Load,
    F32Load,
    F64Load,
    I32Load8S,
    I32Load8U,
    I32Load16S,
    I32Load16U,
    I64Load8S,
    I64Load8U,
    I64Load16S,
    I64Load16U,
    I64Load32S,
    I64Load32U,

    I32Store,
    I64Store,
    F32Store,
    F64Store,
    I32Store8,
    I32Store16,
    I64Store8,
    I64Store16,
    I64Store32,

    MemorySize,
    MemoryGrow,
    MemoryInit,
    DataDrop,
    MemoryCopy,
    MemoryFill,

    // Numeric instructions
    I32Const,
    I64Const,
    F32Const,
    F64Const,

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

impl WasmOpcode {
    pub fn get_byte_code(&self) -> Vec<u8> {
        match self {
            WasmOpcode::Unreachable => vec![0x00],
            WasmOpcode::Nop => vec![0x01],
            WasmOpcode::Block => vec![0x02],
            WasmOpcode::Loop => vec![0x03],
            WasmOpcode::If => vec![0x04],
            WasmOpcode::Else => vec![0x05],
            WasmOpcode::End => vec![0x0B],
            WasmOpcode::Br => vec![0x0C],
            WasmOpcode::BrIf => vec![0x0D],
            WasmOpcode::BrTable => vec![0x0E],
            WasmOpcode::Return => vec![0x0F],
            WasmOpcode::Call => vec![0x10],
            WasmOpcode::CallIndirect => vec![0x11],
            WasmOpcode::RefNull => vec![0xD0],
            WasmOpcode::RefIsNull => vec![0xD1],
            WasmOpcode::RefFunc => vec![0xD2],
            WasmOpcode::Drop => vec![0x1A],
            WasmOpcode::Select => vec![0x1B],
            WasmOpcode::SelectTyped => vec![0x1C],
            WasmOpcode::LocalGet => vec![0x20],
            WasmOpcode::LocalSet => vec![0x21],
            WasmOpcode::LocalTee => vec![0x22],
            WasmOpcode::GlobalGet => vec![0x23],
            WasmOpcode::GlobalSet => vec![0x24],

            WasmOpcode::TableGet => {
                vec![0x25]
            }
            WasmOpcode::TableSet => {
                vec![0x26]
            }
            WasmOpcode::TableInit => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(12));
                v
            }
            WasmOpcode::ElemDrop => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(13));
                v
            }
            WasmOpcode::TableCopy => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(14));
                v
            }
            WasmOpcode::TableGrow => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(15));
                v
            }
            WasmOpcode::TableSize => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(16));
                v
            }
            WasmOpcode::TableFill => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(17));
                v
            }

            WasmOpcode::I32Load => {
                vec![0x28]
            }
            WasmOpcode::I64Load => {
                vec![0x29]
            }
            WasmOpcode::F32Load => {
                vec![0x2A]
            }
            WasmOpcode::F64Load => {
                vec![0x2B]
            }
            WasmOpcode::I32Load8S => {
                vec![0x2C]
            }
            WasmOpcode::I32Load8U => {
                vec![0x2D]
            }
            WasmOpcode::I32Load16S => {
                vec![0x2E]
            }
            WasmOpcode::I32Load16U => {
                vec![0x2F]
            }
            WasmOpcode::I64Load8S => {
                vec![0x30]
            }
            WasmOpcode::I64Load8U => {
                vec![0x31]
            }
            WasmOpcode::I64Load16S => {
                vec![0x32]
            }
            WasmOpcode::I64Load16U => {
                vec![0x33]
            }
            WasmOpcode::I64Load32S => {
                vec![0x34]
            }
            WasmOpcode::I64Load32U => {
                vec![0x35]
            }
            WasmOpcode::I32Store => {
                vec![0x36]
            }
            WasmOpcode::I64Store => {
                vec![0x37]
            }
            WasmOpcode::F32Store => {
                vec![0x38]
            }
            WasmOpcode::F64Store => {
                vec![0x39]
            }
            WasmOpcode::I32Store8 => {
                vec![0x3A]
            }
            WasmOpcode::I32Store16 => {
                vec![0x3B]
            }
            WasmOpcode::I64Store8 => {
                vec![0x3C]
            }
            WasmOpcode::I64Store16 => {
                vec![0x3D]
            }
            WasmOpcode::I64Store32 => {
                vec![0x3E]
            }
            WasmOpcode::MemorySize => {
                vec![0x3F, 0x00]
            }
            WasmOpcode::MemoryGrow => {
                vec![0x40, 0x00]
            }
            WasmOpcode::MemoryInit => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(8));
                v
            }
            WasmOpcode::DataDrop => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(9));
                v
            }
            WasmOpcode::MemoryCopy => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(10));
                v.push(0x00);
                v.push(0x00);
                v
            }
            WasmOpcode::MemoryFill => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(11));
                v.push(0x00);
                v
            }
            WasmOpcode::I32Const => {
                vec![0x41]
            }
            WasmOpcode::I64Const => {
                vec![0x42]
            }
            WasmOpcode::F32Const => {
                vec![0x43]
            }
            WasmOpcode::F64Const => {
                vec![0x44]
            }

            WasmOpcode::I32Eqz => {
                vec![0x45]
            }
            WasmOpcode::I32Eq => {
                vec![0x46]
            }
            WasmOpcode::I32Ne => {
                vec![0x47]
            }
            WasmOpcode::I32LtS => {
                vec![0x48]
            }
            WasmOpcode::I32LtU => {
                vec![0x49]
            }
            WasmOpcode::I32GtS => {
                vec![0x4A]
            }
            WasmOpcode::I32GtU => {
                vec![0x4b]
            }
            WasmOpcode::I32LeS => {
                vec![0x4c]
            }
            WasmOpcode::I32LeU => {
                vec![0x4d]
            }
            WasmOpcode::I32GeS => {
                vec![0x4e]
            }
            WasmOpcode::I32GeU => {
                vec![0x4f]
            }

            WasmOpcode::I64Eqz => {
                vec![0x50]
            }
            WasmOpcode::I64Eq => {
                vec![0x51]
            }
            WasmOpcode::I64Ne => {
                vec![0x52]
            }
            WasmOpcode::I64LtS => {
                vec![0x53]
            }
            WasmOpcode::I64LtU => {
                vec![0x54]
            }
            WasmOpcode::I64GtS => {
                vec![0x55]
            }
            WasmOpcode::I64GtU => {
                vec![0x56]
            }
            WasmOpcode::I64LeS => {
                vec![0x57]
            }
            WasmOpcode::I64LeU => {
                vec![0x58]
            }
            WasmOpcode::I64GeS => {
                vec![0x59]
            }
            WasmOpcode::I64GeU => {
                vec![0x5a]
            }

            WasmOpcode::F32Eq => {
                vec![0x5b]
            }
            WasmOpcode::F32Ne => {
                vec![0x5c]
            }
            WasmOpcode::F32Lt => {
                vec![0x5d]
            }
            WasmOpcode::F32Gt => {
                vec![0x5e]
            }
            WasmOpcode::F32Le => {
                vec![0x5f]
            }
            WasmOpcode::F32Ge => {
                vec![0x60]
            }

            WasmOpcode::F64Eq => {
                vec![0x61]
            }
            WasmOpcode::F64Ne => {
                vec![0x62]
            }
            WasmOpcode::F64Lt => {
                vec![0x63]
            }
            WasmOpcode::F64Gt => {
                vec![0x64]
            }
            WasmOpcode::F64Le => {
                vec![0x65]
            }
            WasmOpcode::F64Ge => {
                vec![0x66]
            }

            WasmOpcode::I32Clz => {
                vec![0x67]
            }
            WasmOpcode::I32Ctz => {
                vec![0x68]
            }
            WasmOpcode::I32PopCnt => {
                vec![0x69]
            }
            WasmOpcode::I32Add => {
                vec![0x6a]
            }
            WasmOpcode::I32Sub => {
                vec![0x6b]
            }
            WasmOpcode::I32Mul => {
                vec![0x6c]
            }
            WasmOpcode::I32DivS => {
                vec![0x6d]
            }
            WasmOpcode::I32DivU => {
                vec![0x6e]
            }
            WasmOpcode::I32RemS => {
                vec![0x6f]
            }
            WasmOpcode::I32RemU => {
                vec![0x70]
            }
            WasmOpcode::I32And => {
                vec![0x71]
            }
            WasmOpcode::I32Or => {
                vec![0x72]
            }
            WasmOpcode::I32Xor => {
                vec![0x73]
            }
            WasmOpcode::I32Shl => {
                vec![0x74]
            }
            WasmOpcode::I32ShrS => {
                vec![0x75]
            }
            WasmOpcode::I32ShrU => {
                vec![0x76]
            }
            WasmOpcode::I32RotL => {
                vec![0x77]
            }
            WasmOpcode::I32RotR => {
                vec![0x78]
            }

            WasmOpcode::I64Clz => {
                vec![0x79]
            }
            WasmOpcode::I64Ctz => {
                vec![0x7a]
            }
            WasmOpcode::I64PopCnt => {
                vec![0x7b]
            }
            WasmOpcode::I64Add => {
                vec![0x7c]
            }
            WasmOpcode::I64Sub => {
                vec![0x7d]
            }
            WasmOpcode::I64Mul => {
                vec![0x7e]
            }
            WasmOpcode::I64DivS => {
                vec![0x7f]
            }
            WasmOpcode::I64DivU => {
                vec![0x80]
            }
            WasmOpcode::I64RemS => {
                vec![0x81]
            }
            WasmOpcode::I64RemU => {
                vec![0x82]
            }
            WasmOpcode::I64And => {
                vec![0x83]
            }
            WasmOpcode::I64Or => {
                vec![0x84]
            }
            WasmOpcode::I64Xor => {
                vec![0x85]
            }
            WasmOpcode::I64Shl => {
                vec![0x86]
            }
            WasmOpcode::I64ShrS => {
                vec![0x87]
            }
            WasmOpcode::I64ShrU => {
                vec![0x88]
            }
            WasmOpcode::I64RotL => {
                vec![0x89]
            }
            WasmOpcode::I64RotR => {
                vec![0x8a]
            }

            WasmOpcode::F32Abs => {
                vec![0x8b]
            }
            WasmOpcode::F32Neg => {
                vec![0x8c]
            }
            WasmOpcode::F32Ceil => {
                vec![0x8d]
            }
            WasmOpcode::F32Floor => {
                vec![0x8e]
            }
            WasmOpcode::F32Trunc => {
                vec![0x8f]
            }
            WasmOpcode::F32Nearest => {
                vec![0x90]
            }
            WasmOpcode::F32Sqrt => {
                vec![0x91]
            }
            WasmOpcode::F32Add => {
                vec![0x92]
            }
            WasmOpcode::F32Sub => {
                vec![0x93]
            }
            WasmOpcode::F32Mul => {
                vec![0x94]
            }
            WasmOpcode::F32Div => {
                vec![0x95]
            }
            WasmOpcode::F32Min => {
                vec![0x96]
            }
            WasmOpcode::F32Max => {
                vec![0x97]
            }
            WasmOpcode::F32CopySign => {
                vec![0x98]
            }

            WasmOpcode::F64Abs => {
                vec![0x99]
            }
            WasmOpcode::F64Neg => {
                vec![0x9a]
            }
            WasmOpcode::F64Ceil => {
                vec![0x9b]
            }
            WasmOpcode::F64Floor => {
                vec![0x9c]
            }
            WasmOpcode::F64Trunc => {
                vec![0x9d]
            }
            WasmOpcode::F64Nearest => {
                vec![0x9e]
            }
            WasmOpcode::F64Sqrt => {
                vec![0x9f]
            }
            WasmOpcode::F64Add => {
                vec![0xa0]
            }
            WasmOpcode::F64Sub => {
                vec![0xa1]
            }
            WasmOpcode::F64Mul => {
                vec![0xa2]
            }
            WasmOpcode::F64Div => {
                vec![0xa3]
            }
            WasmOpcode::F64Min => {
                vec![0xa4]
            }
            WasmOpcode::F64Max => {
                vec![0xa5]
            }
            WasmOpcode::F64CopySign => {
                vec![0xa6]
            }

            WasmOpcode::I32WrapI64 => {
                vec![0xa7]
            }
            WasmOpcode::I32TruncF32S => {
                vec![0xa8]
            }
            WasmOpcode::I32TruncF32U => {
                vec![0xa9]
            }
            WasmOpcode::I32TruncF64S => {
                vec![0xaa]
            }
            WasmOpcode::I32TruncF64U => {
                vec![0xab]
            }
            WasmOpcode::I64ExtendI32S => {
                vec![0xac]
            }
            WasmOpcode::I64ExtendI32U => {
                vec![0xad]
            }
            WasmOpcode::I64TruncF32S => {
                vec![0xae]
            }
            WasmOpcode::I64TruncF32U => {
                vec![0xaf]
            }
            WasmOpcode::I64TruncF64S => {
                vec![0xb0]
            }
            WasmOpcode::I64TruncF64U => {
                vec![0xb1]
            }
            WasmOpcode::F32ConvertI32S => {
                vec![0xb2]
            }
            WasmOpcode::F32ConvertI32U => {
                vec![0xb3]
            }
            WasmOpcode::F32ConvertI64S => {
                vec![0xb4]
            }
            WasmOpcode::F32ConvertI64U => {
                vec![0xb5]
            }
            WasmOpcode::F32DemoteF64 => {
                vec![0xb6]
            }
            WasmOpcode::F64ConvertI32S => {
                vec![0xb7]
            }
            WasmOpcode::F64ConvertI32U => {
                vec![0xb8]
            }
            WasmOpcode::F64ConvertI64S => {
                vec![0xb9]
            }
            WasmOpcode::F64ConvertI64U => {
                vec![0xba]
            }
            WasmOpcode::F64PromoteF32 => {
                vec![0xbb]
            }
            WasmOpcode::I32ReinterpretF32 => {
                vec![0xbc]
            }
            WasmOpcode::I64ReinterpretF64 => {
                vec![0xbd]
            }
            WasmOpcode::F32ReinterpretI32 => {
                vec![0xbe]
            }
            WasmOpcode::F64ReinterpretI64 => {
                vec![0xbf]
            }

            WasmOpcode::I32Extend8S => {
                vec![0xc0]
            }
            WasmOpcode::I32Extend16S => {
                vec![0xc1]
            }
            WasmOpcode::I64Extend8S => {
                vec![0xc2]
            }
            WasmOpcode::I64Extend16S => {
                vec![0xc3]
            }
            WasmOpcode::I64Extend32S => {
                vec![0xc4]
            }

            WasmOpcode::I32TruncSatF32S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(0));
                v
            }
            WasmOpcode::I32TruncSatF32U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(1));
                v
            }
            WasmOpcode::I32TruncSatF64S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(2));
                v
            }
            WasmOpcode::I32TruncSatF64U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(3));
                v
            }
            WasmOpcode::I64TruncSatF32S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(4));
                v
            }
            WasmOpcode::I64TruncSatF32U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(5));
                v
            }
            WasmOpcode::I64TruncSatF64S => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(6));
                v
            }
            WasmOpcode::I64TruncSatF64U => {
                let mut v = vec![0xFC];
                v.append(&mut encode_unsigned_int(7));
                v
            }
        }
    }
}
