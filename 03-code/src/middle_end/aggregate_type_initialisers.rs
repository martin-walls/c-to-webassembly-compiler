use crate::middle_end::ast_to_ir::convert_expression_to_ir;
use crate::middle_end::context::Context;
use crate::middle_end::ids::{ValueType, VarId};
use crate::middle_end::instructions::{Constant, Instruction, Src};
use crate::middle_end::ir::Program;
use crate::middle_end::ir_types::{IrType, TypeSize};
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::middle_end::type_conversions::convert_type_for_assignment;
use crate::parser::ast::{Constant as AstConstant, Expression, Initialiser};

pub fn array_initialiser(
    dest: VarId,
    dest_type_info: IrType,
    initialiser_list: Vec<Initialiser>,
    prog: &mut Program,
    context: &mut Context,
) -> Result<Vec<Instruction>, MiddleEndError> {
    let mut instrs = Vec::new();

    let array_member_type = dest_type_info.unwrap_array_type()?;
    // sizes of array members must be known at compile time
    let array_member_byte_size = match array_member_type.get_byte_size(&prog.program_metadata) {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => return Err(MiddleEndError::ArrayMemberSizeNotKnownAtCompileTime),
    };

    // pointer to the array member we're currently initialising
    let member_ptr_var = prog.new_var(ValueType::LValue);
    prog.add_var_type(
        member_ptr_var.to_owned(),
        IrType::PointerTo(Box::new(array_member_type.to_owned())),
    )?;
    instrs.push(Instruction::SimpleAssignment(
        prog.new_instr_id(),
        member_ptr_var.to_owned(),
        Src::Var(dest),
    ));

    // array length should be known at compile time (either explicitly, or inferred
    // from initialiser list)
    let array_size = match dest_type_info.get_array_size()? {
        TypeSize::CompileTime(size) => size,
        TypeSize::Runtime(_) => return Err(MiddleEndError::UndefinedArraySize),
    };
    // check that the array length matches the number of initialisers
    if array_size as usize != initialiser_list.len() {
        return Err(MiddleEndError::MismatchedArrayInitialiserLength);
    }

    for array_member_initialiser in initialiser_list {
        match array_member_initialiser {
            Initialiser::Expr(e) => {
                if array_member_type.is_aggregate_type() {
                    return Err(MiddleEndError::AssignNonAggregateValueToAggregateType);
                }
                let (mut expr_instrs, mut expr_var) = convert_expression_to_ir(e, prog, context)?;
                instrs.append(&mut expr_instrs);

                // check type of the expression and convert if necessary
                let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
                if expr_var_type != array_member_type {
                    if let Src::Constant(c) = &expr_var {
                        let temp = prog.new_var(ValueType::RValue);
                        prog.add_var_type(
                            temp.to_owned(),
                            c.get_type(Some(array_member_type.to_owned())),
                        )?;
                        instrs.push(Instruction::SimpleAssignment(
                            prog.new_instr_id(),
                            temp.to_owned(),
                            expr_var,
                        ));
                        expr_var = Src::Var(temp);
                    }
                    let (mut convert_instrs, converted_var) = convert_type_for_assignment(
                        expr_var.to_owned(),
                        expr_var.get_type(&prog.program_metadata)?,
                        array_member_type.to_owned(),
                        prog,
                    )?;
                    instrs.append(&mut convert_instrs);
                    expr_var = converted_var;
                }

                instrs.push(Instruction::StoreToAddress(
                    prog.new_instr_id(),
                    member_ptr_var.to_owned(),
                    expr_var,
                ));
            }
            Initialiser::List(sub_member_initialisers) => match array_member_type.to_owned() {
                IrType::ArrayOf(sub_member_type, size) => {
                    // initialise nested array
                    let mut init_instrs = array_initialiser(
                        member_ptr_var.to_owned(),
                        IrType::ArrayOf(sub_member_type, size),
                        sub_member_initialisers,
                        prog,
                        context,
                    )?;
                    instrs.append(&mut init_instrs);
                }
                IrType::Struct(struct_id) => {
                    // initialise nested struct
                    let mut init_instrs = struct_initialiser(
                        member_ptr_var.to_owned(),
                        IrType::Struct(struct_id),
                        sub_member_initialisers,
                        prog,
                        context,
                    )?;
                    instrs.append(&mut init_instrs);
                }
                _ => return Err(MiddleEndError::InvalidInitialiserExpression),
            },
        }
        // increment pointer to the next member
        instrs.push(Instruction::Add(
            prog.new_instr_id(),
            member_ptr_var.to_owned(),
            Src::Var(member_ptr_var.to_owned()),
            Src::Constant(Constant::Int(array_member_byte_size as i128)),
        ));
    }

    Ok(instrs)
}

pub fn struct_initialiser(
    dest: VarId,
    dest_type_info: IrType,
    initialiser_list: Vec<Initialiser>,
    prog: &mut Program,
    context: &mut Context,
) -> Result<Vec<Instruction>, MiddleEndError> {
    let mut instrs = Vec::new();

    let struct_type = dest_type_info.unwrap_struct_type(prog)?;

    // check that the number of initialisers matches the number of struct members
    if struct_type.member_count() != initialiser_list.len() {
        return Err(MiddleEndError::MismatchedArrayInitialiserLength);
    }

    for member_index in 0..struct_type.member_count() {
        let mut member_initialiser = initialiser_list.get(member_index).unwrap().to_owned();
        let member_type = struct_type.get_member_type_by_index(member_index)?;
        let member_byte_offset = struct_type.get_member_byte_offset_by_index(member_index)?;

        // pointer to the struct member we're currently initialising
        let member_ptr_var = prog.new_var(ValueType::LValue);
        prog.add_var_type(
            member_ptr_var.to_owned(),
            IrType::PointerTo(Box::new(member_type.to_owned())),
        )?;
        // member_ptr_var = &dest + byte_offset
        instrs.push(Instruction::AddressOf(
            prog.new_instr_id(),
            member_ptr_var.to_owned(),
            Src::Var(dest.to_owned()),
        ));
        instrs.push(Instruction::Add(
            prog.new_instr_id(),
            member_ptr_var.to_owned(),
            Src::Var(member_ptr_var.to_owned()),
            Src::Constant(Constant::Int(member_byte_offset as i128)),
        ));

        // check for case of initialising a char array with a string literal
        if member_type.is_array_type() {
            match member_initialiser.to_owned() {
                Initialiser::Expr(e) => {
                    if let Expression::StringLiteral(s) = e.to_owned() {
                        // convert string literal to array of chars
                        member_initialiser = convert_string_literal_to_init_list_of_chars_ast(s);
                    }
                }
                Initialiser::List(inits) => {
                    if inits.len() == 1 {
                        if let Initialiser::Expr(e) = inits.first().unwrap() {
                            if let Expression::StringLiteral(s) = e.to_owned() {
                                // convert string literal in braces to array of chars
                                member_initialiser =
                                    convert_string_literal_to_init_list_of_chars_ast(s);
                            }
                        }
                    }
                }
            }
        }

        match member_initialiser {
            Initialiser::Expr(e) => {
                if member_type.is_aggregate_type() {
                    return Err(MiddleEndError::AssignNonAggregateValueToAggregateType);
                }

                let (mut expr_instrs, mut expr_var) = convert_expression_to_ir(e, prog, context)?;
                instrs.append(&mut expr_instrs);

                // check type of the expression and convert if necessary
                let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
                if expr_var_type != member_type {
                    if let Src::Constant(c) = &expr_var {
                        let temp = prog.new_var(ValueType::RValue);
                        prog.add_var_type(
                            temp.to_owned(),
                            c.get_type(Some(member_type.to_owned())),
                        )?;
                        instrs.push(Instruction::SimpleAssignment(
                            prog.new_instr_id(),
                            temp.to_owned(),
                            expr_var,
                        ));
                        expr_var = Src::Var(temp);
                    }
                    let (mut convert_instrs, converted_var) = convert_type_for_assignment(
                        expr_var.to_owned(),
                        expr_var.get_type(&prog.program_metadata)?,
                        member_type.to_owned(),
                        prog,
                    )?;
                    instrs.append(&mut convert_instrs);
                    expr_var = converted_var;
                }

                instrs.push(Instruction::StoreToAddress(
                    prog.new_instr_id(),
                    member_ptr_var,
                    expr_var,
                ));
            }
            Initialiser::List(sub_member_initialisers) => match member_type.to_owned() {
                IrType::ArrayOf(sub_member_type, size) => {
                    // initialise nested array
                    let mut init_instrs = array_initialiser(
                        member_ptr_var,
                        IrType::ArrayOf(sub_member_type, size),
                        sub_member_initialisers,
                        prog,
                        context,
                    )?;
                    instrs.append(&mut init_instrs);
                }
                IrType::Struct(struct_id) => {
                    // initialise nested struct
                    let mut init_instrs = struct_initialiser(
                        member_ptr_var,
                        IrType::Struct(struct_id),
                        sub_member_initialisers,
                        prog,
                        context,
                    )?;
                    instrs.append(&mut init_instrs);
                }
                _ => return Err(MiddleEndError::InvalidInitialiserExpression),
            },
        }
    }

    Ok(instrs)
}

/// convert a string literal to an array of chars, for array initialiser
pub fn convert_string_literal_to_init_list_of_chars_ast(s: String) -> Initialiser {
    let mut char_initialisers = Vec::new();
    for c in s.chars() {
        char_initialisers.push(Initialiser::Expr(Expression::Constant(AstConstant::Char(
            c,
        ))));
    }
    // string terminating char
    char_initialisers.push(Initialiser::Expr(Expression::Constant(AstConstant::Char(
        '\0',
    ))));
    Initialiser::List(char_initialisers)
}
