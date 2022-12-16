use crate::middle_end::instructions::{Instruction, Src};
use crate::middle_end::ir::Program;
use crate::middle_end::ir_types::IrType;
use crate::middle_end::middle_end_error::MiddleEndError;
use log::trace;

pub fn unary_convert(
    src: Src,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    let src_type = src.get_type(&prog.program_metadata)?;
    let unary_converted_type = src_type.unary_convert();
    if src_type != unary_converted_type {
        let (instrs, converted_var) =
            get_type_conversion_instrs(src, src_type, unary_converted_type.to_owned(), prog)?;
        return Ok((instrs, converted_var));
    }
    Ok((Vec::new(), src))
}

pub fn binary_convert(
    left: Src,
    right: Src,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src, Src), MiddleEndError> {
    let (mut left_convert_instrs, mut right_convert_instrs, left_result, right_result) =
        binary_convert_separately(left, right, prog)?;
    left_convert_instrs.append(&mut right_convert_instrs);
    Ok((left_convert_instrs, left_result, right_result))
}

pub fn binary_convert_separately(
    left: Src,
    right: Src,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Vec<Instruction>, Src, Src), MiddleEndError> {
    let mut left_instrs = Vec::new();
    let mut right_instrs = Vec::new();
    let (mut left_unary_convert_instrs, unary_left) = unary_convert(left, prog)?;
    left_instrs.append(&mut left_unary_convert_instrs);
    let (mut right_unary_convert_instrs, unary_right) = unary_convert(right, prog)?;
    right_instrs.append(&mut right_unary_convert_instrs);
    let left_type = unary_left.get_type(&prog.program_metadata)?;
    let right_type = unary_right.get_type(&prog.program_metadata)?;
    if left_type == right_type
        || !left_type.is_arithmetic_type()
        || !right_type.is_arithmetic_type()
    {
        return Ok((left_instrs, right_instrs, unary_left, unary_right));
    }

    // if one operand is a double
    if *left_type == IrType::F64 {
        let right_dest = prog.new_var(unary_right.get_value_type());
        match *right_type {
            IrType::I32 => {
                right_instrs.push(Instruction::I32toF64(right_dest.to_owned(), unary_right))
            }
            IrType::U32 => {
                right_instrs.push(Instruction::U32toF64(right_dest.to_owned(), unary_right))
            }
            IrType::I64 => {
                right_instrs.push(Instruction::I64toF64(right_dest.to_owned(), unary_right))
            }
            IrType::U64 => {
                right_instrs.push(Instruction::U64toF64(right_dest.to_owned(), unary_right))
            }
            IrType::F32 => {
                right_instrs.push(Instruction::F32toF64(right_dest.to_owned(), unary_right))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(right_dest.to_owned(), Box::new(IrType::F64))?;
        return Ok((left_instrs, right_instrs, unary_left, Src::Var(right_dest)));
    }
    if *right_type == IrType::F64 {
        let left_dest = prog.new_var(unary_left.get_value_type());
        match *left_type {
            IrType::I32 => {
                left_instrs.push(Instruction::I32toF64(left_dest.to_owned(), unary_left))
            }
            IrType::U32 => {
                left_instrs.push(Instruction::U32toF64(left_dest.to_owned(), unary_left))
            }
            IrType::I64 => {
                left_instrs.push(Instruction::I64toF64(left_dest.to_owned(), unary_left))
            }
            IrType::U64 => {
                left_instrs.push(Instruction::U64toF64(left_dest.to_owned(), unary_left))
            }
            IrType::F32 => {
                left_instrs.push(Instruction::F32toF64(left_dest.to_owned(), unary_left))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(left_dest.to_owned(), Box::new(IrType::F64))?;
        return Ok((left_instrs, right_instrs, Src::Var(left_dest), unary_right));
    }

    // if one operand is a float
    if *left_type == IrType::F32 {
        let right_dest = prog.new_var(unary_right.get_value_type());
        match *right_type {
            IrType::I32 => {
                right_instrs.push(Instruction::I32toF32(right_dest.to_owned(), unary_right))
            }
            IrType::U32 => {
                right_instrs.push(Instruction::U32toF32(right_dest.to_owned(), unary_right))
            }
            IrType::I64 => {
                right_instrs.push(Instruction::I64toF32(right_dest.to_owned(), unary_right))
            }
            IrType::U64 => {
                right_instrs.push(Instruction::U64toF32(right_dest.to_owned(), unary_right))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(right_dest.to_owned(), Box::new(IrType::F32))?;
        return Ok((left_instrs, right_instrs, unary_left, Src::Var(right_dest)));
    }
    if *right_type == IrType::F32 {
        let left_dest = prog.new_var(unary_left.get_value_type());
        match *left_type {
            IrType::I32 => {
                left_instrs.push(Instruction::I32toF32(left_dest.to_owned(), unary_left))
            }
            IrType::U32 => {
                left_instrs.push(Instruction::U32toF32(left_dest.to_owned(), unary_left))
            }
            IrType::I64 => {
                left_instrs.push(Instruction::I64toF32(left_dest.to_owned(), unary_left))
            }
            IrType::U64 => {
                left_instrs.push(Instruction::U64toF32(left_dest.to_owned(), unary_left))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(left_dest.to_owned(), Box::new(IrType::F32))?;
        return Ok((left_instrs, right_instrs, Src::Var(left_dest), unary_right));
    }

    // if one operand is an unsigned long
    if *left_type == IrType::U64 {
        let right_dest = prog.new_var(unary_right.get_value_type());
        match *right_type {
            IrType::I32 => {
                right_instrs.push(Instruction::I32toU64(right_dest.to_owned(), unary_right))
            }
            IrType::U32 => {
                right_instrs.push(Instruction::U32toU64(right_dest.to_owned(), unary_right))
            }
            IrType::I64 => {
                right_instrs.push(Instruction::I64toU64(right_dest.to_owned(), unary_right))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(right_dest.to_owned(), Box::new(IrType::U64))?;
        return Ok((left_instrs, right_instrs, unary_left, Src::Var(right_dest)));
    }
    if *right_type == IrType::U64 {
        let left_dest = prog.new_var(unary_left.get_value_type());
        match *left_type {
            IrType::I32 => {
                left_instrs.push(Instruction::I32toU64(left_dest.to_owned(), unary_left))
            }
            IrType::U32 => {
                left_instrs.push(Instruction::U32toU64(left_dest.to_owned(), unary_left))
            }
            IrType::I64 => {
                left_instrs.push(Instruction::I64toU64(left_dest.to_owned(), unary_left))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(left_dest.to_owned(), Box::new(IrType::U64))?;
        return Ok((left_instrs, right_instrs, Src::Var(left_dest), unary_right));
    }

    // if one operand is a long
    if *left_type == IrType::I64 {
        let right_dest = prog.new_var(unary_right.get_value_type());
        match *right_type {
            IrType::I32 => {
                right_instrs.push(Instruction::I32toI64(right_dest.to_owned(), unary_right))
            }
            IrType::U32 => {
                right_instrs.push(Instruction::U32toI64(right_dest.to_owned(), unary_right))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(right_dest.to_owned(), Box::new(IrType::I64))?;
        return Ok((left_instrs, right_instrs, unary_left, Src::Var(right_dest)));
    }
    if *right_type == IrType::I64 {
        let left_dest = prog.new_var(unary_left.get_value_type());
        match *left_type {
            IrType::I32 => {
                left_instrs.push(Instruction::I32toI64(left_dest.to_owned(), unary_left))
            }
            IrType::U32 => {
                left_instrs.push(Instruction::U32toI64(left_dest.to_owned(), unary_left))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(left_dest.to_owned(), Box::new(IrType::I64))?;
        return Ok((left_instrs, right_instrs, Src::Var(left_dest), unary_right));
    }

    // if one operand is an unsigned int
    if *left_type == IrType::U32 {
        let right_dest = prog.new_var(unary_right.get_value_type());
        match *right_type {
            IrType::I32 => {
                right_instrs.push(Instruction::I32toU32(right_dest.to_owned(), unary_right))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(right_dest.to_owned(), Box::new(IrType::U32))?;
        return Ok((left_instrs, right_instrs, unary_left, Src::Var(right_dest)));
    }
    if *right_type == IrType::U32 {
        let left_dest = prog.new_var(unary_left.get_value_type());
        match *left_type {
            IrType::I32 => {
                left_instrs.push(Instruction::I32toU32(left_dest.to_owned(), unary_left))
            }
            _ => unreachable!(),
        }
        prog.add_var_type(left_dest.to_owned(), Box::new(IrType::U32))?;
        return Ok((left_instrs, right_instrs, Src::Var(left_dest), unary_right));
    }

    unreachable!("No other possible combinations of types left");
}

pub fn convert_type_for_assignment(
    src: Src,
    src_type: Box<IrType>,
    dest_type: Box<IrType>,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    trace!("convert {}: {} to {}", src, src_type, dest_type);
    let (convert_instrs, converted_var) =
        get_type_conversion_instrs(src, src_type, dest_type, prog)?;
    Ok((convert_instrs, converted_var))
}

pub fn get_type_conversion_instrs(
    src: Src,
    src_type: Box<IrType>,
    dest_type: Box<IrType>,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    trace!("convert {}: {} to {}", src, src_type, dest_type);
    let mut instrs = Vec::new();
    if src_type == dest_type {
        return Ok((instrs, src));
    }
    match (*src_type, *dest_type) {
        // char promotions
        (IrType::I8, dest_type) => {
            let intermediate_var = prog.new_var(src.get_value_type());
            let intermediate_type;
            if dest_type.is_signed_integral() {
                instrs.push(Instruction::I8toI16(intermediate_var.to_owned(), src));
                intermediate_type = IrType::I16;
            } else {
                // unsigned
                instrs.push(Instruction::I8toU16(intermediate_var.to_owned(), src));
                intermediate_type = IrType::U16;
            }
            prog.add_var_type(
                intermediate_var.to_owned(),
                Box::new(intermediate_type.to_owned()),
            )?;
            let (mut convert_instrs, dest) = get_type_conversion_instrs(
                Src::Var(intermediate_var),
                Box::new(intermediate_type),
                Box::new(dest_type),
                prog,
            )?;
            instrs.append(&mut convert_instrs);
            Ok((instrs, dest))
        }
        // (IrType::U8, IrType::I8) => {
        //     let dest = prog.new_var(src.get_value_type());
        //     instrs.push(Instruction::U8toI8(dest.to_owned(), src));
        //     prog.add_var_type(dest.to_owned, Box::new(dest.to_owned()))?;
        //     Ok((instrs, dest))
        // }
        (IrType::U8, dest_type) => {
            let intermediate_var = prog.new_var(src.get_value_type());
            let intermediate_type;
            if dest_type.is_signed_integral() {
                instrs.push(Instruction::U8toI16(intermediate_var.to_owned(), src));
                intermediate_type = IrType::I16;
            } else {
                // unsigned
                instrs.push(Instruction::U8toU16(intermediate_var.to_owned(), src));
                intermediate_type = IrType::U16;
            }
            prog.add_var_type(
                intermediate_var.to_owned(),
                Box::new(intermediate_type.to_owned()),
            )?;
            let (mut convert_instrs, dest) = get_type_conversion_instrs(
                Src::Var(intermediate_var),
                Box::new(intermediate_type),
                Box::new(dest_type),
                prog,
            )?;
            instrs.append(&mut convert_instrs);
            Ok((instrs, dest))
        }
        (IrType::I16, dest_type) => {
            let intermediate_var = prog.new_var(src.get_value_type());
            let intermediate_type;
            if dest_type.is_signed_integral() {
                instrs.push(Instruction::I16toI32(intermediate_var.to_owned(), src));
                intermediate_type = IrType::I32;
            } else {
                // unsigned
                instrs.push(Instruction::I16toU32(intermediate_var.to_owned(), src));
                intermediate_type = IrType::U32;
            }
            prog.add_var_type(
                intermediate_var.to_owned(),
                Box::new(intermediate_type.to_owned()),
            )?;
            let (mut convert_instrs, dest) = get_type_conversion_instrs(
                Src::Var(intermediate_var),
                Box::new(intermediate_type),
                Box::new(dest_type),
                prog,
            )?;
            instrs.append(&mut convert_instrs);
            Ok((instrs, dest))
        }
        (IrType::U16, dest_type) => {
            let intermediate_var = prog.new_var(src.get_value_type());
            let intermediate_type;
            if dest_type.is_signed_integral() {
                instrs.push(Instruction::U16toI32(intermediate_var.to_owned(), src));
                intermediate_type = IrType::I32;
            } else {
                // unsigned
                instrs.push(Instruction::U16toU32(intermediate_var.to_owned(), src));
                intermediate_type = IrType::U32;
            }
            prog.add_var_type(
                intermediate_var.to_owned(),
                Box::new(intermediate_type.to_owned()),
            )?;
            let (mut convert_instrs, dest) = get_type_conversion_instrs(
                Src::Var(intermediate_var),
                Box::new(intermediate_type),
                Box::new(dest_type),
                prog,
            )?;
            instrs.append(&mut convert_instrs);
            Ok((instrs, dest))
        }
        (IrType::I32, IrType::I8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I8))?;
            instrs.push(Instruction::I32toI8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I32, IrType::U8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U8))?;
            instrs.push(Instruction::I32toU8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I32, IrType::U32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U32))?;
            instrs.push(Instruction::I32toU32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I32, IrType::I64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I64))?;
            instrs.push(Instruction::I32toI64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I32, IrType::U64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U64))?;
            instrs.push(Instruction::I32toU64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I32, IrType::F32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F32))?;
            instrs.push(Instruction::I32toF32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I32, IrType::F64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F64))?;
            instrs.push(Instruction::I32toF64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        // cast to void *
        (IrType::I32, IrType::PointerTo(t)) if *t == IrType::Void => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(
                dest.to_owned(),
                Box::new(IrType::PointerTo(Box::new(IrType::Void))),
            )?;
            instrs.push(Instruction::I32toPtr(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }

        (IrType::U32, IrType::I8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I8))?;
            instrs.push(Instruction::U32toI8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U32, IrType::U8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U8))?;
            instrs.push(Instruction::U32toU8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U32, IrType::I64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I64))?;
            instrs.push(Instruction::U32toI64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U32, IrType::U64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U64))?;
            instrs.push(Instruction::U32toU64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U32, IrType::F32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F32))?;
            instrs.push(Instruction::U32toF32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U32, IrType::F64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F64))?;
            instrs.push(Instruction::U32toF64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        // cast to void *
        (IrType::U32, IrType::PointerTo(t)) if *t == IrType::Void => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(
                dest.to_owned(),
                Box::new(IrType::PointerTo(Box::new(IrType::Void))),
            )?;
            instrs.push(Instruction::U32toPtr(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }

        (IrType::I64, IrType::I8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I8))?;
            instrs.push(Instruction::I64toI8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I64, IrType::U8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U8))?;
            instrs.push(Instruction::I64toU8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I64, IrType::I32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
            instrs.push(Instruction::I64toI32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I64, IrType::U64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U64))?;
            instrs.push(Instruction::I64toU64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I64, IrType::F32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F32))?;
            instrs.push(Instruction::I64toF32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::I64, IrType::F64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F64))?;
            instrs.push(Instruction::I64toF64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }

        (IrType::U64, IrType::I8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I8))?;
            instrs.push(Instruction::U64toI8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U64, IrType::U8) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::U8))?;
            instrs.push(Instruction::U64toU8(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U64, IrType::I32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
            instrs.push(Instruction::U64toI32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U64, IrType::F32) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F32))?;
            instrs.push(Instruction::U64toF32(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (IrType::U64, IrType::F64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F64))?;
            instrs.push(Instruction::U64toF64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }

        (IrType::F32, IrType::F64) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::F64))?;
            instrs.push(Instruction::F32toF64(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }

        (IrType::Function(_, _, _), IrType::PointerTo(t))
        | (IrType::ArrayOf(_, _), IrType::PointerTo(t)) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(t)))?;
            instrs.push(Instruction::AddressOf(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        // void * to other pointer
        (IrType::PointerTo(t1), IrType::PointerTo(t2)) if *t1 == IrType::Void => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(t2)))?;
            instrs.push(Instruction::SimpleAssignment(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (s, d) => {
            return Err(MiddleEndError::TypeConversionError(
                "Cannot convert type",
                Box::new(s),
                Some(Box::new(d)),
            ))
        } // todo rest of types
    }
}
