use log::{debug, trace};

use crate::middle_end::aggregate_type_initialisers::{
    array_initialiser, convert_string_literal_to_init_list_of_chars_ast, struct_initialiser,
};
use crate::middle_end::context::{Context, IdentifierResolveResult, LoopContext, SwitchContext};
use crate::middle_end::get_ast_type_info::get_type_info;
use crate::middle_end::ids::{ValueType, VarId};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::instructions::{Constant, Src};
use crate::middle_end::ir::{Function, Program};
use crate::middle_end::ir_types::{array_to_pointer_type, IrType, TypeSize};
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::middle_end::type_conversions::{
    binary_convert, binary_convert_separately, convert_type_for_assignment,
    get_type_conversion_instrs, unary_convert,
};
use crate::parser::ast;
use crate::parser::ast::{
    BinaryOperator, DeclaratorInitialiser, Expression, ExpressionOrDeclaration, Identifier,
    Initialiser, LabelledStatement, Program as AstProgram, Statement, TypeSpecifier, UnaryOperator,
};

pub fn convert_to_ir(ast: AstProgram) -> Result<Box<Program>, MiddleEndError> {
    let mut prog = Box::new(Program::new());
    let mut context = Box::new(Context::new());
    for stmt in ast.0 {
        let global_instrs = convert_statement_to_ir(stmt, &mut prog, &mut context);
        match global_instrs {
            Ok(mut instrs) => prog.program_instructions.global_instrs.append(&mut instrs),
            Err(e) => return Err(e),
        }
    }
    Ok(prog)
}

fn convert_statement_to_ir(
    stmt: Box<Statement>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<Vec<Instruction>, MiddleEndError> {
    let mut instrs: Vec<Instruction> = Vec::new();
    match *stmt {
        Statement::Block(stmts) => {
            context.push_scope();
            for s in stmts {
                instrs.append(&mut convert_statement_to_ir(s, prog, context)?);
            }
            context.pop_scope();
        }
        Statement::Goto(x) => {
            let label = match prog.resolve_identifier_to_label(&x.0) {
                Some(label) => label.to_owned(),
                None => prog.new_identifier_label(x.0),
            };
            instrs.push(Instruction::Br(prog.new_instr_id(), label));
        }
        Statement::Continue => match context.get_continue_label() {
            None => {
                return Err(MiddleEndError::ContinueOutsideLoopContext);
            }
            Some(label) => {
                instrs.push(Instruction::Br(prog.new_instr_id(), label.to_owned()));
            }
        },
        Statement::Break => match context.get_break_label() {
            None => {
                return Err(MiddleEndError::BreakOutsideLoopOrSwitchContext);
            }
            Some(label) => {
                instrs.push(Instruction::Br(prog.new_instr_id(), label.to_owned()));
            }
        },
        Statement::Return(expr) => match expr {
            None => {
                instrs.push(Instruction::Ret(prog.new_instr_id(), None));
            }
            Some(expr) => {
                let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
                instrs.append(&mut expr_instrs);
                instrs.push(Instruction::Ret(prog.new_instr_id(), Some(expr_var)));
            }
        },
        Statement::While(cond, body) => {
            let loop_start_label = prog.new_label();
            let loop_end_label = prog.new_label();
            // start of loop label
            instrs.push(Instruction::Label(
                prog.new_instr_id(),
                loop_start_label.to_owned(),
            ));
            context.push_loop(LoopContext::while_loop(
                loop_start_label.to_owned(),
                loop_end_label.to_owned(),
            ));
            // while condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // jump out of loop if condition false
            instrs.push(Instruction::BrIfEq(
                prog.new_instr_id(),
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_end_label.to_owned(),
            ));
            // loop body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // jump back to start of loop to evaluate condition again
            instrs.push(Instruction::Br(prog.new_instr_id(), loop_start_label));
            instrs.push(Instruction::Label(prog.new_instr_id(), loop_end_label));
            context.pop_loop();
        }
        Statement::DoWhile(body, cond) => {
            let loop_start_label = prog.new_label();
            let loop_end_label = prog.new_label();
            let loop_continue_label = prog.new_label();
            // start of loop label
            instrs.push(Instruction::Label(
                prog.new_instr_id(),
                loop_start_label.to_owned(),
            ));
            context.push_loop(LoopContext::do_while_loop(
                loop_start_label.to_owned(),
                loop_end_label.to_owned(),
                loop_continue_label.to_owned(),
            ));
            // loop body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // continue label
            instrs.push(Instruction::Label(prog.new_instr_id(), loop_continue_label));
            // loop condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // jump back to start of loop if condition true

            instrs.push(Instruction::BrIfNotEq(
                prog.new_instr_id(),
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_start_label,
            ));
            // end of loop
            instrs.push(Instruction::Label(prog.new_instr_id(), loop_end_label));
            context.pop_loop();
        }
        Statement::For(init, cond, end, body) => {
            let loop_start_label = prog.new_label();
            let loop_end_label = prog.new_label();
            let loop_continue_label = prog.new_label();
            // optional initialiser statement - runs before the loop starts
            match init {
                None => {}
                Some(e_or_d) => match e_or_d {
                    ExpressionOrDeclaration::Expression(e) => {
                        let (mut expr_instrs, _) = convert_expression_to_ir(e, prog, context)?;
                        instrs.append(&mut expr_instrs);
                    }
                    ExpressionOrDeclaration::Declaration(d) => {
                        instrs.append(&mut convert_statement_to_ir(d, prog, context)?);
                    }
                },
            }
            // start of loop label
            instrs.push(Instruction::Label(
                prog.new_instr_id(),
                loop_start_label.to_owned(),
            ));
            context.push_loop(LoopContext::for_loop(
                loop_start_label.to_owned(),
                loop_end_label.to_owned(),
                loop_continue_label.to_owned(),
            ));
            // condition
            let cond_var = match cond {
                None => {
                    let temp = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        temp.to_owned(),
                        Src::Constant(Constant::Int(1)),
                    ));
                    prog.add_var_type(temp.to_owned(), Box::new(IrType::I32))?;
                    Src::Var(temp)
                }
                Some(e) => {
                    let (mut expr_instrs, expr_var) = convert_expression_to_ir(e, prog, context)?;
                    instrs.append(&mut expr_instrs);
                    expr_var
                }
            };
            instrs.push(Instruction::BrIfEq(
                prog.new_instr_id(),
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_end_label.to_owned(),
            ));
            // loop body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // continue label
            instrs.push(Instruction::Label(prog.new_instr_id(), loop_continue_label));
            // end-of-loop expression, before looping back to condition again
            match end {
                None => {}
                Some(e) => {
                    let (mut expr_instrs, _) = convert_expression_to_ir(e, prog, context)?;
                    instrs.append(&mut expr_instrs);
                }
            }
            // loop back to condition
            instrs.push(Instruction::Br(prog.new_instr_id(), loop_start_label));
            // end of loop label
            instrs.push(Instruction::Label(prog.new_instr_id(), loop_end_label));
            context.pop_loop();
        }
        Statement::If(cond, body) => {
            // if statement condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // if condition is false, jump to after body
            let if_end_label = prog.new_label();
            instrs.push(Instruction::BrIfEq(
                prog.new_instr_id(),
                cond_var,
                Src::Constant(Constant::Int(0)),
                if_end_label.to_owned(),
            ));
            // if statement body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // end of if statement label
            instrs.push(Instruction::Label(prog.new_instr_id(), if_end_label));
        }
        Statement::IfElse(cond, true_body, false_body) => {
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // if condition is false, jump to else body
            let else_label = prog.new_label();
            instrs.push(Instruction::BrIfEq(
                prog.new_instr_id(),
                cond_var,
                Src::Constant(Constant::Int(0)),
                else_label.to_owned(),
            ));
            // if body
            instrs.append(&mut convert_statement_to_ir(true_body, prog, context)?);
            // jump to after else body
            let else_end_label = prog.new_label();
            instrs.push(Instruction::Br(
                prog.new_instr_id(),
                else_end_label.to_owned(),
            ));
            // else body
            instrs.push(Instruction::Label(prog.new_instr_id(), else_label));
            instrs.append(&mut convert_statement_to_ir(false_body, prog, context)?);
            instrs.push(Instruction::Label(prog.new_instr_id(), else_end_label));
        }
        Statement::Switch(switch_expr, body) => {
            let switch_end_label = prog.new_label();
            let (mut expr_instrs, switch_src) =
                convert_expression_to_ir(switch_expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let switch_var = match switch_src {
                Src::Var(var) => var,
                Src::Constant(c) => {
                    let temp = prog.new_var(ValueType::RValue);
                    prog.add_var_type(temp.to_owned(), c.get_type(None))?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        temp.to_owned(),
                        Src::Constant(c),
                    ));
                    temp
                }
                Src::Fun(_) | Src::StoreAddressVar(_) => unreachable!(),
            };
            context.push_switch(SwitchContext::new(switch_end_label.to_owned(), switch_var));
            // convert switch body - ignore the return, because the instrs will
            // be stored into the SwitchContext, and we'll get them from there after
            convert_statement_to_ir(body, prog, context)?;

            let mut switch_context = context.pop_switch()?;

            // add case comparison instructions
            instrs.append(&mut switch_context.case_condition_instrs);
            // if we have a default block, then jump there unconditionally after
            // checking all the other conditions. If no default block, then exit
            // the switch (ie. if no cases match)
            match switch_context.default_block_label {
                Some(label) => {
                    instrs.push(Instruction::Br(prog.new_instr_id(), label));
                }
                None => {
                    instrs.push(Instruction::Br(
                        prog.new_instr_id(),
                        switch_end_label.to_owned(),
                    ));
                }
            }

            // add case bodies
            for mut case_body_instrs in switch_context.case_blocks {
                instrs.append(&mut case_body_instrs);
            }

            // end of switch label
            instrs.push(Instruction::Label(prog.new_instr_id(), switch_end_label));
        }
        Statement::Labelled(stmt) => {
            match stmt {
                LabelledStatement::Named(Identifier(label_name), stmt) => {
                    let label = prog.new_identifier_label(label_name);
                    instrs.push(Instruction::Label(prog.new_instr_id(), label));
                    instrs.append(&mut convert_statement_to_ir(stmt, prog, context)?);
                }
                LabelledStatement::Case(expr, stmt) => {
                    // case statements are only allowed in a switch context
                    if !context.is_in_switch_context() {
                        return Err(MiddleEndError::CaseOutsideSwitchContext);
                    }
                    let (mut condition_instrs, expr_var) =
                        convert_expression_to_ir(expr, prog, context)?;
                    let case_body_label = prog.new_label();
                    // check if case condition matches the switch expression
                    // if so, jump to the corresponding block
                    condition_instrs.push(Instruction::BrIfEq(
                        prog.new_instr_id(),
                        Src::Var(context.get_switch_variable().unwrap()),
                        expr_var,
                        case_body_label.to_owned(),
                    ));
                    context.new_switch_case_block(
                        case_body_label,
                        condition_instrs,
                        &mut prog.program_metadata,
                    )?;
                    // start of case body - the result of this will be automatically pushed to
                    // the case block we just created, because we're in a switch context
                    convert_statement_to_ir(stmt, prog, context)?;
                    return Ok(instrs);
                }
                LabelledStatement::Default(stmt) => {
                    let case_body_label = prog.new_label();
                    context.add_default_switch_block_label(case_body_label.to_owned())?;
                    // default case has no condition instruction to add, because it'll get added
                    // at the end of converting the whole switch statement
                    context.new_switch_case_block(
                        case_body_label,
                        Vec::new(),
                        &mut prog.program_metadata,
                    )?;
                    convert_statement_to_ir(stmt, prog, context)?;
                    return Ok(instrs);
                }
            }
        }
        Statement::Expr(e) => {
            let (mut expr_instrs, _) = convert_expression_to_ir(e, prog, context)?;
            instrs.append(&mut expr_instrs);
        }
        Statement::Declaration(sq, declarators) => {
            let mut is_initial_declarator = true;
            for declarator in declarators {
                match declarator {
                    DeclaratorInitialiser::NoInit(d) => {
                        match get_type_info(
                            &sq,
                            Some(d.to_owned()),
                            !is_initial_declarator,
                            prog,
                            context,
                        )? {
                            None => {
                                // typedef declaration, so no need to do anything more here
                            }
                            Some((type_info, name, Some(_params))) => match name {
                                // function declaration
                                Some(name) => {
                                    trace!("Function declaration: {}", name);
                                    let fun_declaration =
                                        Function::declaration(type_info.to_owned());
                                    let fun_id =
                                        prog.new_fun_declaration(name.to_owned(), fun_declaration)?;
                                    context.add_function_declaration(name, fun_id)?;
                                }
                                None => {
                                    // If declarator has no name, it should be a
                                    // Statement::EmptyDeclaration, so this case should never
                                    // be reached
                                    unreachable!()
                                }
                            },
                            Some((type_info, name, None)) => match name {
                                // non-function declaration (normal variable)
                                Some(name) => {
                                    trace!("Variable declaration: {}", name);
                                    let var = prog.new_var(ValueType::LValue);
                                    context.add_variable_to_scope(
                                        name,
                                        var.to_owned(),
                                        type_info.to_owned(),
                                    )?;
                                    prog.add_var_type(var.to_owned(), type_info.to_owned())?;

                                    // if we're declaring an array, allocate space at the end of the stack
                                    if type_info.is_array_type() {
                                        let array_byte_size = match type_info
                                            .get_array_byte_size(&prog.program_metadata)?
                                        {
                                            TypeSize::CompileTime(size) => {
                                                Src::Constant(Constant::Int(size as i128))
                                            }
                                            TypeSize::Runtime(size_expr) => {
                                                let (mut size_expr_instrs, size_var) =
                                                    convert_expression_to_ir(
                                                        size_expr, prog, context,
                                                    )?;
                                                instrs.append(&mut size_expr_instrs);
                                                size_var
                                            }
                                        };
                                        instrs.push(Instruction::AllocateVariable(
                                            prog.new_instr_id(),
                                            var,
                                            array_byte_size,
                                        ));
                                    } else {
                                        instrs.push(Instruction::DeclareVariable(
                                            prog.new_instr_id(),
                                            var,
                                        ));
                                    }
                                }
                                None => unreachable!(),
                            },
                        };
                    }
                    DeclaratorInitialiser::Init(d, mut init_expr) => {
                        let (dest_type_info, name, _) = match get_type_info(
                            &sq,
                            Some(d),
                            !is_initial_declarator,
                            prog,
                            context,
                        )? {
                            None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                            Some(x) => x,
                        };

                        // check for case of initialising a char array with a string literal
                        if let IrType::ArrayOf(_, _) = *dest_type_info {
                            match *init_expr.to_owned() {
                                Initialiser::Expr(e) => {
                                    if let Expression::StringLiteral(s) = *e.to_owned() {
                                        // convert string literal to array of chars
                                        init_expr =
                                            convert_string_literal_to_init_list_of_chars_ast(s);
                                    }
                                }
                                Initialiser::List(inits) => {
                                    if inits.len() == 1 {
                                        if let Initialiser::Expr(e) = &**inits.first().unwrap() {
                                            if let Expression::StringLiteral(s) = *e.to_owned() {
                                                // convert string literal in braces to array of chars
                                                init_expr = convert_string_literal_to_init_list_of_chars_ast(s);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // if array type, get array size from number of initialisers (recursively)
                        let dest_type_info =
                            dest_type_info.resolve_array_size_from_initialiser(&init_expr)?;

                        match name {
                            None => return Err(MiddleEndError::InvalidAbstractDeclarator),
                            Some(name) => {
                                match *init_expr {
                                    Initialiser::Expr(e) => {
                                        let (mut expr_instrs, expr_var) =
                                            convert_expression_to_ir(e, prog, context)?;
                                        instrs.append(&mut expr_instrs);
                                        let mut src = match expr_var {
                                            Src::Var(var) => Src::Var(var),
                                            Src::Constant(c) => Src::Constant(c),
                                            Src::Fun(_) | Src::StoreAddressVar(_) => {
                                                return Err(
                                                    MiddleEndError::InvalidInitialiserExpression,
                                                )
                                            }
                                        };

                                        // convert src to dest type
                                        let src_type = src.get_type(&prog.program_metadata)?;
                                        if src_type != dest_type_info {
                                            if let Src::Constant(c) = &src {
                                                let temp = prog.new_var(ValueType::RValue);
                                                prog.add_var_type(
                                                    temp.to_owned(),
                                                    c.get_type(Some(dest_type_info.to_owned())),
                                                )?;
                                                instrs.push(Instruction::SimpleAssignment(
                                                    prog.new_instr_id(),
                                                    temp.to_owned(),
                                                    src,
                                                ));
                                                src = Src::Var(temp);
                                            }
                                            let (mut convert_instrs, converted_var) =
                                                convert_type_for_assignment(
                                                    src.to_owned(),
                                                    src.get_type(&prog.program_metadata)?,
                                                    dest_type_info.to_owned(),
                                                    prog,
                                                )?;
                                            instrs.append(&mut convert_instrs);
                                            src = converted_var;
                                        }

                                        let dest = prog.new_var(ValueType::LValue);
                                        prog.add_var_type(
                                            dest.to_owned(),
                                            src.get_type(&prog.program_metadata)?,
                                        )?;
                                        instrs.push(Instruction::SimpleAssignment(
                                            prog.new_instr_id(),
                                            dest.to_owned(),
                                            src,
                                        ));
                                        context.add_variable_to_scope(
                                            name,
                                            dest.to_owned(),
                                            dest_type_info.to_owned(),
                                        )?;
                                    }
                                    Initialiser::List(initialisers) => match *dest_type_info {
                                        IrType::ArrayOf(member_type, size) => {
                                            let dest = prog.new_var(ValueType::LValue);
                                            let dest_type =
                                                Box::new(IrType::ArrayOf(member_type, size));
                                            prog.add_var_type(
                                                dest.to_owned(),
                                                dest_type.to_owned(),
                                            )?;
                                            context.add_variable_to_scope(
                                                name,
                                                dest.to_owned(),
                                                dest_type.to_owned(),
                                            )?;

                                            let array_byte_size = match dest_type
                                                .get_array_byte_size(&prog.program_metadata)?
                                            {
                                                TypeSize::CompileTime(size) => {
                                                    Src::Constant(Constant::Int(size as i128))
                                                }
                                                TypeSize::Runtime(size_expr) => {
                                                    let (mut size_expr_instrs, size_var) =
                                                        convert_expression_to_ir(
                                                            size_expr, prog, context,
                                                        )?;
                                                    instrs.append(&mut size_expr_instrs);
                                                    size_var
                                                }
                                            };
                                            instrs.push(Instruction::AllocateVariable(
                                                prog.new_instr_id(),
                                                dest.to_owned(),
                                                array_byte_size,
                                            ));

                                            let mut init_instrs = array_initialiser(
                                                dest,
                                                dest_type,
                                                initialisers,
                                                prog,
                                                context,
                                            )?;
                                            instrs.append(&mut init_instrs);
                                        }
                                        IrType::Struct(struct_id) => {
                                            let dest = prog.new_var(ValueType::LValue);
                                            let dest_type = Box::new(IrType::Struct(struct_id));
                                            prog.add_var_type(
                                                dest.to_owned(),
                                                dest_type.to_owned(),
                                            )?;
                                            context.add_variable_to_scope(
                                                name,
                                                dest.to_owned(),
                                                dest_type.to_owned(),
                                            )?;

                                            let byte_size = match dest_type
                                                .get_byte_size(&prog.program_metadata)
                                            {
                                                TypeSize::CompileTime(size) => {
                                                    Src::Constant(Constant::Int(size as i128))
                                                }
                                                TypeSize::Runtime(size_expr) => {
                                                    let (mut size_expr_instrs, size_var) =
                                                        convert_expression_to_ir(
                                                            size_expr, prog, context,
                                                        )?;
                                                    instrs.append(&mut size_expr_instrs);
                                                    size_var
                                                }
                                            };
                                            instrs.push(Instruction::AllocateVariable(
                                                prog.new_instr_id(),
                                                dest.to_owned(),
                                                byte_size,
                                            ));

                                            let mut init_instrs = struct_initialiser(
                                                dest,
                                                dest_type,
                                                initialisers,
                                                prog,
                                                context,
                                            )?;
                                            instrs.append(&mut init_instrs);
                                        }
                                        _ => {
                                            return Err(
                                                MiddleEndError::InvalidInitialiserExpression,
                                            )
                                        }
                                    },
                                };
                            }
                        }
                    }
                }
                is_initial_declarator = false;
            }
        }
        Statement::EmptyDeclaration(sq) => {
            match get_type_info(&sq, None, false, prog, context)? {
                None => {
                    // typedef declaration is invalid without a name
                    return Err(MiddleEndError::InvalidTypedefDeclaration);
                }
                Some((type_info, _name, _params)) => match *type_info {
                    IrType::I32 => match &sq.type_specifier {
                        TypeSpecifier::Enum(_) => {}
                        _ => return Err(MiddleEndError::InvalidDeclaration),
                    },
                    IrType::Struct(_) | IrType::Union(_) => {}
                    _ => return Err(MiddleEndError::InvalidDeclaration),
                },
            }
        }
        Statement::FunctionDeclaration(sq, decl, body) => {
            let (type_info, name, param_bindings) =
                match get_type_info(&sq, Some(decl), false, prog, context)? {
                    None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                    Some(x) => x,
                };
            let name = match name {
                None => return Err(MiddleEndError::InvalidFunctionDeclaration),
                Some(n) => n,
            };
            context.push_scope();
            // for each parameter, store which var it maps to
            let mut param_var_mappings: Vec<VarId> = Vec::new();
            // put parameter names into scope
            if let Some(param_bindings) = param_bindings {
                for (param_name, param_type) in param_bindings {
                    let param_var = prog.new_var(ValueType::LValue);
                    param_var_mappings.push(param_var.to_owned());
                    let param_type = array_to_pointer_type(param_type);
                    context.add_variable_to_scope(
                        param_name,
                        param_var.to_owned(),
                        param_type.to_owned(),
                    )?;
                    prog.add_var_type(param_var, param_type)?;
                }
            }
            // add function to context and prog before converting body, because might be recursive
            let fun_declaration = Function::declaration(type_info.to_owned());
            let fun_id = prog.new_fun_body(name.to_owned(), fun_declaration)?;
            context.add_function_declaration(name.to_owned(), fun_id)?;
            // function body instructions
            let instrs = convert_statement_to_ir(body, prog, context)?;
            // update function in program with full body
            let fun = Function::new(instrs, type_info, param_var_mappings);
            prog.new_fun_body(name, fun)?;
            context.pop_scope()
        }
        Statement::Empty => {}
    }

    if context.is_in_switch_context() {
        context.push_instrs_to_switch_case_block(instrs)?;
        return Ok(Vec::new());
    }

    Ok(instrs)
}

/// returns the list of instructions generated, and the name of the temp variable
/// the result is assigned to
pub fn convert_expression_to_ir(
    src_expr: Box<Expression>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    let mut instrs: Vec<Instruction> = Vec::new();
    // this flag should only ever persist one level deep
    let this_expr_directly_on_lhs_of_assignment = context.directly_on_lhs_of_assignment;
    context.directly_on_lhs_of_assignment = false;
    match *src_expr {
        Expression::Identifier(Identifier(name)) => {
            if context.in_function_name_expr {
                let fun = context.resolve_identifier_to_fun(&name)?;
                Ok((instrs, Src::Fun(fun)))
            } else {
                match context.resolve_identifier_to_var_or_const(&name)? {
                    IdentifierResolveResult::Var(var) => Ok((instrs, Src::Var(var))),
                    IdentifierResolveResult::EnumConst(c) => {
                        Ok((instrs, Src::Constant(Constant::Int(c as i128))))
                    }
                }
            }
        }
        Expression::Constant(c) => match c {
            ast::Constant::Int(i) => Ok((instrs, Src::Constant(Constant::Int(i as i128)))),
            ast::Constant::Float(f) => Ok((instrs, Src::Constant(Constant::Float(f)))),
            ast::Constant::Char(ch) => Ok((instrs, Src::Constant(Constant::Int(ch as i128)))),
        },
        Expression::StringLiteral(s) => {
            let string_id = prog.new_string_literal(s);
            let dest = prog.new_var(ValueType::LValue);
            instrs.push(Instruction::PointerToStringLiteral(
                prog.new_instr_id(),
                dest.to_owned(),
                string_id,
            ));
            // dest has char * type
            prog.add_var_type(
                dest.to_owned(),
                Box::new(IrType::PointerTo(Box::new(IrType::I8))),
            )?;
            Ok((instrs, Src::Var(dest)))
        }
        Expression::Index(arr, index) => {
            let (mut arr_instrs, arr_var) = convert_expression_to_ir(arr, prog, context)?;
            instrs.append(&mut arr_instrs);
            let (mut index_instrs, index_var) = convert_expression_to_ir(index, prog, context)?;
            instrs.append(&mut index_instrs);

            // unary conversion
            let (mut unary_convert_arr_instrs, arr_var) = unary_convert(arr_var, prog)?;
            instrs.append(&mut unary_convert_arr_instrs);
            let (mut unary_convert_index_instrs, mut index_var) = unary_convert(index_var, prog)?;
            instrs.append(&mut unary_convert_index_instrs);
            let arr_var_type = arr_var.get_type(&prog.program_metadata)?;
            arr_var_type.require_pointer_type()?;
            // the type of the actual array elements
            let arr_inner_type = arr_var_type.dereference_pointer_type()?;
            let index_var_type = index_var.get_type(&prog.program_metadata)?;
            index_var_type.require_integral_type()?;

            match *index_var_type {
                IrType::I64 => {
                    // index should be int not long, so we can add it to ptr
                    let temp_index_var = prog.new_var(index_var.get_value_type());
                    prog.add_var_type(temp_index_var.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::U64toI32(
                        prog.new_instr_id(),
                        temp_index_var.to_owned(),
                        index_var,
                    ));
                    index_var = Src::Var(temp_index_var);
                }
                IrType::U64 => {
                    // index should be int not long, so we can add it to ptr
                    let temp_index_var = prog.new_var(index_var.get_value_type());
                    prog.add_var_type(temp_index_var.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::I64toI32(
                        prog.new_instr_id(),
                        temp_index_var.to_owned(),
                        index_var,
                    ));
                    index_var = Src::Var(temp_index_var);
                }
                _ => {}
            }

            // multiply index by size of element, to get number of bytes to advance ptr by
            let element_byte_size = arr_inner_type.get_byte_size(&prog.program_metadata);
            let byte_size_var = match element_byte_size {
                TypeSize::CompileTime(byte_size) => Src::Constant(Constant::Int(byte_size as i128)),
                TypeSize::Runtime(byte_size_expr) => {
                    let (mut byte_size_instrs, byte_size_var) =
                        convert_expression_to_ir(byte_size_expr, prog, context)?;
                    instrs.append(&mut byte_size_instrs);
                    byte_size_var
                }
            };
            let (mut binary_convert_instrs, left_var, right_var) =
                binary_convert(index_var, byte_size_var, prog)?;
            instrs.append(&mut binary_convert_instrs);
            let ptr_offset_var = prog.new_var(ValueType::RValue);
            // left and right vars have same type cos of binary conversion
            let left_var_type = left_var.get_type(&prog.program_metadata)?;
            prog.add_var_type(ptr_offset_var.to_owned(), left_var_type)?;
            instrs.push(Instruction::Mult(
                prog.new_instr_id(),
                ptr_offset_var.to_owned(),
                left_var,
                right_var,
            ));

            // array variable is a pointer to the start of the array
            let ptr = prog.new_var(ValueType::LValue);
            prog.add_var_type(ptr.to_owned(), arr_var_type)?;
            instrs.push(Instruction::Add(
                prog.new_instr_id(),
                ptr.to_owned(),
                arr_var,
                Src::Var(ptr_offset_var),
            ));
            if this_expr_directly_on_lhs_of_assignment {
                // store to array index
                Ok((instrs, Src::StoreAddressVar(ptr)))
            } else {
                // read from array index
                let dest = prog.new_var(ValueType::LValue);
                prog.add_var_type(dest.to_owned(), arr_inner_type)?;
                instrs.push(Instruction::LoadFromAddress(
                    prog.new_instr_id(),
                    dest.to_owned(),
                    Src::Var(ptr),
                ));
                Ok((instrs, Src::Var(dest)))
            }
        }
        Expression::FunctionCall(fun, params) => {
            context.in_function_name_expr = true;
            let (mut fun_instrs, fun_var) = convert_expression_to_ir(fun, prog, context)?;
            instrs.append(&mut fun_instrs);
            context.in_function_name_expr = false;

            // unary conversion
            let (mut unary_convert_fun_instrs, fun_var) = unary_convert(fun_var, prog)?;
            instrs.append(&mut unary_convert_fun_instrs);
            // must be a function name to be able to be called
            let fun_id = fun_var.require_function_id()?;
            let dest_type = fun_var.get_function_return_type(prog)?;

            let mut param_srcs: Vec<Src> = Vec::new();
            for param in params {
                let (mut param_instrs, param_var) = convert_expression_to_ir(param, prog, context)?;
                // todo function parameter passing type conversions
                instrs.append(&mut param_instrs);
                param_srcs.push(param_var);
            }
            let dest = prog.new_var(ValueType::RValue);
            prog.add_var_type(dest.to_owned(), dest_type)?;
            instrs.push(Instruction::Call(
                prog.new_instr_id(),
                dest.to_owned(),
                fun_id,
                param_srcs,
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::DirectMemberSelection(obj, Identifier(member_name)) => {
            let (mut obj_instrs, obj_var) = convert_expression_to_ir(obj, prog, context)?;
            instrs.append(&mut obj_instrs);
            let obj_var_type = obj_var.get_type(&prog.program_metadata)?;
            obj_var_type.require_struct_or_union_type()?;

            // obj_ptr = &obj_var
            let obj_ptr = prog.new_var(obj_var.get_value_type());
            prog.add_var_type(
                obj_ptr.to_owned(),
                Box::new(IrType::PointerTo(obj_var_type.to_owned())),
            )?;
            instrs.push(Instruction::AddressOf(
                prog.new_instr_id(),
                obj_ptr.to_owned(),
                obj_var,
            ));

            match *obj_var_type {
                IrType::Struct(struct_id) => {
                    let struct_type = prog.get_struct_type(&struct_id)?;
                    let member_type = struct_type.get_member_type(&member_name)?;
                    let member_byte_offset = struct_type.get_member_byte_offset(&member_name)?;

                    let ptr = prog.new_var(ValueType::LValue);
                    prog.add_var_type(
                        ptr.to_owned(),
                        Box::new(IrType::PointerTo(member_type.to_owned())),
                    )?;
                    // ptr = obj_ptr + (byte offset)
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        ptr.to_owned(),
                        Src::Var(obj_ptr),
                        Src::Constant(Constant::Int(member_byte_offset as i128)),
                    ));

                    if this_expr_directly_on_lhs_of_assignment {
                        // store to struct member
                        Ok((instrs, Src::StoreAddressVar(ptr)))
                    } else {
                        // load from struct member
                        let dest = prog.new_var(ValueType::LValue);
                        prog.add_var_type(dest.to_owned(), member_type)?;
                        // dest = *ptr
                        instrs.push(Instruction::LoadFromAddress(
                            prog.new_instr_id(),
                            dest.to_owned(),
                            Src::Var(ptr),
                        ));
                        Ok((instrs, Src::Var(dest)))
                    }
                }
                IrType::Union(union_id) => {
                    let union_type = prog.get_union_type(&union_id)?;
                    let member_type = union_type.get_member_type(&member_name)?;

                    if this_expr_directly_on_lhs_of_assignment {
                        // store to union
                        Ok((instrs, Src::StoreAddressVar(obj_ptr)))
                    } else {
                        // load from union
                        let dest = prog.new_var(ValueType::LValue);
                        prog.add_var_type(dest.to_owned(), member_type)?;
                        // dest = *obj_ptr
                        instrs.push(Instruction::LoadFromAddress(
                            prog.new_instr_id(),
                            dest.to_owned(),
                            Src::Var(obj_ptr),
                        ));
                        Ok((instrs, Src::Var(dest)))
                    }
                }
                _ => unreachable!(),
            }
        }
        Expression::IndirectMemberSelection(obj, Identifier(member_name)) => {
            let (mut obj_instrs, obj_var) = convert_expression_to_ir(obj, prog, context)?;
            instrs.append(&mut obj_instrs);
            let obj_var_type = obj_var.get_type(&prog.program_metadata)?;
            obj_var_type.require_pointer_type()?;
            let inner_type = obj_var_type.dereference_pointer_type()?;
            inner_type.require_struct_or_union_type()?;
            match *inner_type {
                IrType::Struct(struct_id) => {
                    let struct_type = prog.get_struct_type(&struct_id)?;
                    let member_type = struct_type.get_member_type(&member_name)?;
                    let member_byte_offset = struct_type.get_member_byte_offset(&member_name)?;

                    let ptr = prog.new_var(ValueType::LValue);
                    prog.add_var_type(
                        ptr.to_owned(),
                        Box::new(IrType::PointerTo(member_type.to_owned())),
                    )?;
                    // ptr = (address of struct) + (byte offset)
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        ptr.to_owned(),
                        obj_var,
                        Src::Constant(Constant::Int(member_byte_offset as i128)),
                    ));

                    if this_expr_directly_on_lhs_of_assignment {
                        // store to struct member
                        Ok((instrs, Src::StoreAddressVar(ptr)))
                    } else {
                        // load from struct member
                        let dest = prog.new_var(ValueType::LValue);
                        prog.add_var_type(dest.to_owned(), member_type)?;
                        // dest = *ptr
                        instrs.push(Instruction::LoadFromAddress(
                            prog.new_instr_id(),
                            dest.to_owned(),
                            Src::Var(ptr),
                        ));
                        Ok((instrs, Src::Var(dest)))
                    }
                }
                IrType::Union(union_id) => {
                    let union_type = prog.get_union_type(&union_id)?;
                    let member_type = union_type.get_member_type(&member_name)?;

                    if this_expr_directly_on_lhs_of_assignment {
                        // store to union
                        Ok((instrs, Src::StoreAddressVar(obj_var.unwrap_var()?)))
                    } else {
                        // load from union
                        let dest = prog.new_var(ValueType::LValue);
                        prog.add_var_type(dest.to_owned(), member_type)?;
                        // dest = *obj_ptr
                        instrs.push(Instruction::LoadFromAddress(
                            prog.new_instr_id(),
                            dest.to_owned(),
                            obj_var,
                        ));
                        Ok((instrs, Src::Var(dest)))
                    }
                }
                _ => unreachable!(),
            }
        }
        Expression::PostfixIncrement(expr) => {
            context.directly_on_lhs_of_assignment = true;
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            context.directly_on_lhs_of_assignment = false;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var(ValueType::RValue);
            // propagate the type of dest: same as src
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            // check type is valid to be incremented
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::InvalidOperation(
                    "Incrementing a non-scalar type",
                ));
            }
            match expr_var {
                Src::Var(var) => {
                    // the returned value is the variable before incrementing
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(var.to_owned()),
                    ));

                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        var.to_owned(),
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                }
                Src::StoreAddressVar(var) => {
                    // load value to increment
                    let src_to_increment = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        src_to_increment.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::LoadFromAddress(
                        prog.new_instr_id(),
                        src_to_increment.to_owned(),
                        Src::Var(var.to_owned()),
                    ));

                    // the returned value is the variable before incrementing
                    prog.add_var_type(dest.to_owned(), expr_var_type.dereference_pointer_type()?)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(src_to_increment.to_owned()),
                    ));

                    // increment value
                    let result = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        result.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        result.to_owned(),
                        Src::Var(src_to_increment),
                        Src::Constant(Constant::Int(1)),
                    ));

                    // store value back
                    instrs.push(Instruction::StoreToAddress(
                        prog.new_instr_id(),
                        var,
                        Src::Var(result),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PostfixDecrement(expr) => {
            context.directly_on_lhs_of_assignment = true;
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            context.directly_on_lhs_of_assignment = false;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var(ValueType::RValue);
            // propagate the type of dest: same as src
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            // check type is valid to be incremented
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::InvalidOperation(
                    "Decrementing a non-scalar type",
                ));
            }
            match expr_var {
                Src::Var(var) => {
                    // the returned value is the variable before decrementing
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(var.to_owned()),
                    ));

                    instrs.push(Instruction::Sub(
                        prog.new_instr_id(),
                        var.to_owned(),
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                }
                Src::StoreAddressVar(var) => {
                    // load value to decrement
                    let src_to_decrement = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        src_to_decrement.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::LoadFromAddress(
                        prog.new_instr_id(),
                        src_to_decrement.to_owned(),
                        Src::Var(var.to_owned()),
                    ));

                    // the returned value is the variable before decrementing
                    prog.add_var_type(dest.to_owned(), expr_var_type.dereference_pointer_type()?)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(src_to_decrement.to_owned()),
                    ));

                    // increment value
                    let result = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        result.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::Sub(
                        prog.new_instr_id(),
                        result.to_owned(),
                        Src::Var(src_to_decrement),
                        Src::Constant(Constant::Int(1)),
                    ));

                    // store value back
                    instrs.push(Instruction::StoreToAddress(
                        prog.new_instr_id(),
                        var,
                        Src::Var(result),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PrefixIncrement(expr) => {
            context.directly_on_lhs_of_assignment = true;
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            context.directly_on_lhs_of_assignment = false;
            instrs.append(&mut expr_instrs);
            // make sure the result is an rvalue
            let dest = prog.new_var(ValueType::RValue);
            // expr_var is the variable returned, after incrementing
            // check type is valid to be incremented
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::InvalidOperation(
                    "Incrementing a non-scalar type",
                ));
            }
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        var.to_owned(),
                        Src::Var(var.to_owned()),
                        Src::Constant(Constant::Int(1)),
                    ));
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(var),
                    ));
                }
                Src::StoreAddressVar(var) => {
                    // load value to increment
                    let src_to_increment = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        src_to_increment.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::LoadFromAddress(
                        prog.new_instr_id(),
                        src_to_increment.to_owned(),
                        Src::Var(var.to_owned()),
                    ));

                    // increment value
                    let result = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        result.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        result.to_owned(),
                        Src::Var(src_to_increment),
                        Src::Constant(Constant::Int(1)),
                    ));

                    debug!(
                        "VAR TYPE: {}, RESULT TYPE: {}",
                        prog.get_var_type(&var)?,
                        prog.get_var_type(&result)?
                    );

                    // store value back
                    instrs.push(Instruction::StoreToAddress(
                        prog.new_instr_id(),
                        var,
                        Src::Var(result.to_owned()),
                    ));

                    // store result to dest
                    prog.add_var_type(dest.to_owned(), expr_var_type.dereference_pointer_type()?)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(result),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PrefixDecrement(expr) => {
            context.directly_on_lhs_of_assignment = true;
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            context.directly_on_lhs_of_assignment = false;
            instrs.append(&mut expr_instrs);
            // make sure the result is an rvalue
            let dest = prog.new_var(ValueType::RValue);
            // expr_var is the variable returned, after decrementing
            // check type is valid to be incremented
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::InvalidOperation(
                    "Incrementing a non-scalar type",
                ));
            }
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Sub(
                        prog.new_instr_id(),
                        var.to_owned(),
                        Src::Var(var.to_owned()),
                        Src::Constant(Constant::Int(1)),
                    ));
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(var),
                    ));
                }
                Src::StoreAddressVar(var) => {
                    // load value to increment
                    let src_to_decrement = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        src_to_decrement.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::LoadFromAddress(
                        prog.new_instr_id(),
                        src_to_decrement.to_owned(),
                        Src::Var(var.to_owned()),
                    ));

                    // increment value
                    let result = prog.new_var(ValueType::RValue);
                    prog.add_var_type(
                        result.to_owned(),
                        expr_var_type.dereference_pointer_type()?,
                    )?;
                    instrs.push(Instruction::Sub(
                        prog.new_instr_id(),
                        result.to_owned(),
                        Src::Var(src_to_decrement),
                        Src::Constant(Constant::Int(1)),
                    ));

                    debug!(
                        "VAR TYPE: {}, RESULT TYPE: {}",
                        prog.get_var_type(&var)?,
                        prog.get_var_type(&result)?
                    );

                    // store value back
                    instrs.push(Instruction::StoreToAddress(
                        prog.new_instr_id(),
                        var,
                        Src::Var(result.to_owned()),
                    ));

                    // store result to dest
                    prog.add_var_type(dest.to_owned(), expr_var_type.dereference_pointer_type()?)?;
                    instrs.push(Instruction::SimpleAssignment(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Var(result),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::UnaryOp(UnaryOperator::AddressOf, expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            let dest = prog.new_var(ValueType::RValue);
            instrs.push(Instruction::AddressOf(
                prog.new_instr_id(),
                dest.to_owned(),
                expr_var,
            ));
            // store type of dest
            prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(expr_var_type)))?;
            Ok((instrs, Src::Var(dest)))
        }
        Expression::UnaryOp(UnaryOperator::Dereference, expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            if this_expr_directly_on_lhs_of_assignment {
                // store to memory address
                match *expr_var_type {
                    IrType::PointerTo(_) => {
                        // prog.add_var_type(dest.to_owned(), expr_var_type)?;
                        match expr_var {
                            Src::Var(expr_var) => Ok((instrs, Src::StoreAddressVar(expr_var))),
                            _ => Err(MiddleEndError::AttemptToStoreToNonVariable),
                        }
                    }
                    _ => Err(MiddleEndError::DereferenceNonPointerType(expr_var_type)),
                }
            } else {
                // dereference load from memory address
                let dest = prog.new_var(ValueType::LValue);
                instrs.push(Instruction::LoadFromAddress(
                    prog.new_instr_id(),
                    dest.to_owned(),
                    expr_var,
                ));
                // check whether the var is allowed to be dereferenced;
                // if so, store the type of dest
                match *expr_var_type {
                    IrType::PointerTo(inner_type) => {
                        prog.add_var_type(dest.to_owned(), inner_type)?;
                    }
                    _ => return Err(MiddleEndError::DereferenceNonPointerType(expr_var_type)),
                }
                Ok((instrs, Src::Var(dest)))
            }
        }
        Expression::UnaryOp(op, expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            // unary convert type if necessary
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            match op {
                // UnaryOperator::AddressOf => {
                //     let dest = prog.new_var(ValueType::RValue);
                //     instrs.push(Instruction::AddressOf(dest.to_owned(), expr_var));
                //     // store type of dest
                //     prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(expr_var_type)))?;
                //     Ok((instrs, Src::Var(dest)))
                // }
                // UnaryOperator::Dereference => {
                //     if this_expr_directly_on_lhs_of_assignment {
                //         // store to memory address
                //         match *expr_var_type {
                //             IrType::PointerTo(_) => {
                //                 // prog.add_var_type(dest.to_owned(), expr_var_type)?;
                //                 match expr_var {
                //                     Src::Var(expr_var) => {
                //                         Ok((instrs, Src::StoreAddressVar(expr_var)))
                //                     }
                //                     _ => return Err(MiddleEndError::AttemptToStoreToNonVariable),
                //                 }
                //             }
                //             _ => {
                //                 return Err(MiddleEndError::DereferenceNonPointerType(
                //                     expr_var_type,
                //                 ))
                //             }
                //         }
                //     } else {
                //         // dereference load from memory address
                //         let dest = prog.new_var(ValueType::ModifiableLValue);
                //         instrs.push(Instruction::LoadFromAddress(dest.to_owned(), expr_var));
                //         // check whether the var is allowed to be dereferenced;
                //         // if so, store the type of dest
                //         match *expr_var_type {
                //             IrType::PointerTo(inner_type) => {
                //                 prog.add_var_type(dest.to_owned(), inner_type)?;
                //             }
                //             _ => {
                //                 return Err(MiddleEndError::DereferenceNonPointerType(
                //                     expr_var_type,
                //                 ))
                //             }
                //         }
                //         Ok((instrs, Src::Var(dest)))
                //     }
                // }
                UnaryOperator::Plus => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Constant(Constant::Int(0)),
                        expr_var,
                    ));
                    if !expr_var_type.is_arithmetic_type() {
                        return Err(MiddleEndError::InvalidOperation(
                            "Unary plus of a non-arithmetic type",
                        ));
                    }
                    // type of dest is same as type of src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::Minus => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::Sub(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        Src::Constant(Constant::Int(0)),
                        expr_var,
                    ));
                    if !expr_var_type.is_arithmetic_type() {
                        return Err(MiddleEndError::InvalidOperation(
                            "Unary minus of a non-arithmetic type",
                        ));
                    }
                    let dest_type = expr_var_type.smallest_signed_equivalent()?;
                    prog.add_var_type(dest.to_owned(), dest_type)?;
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::BitwiseNot => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::BitwiseNot(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        expr_var,
                    ));
                    if !expr_var_type.is_integral_type() {
                        return Err(MiddleEndError::InvalidOperation(
                            "Bitwise not of a non-integral type",
                        ));
                    }
                    // dest type is same as src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::LogicalNot => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::LogicalNot(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        expr_var,
                    ));
                    if !expr_var_type.is_scalar_type() {
                        return Err(MiddleEndError::InvalidOperation(
                            "Logical not of a non-scalar type",
                        ));
                    }
                    // dest type is same as src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    Ok((instrs, Src::Var(dest)))
                }
                _ => unreachable!("other cases handled separately"),
            }
        }
        Expression::SizeOfExpr(e) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(e, prog, context)?;
            instrs.append(&mut expr_instrs);
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;

            let type_size = if expr_var_type.is_array_type() {
                expr_var_type.get_array_size()?
            } else {
                expr_var_type.get_byte_size(&prog.program_metadata)
            };
            let size = match type_size {
                TypeSize::CompileTime(size) => Src::Constant(Constant::Int(size as i128)),
                TypeSize::Runtime(size_expr) => {
                    let (mut size_expr_instrs, size_var) =
                        convert_expression_to_ir(size_expr, prog, context)?;
                    instrs.append(&mut size_expr_instrs);
                    size_var
                }
            };
            let dest = prog.new_var(ValueType::RValue);
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
            instrs.push(Instruction::SimpleAssignment(
                prog.new_instr_id(),
                dest.to_owned(),
                size,
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::SizeOfType(t) => {
            let (type_info, _, _) = match get_type_info(&t.0, t.1, false, prog, context)? {
                None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                Some(x) => x,
            };
            let type_size = if type_info.is_array_type() {
                type_info.get_array_size()?
            } else {
                type_info.get_byte_size(&prog.program_metadata)
            };
            let byte_size = match type_size {
                TypeSize::CompileTime(size) => Src::Constant(Constant::Int(size as i128)),
                TypeSize::Runtime(size_expr) => {
                    let (mut size_expr_instrs, size_var) =
                        convert_expression_to_ir(size_expr, prog, context)?;
                    instrs.append(&mut size_expr_instrs);
                    size_var
                }
            };
            let dest = prog.new_var(ValueType::RValue);
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
            instrs.push(Instruction::SimpleAssignment(
                prog.new_instr_id(),
                dest.to_owned(),
                byte_size,
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::BinaryOp(op, left, right) => {
            let (mut left_instrs, left_var) = convert_expression_to_ir(left, prog, context)?;
            instrs.append(&mut left_instrs);
            let (mut right_instrs, right_var) = convert_expression_to_ir(right, prog, context)?;
            instrs.append(&mut right_instrs);
            let dest = prog.new_var(ValueType::RValue);
            let left_var_type = left_var.get_type(&prog.program_metadata)?;
            let right_var_type = right_var.get_type(&prog.program_metadata)?;
            match op {
                BinaryOperator::Mult => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_arithmetic_type()?;
                    right_var_type.require_arithmetic_type()?;
                    // left_var_type and right_var_type are the same cos of binary conversion
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::Mult(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Div => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_arithmetic_type()?;
                    right_var_type.require_arithmetic_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::Div(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Mod => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::Mod(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Add => {
                    let (mut convert_instrs, mut left_var, mut right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let mut left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let mut right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // must be either two arithmetic types, or a pointer and an integer
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_object_pointer_type()
                            && right_var_type.is_integral_type())
                        && !(left_var_type.is_integral_type()
                            && right_var_type.is_object_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid addition operand types",
                        ));
                    }

                    // if adding int and pointer, make sure left_var is always the
                    // pointer and right_var is the int
                    if right_var_type.is_object_pointer_type() {
                        std::mem::swap(&mut right_var, &mut left_var);
                        std::mem::swap(&mut right_var_type, &mut left_var_type);
                    }

                    if left_var_type.is_arithmetic_type() {
                        prog.add_var_type(dest.to_owned(), left_var_type)?;
                    } else {
                        // pointer + int
                        // result is the pointer type
                        prog.add_var_type(dest.to_owned(), left_var_type.to_owned())?;
                        // add to pointer in multiples of the byte size of the type it points to
                        let temp = prog.new_var(right_var.get_value_type());
                        prog.add_var_type(temp.to_owned(), right_var_type)?;
                        let ptr_object_byte_size = match left_var_type
                            .get_pointer_object_byte_size(&prog.program_metadata)?
                        {
                            TypeSize::CompileTime(size) => {
                                Src::Constant(Constant::Int(size as i128))
                            }
                            TypeSize::Runtime(size_expr) => {
                                let (mut size_expr_instrs, size_var) =
                                    convert_expression_to_ir(size_expr, prog, context)?;
                                instrs.append(&mut size_expr_instrs);
                                size_var
                            }
                        };
                        instrs.push(Instruction::Mult(
                            prog.new_instr_id(),
                            temp.to_owned(),
                            right_var,
                            ptr_object_byte_size,
                        ));
                        right_var = Src::Var(temp);
                    }
                    instrs.push(Instruction::Add(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Sub => {
                    let (mut convert_instrs, mut left_var, mut right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // must be either arithmetic - arithmetic, or pointer - integer, or pointer - pointer
                    // todo check for pointers being compatible types
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_object_pointer_type()
                            && right_var_type.is_integral_type())
                        && !(left_var_type.is_object_pointer_type()
                            && right_var_type.is_object_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid addition operand types",
                        ));
                    }
                    if left_var_type.is_arithmetic_type() {
                        prog.add_var_type(dest.to_owned(), left_var_type)?;
                    } else if right_var_type.is_integral_type() {
                        // pointer - integer
                        prog.add_var_type(dest.to_owned(), left_var_type.to_owned())?;

                        // subtract from pointer in multiples of the byte size it points to
                        let temp_right_var = prog.new_var(right_var.get_value_type());
                        prog.add_var_type(temp_right_var.to_owned(), right_var_type)?;
                        let ptr_object_byte_size = match left_var_type
                            .get_pointer_object_byte_size(&prog.program_metadata)?
                        {
                            TypeSize::CompileTime(size) => {
                                Src::Constant(Constant::Int(size as i128))
                            }
                            TypeSize::Runtime(size_expr) => {
                                let (mut size_expr_instrs, size_var) =
                                    convert_expression_to_ir(size_expr, prog, context)?;
                                instrs.append(&mut size_expr_instrs);
                                size_var
                            }
                        };
                        instrs.push(Instruction::Mult(
                            prog.new_instr_id(),
                            temp_right_var.to_owned(),
                            right_var,
                            ptr_object_byte_size,
                        ));
                        right_var = Src::Var(temp_right_var);

                        // convert ptr to i32
                        let temp_left_var = prog.new_var(left_var.get_value_type());
                        prog.add_var_type(temp_left_var.to_owned(), Box::new(IrType::I32))?;
                        instrs.push(Instruction::PtrToI32(
                            prog.new_instr_id(),
                            temp_left_var.to_owned(),
                            left_var,
                        ));
                        left_var = Src::Var(temp_left_var);
                    } else {
                        // pointer - pointer -> int
                        prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    }
                    instrs.push(Instruction::Sub(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LeftShift => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::LeftShift(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::RightShift => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::RightShift(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LessThan => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid comparison operand types",
                        ));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::LessThan(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::GreaterThan => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid comparison operand types",
                        ));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::GreaterThan(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LessThanEq => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid comparison operand types",
                        ));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::LessThanEq(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::GreaterThanEq => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid comparison operand types",
                        ));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::GreaterThanEq(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Equal => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // both arithmetic, both pointer, or pointer compared to NULL (int 0)
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_integral_type())
                        && !(left_var_type.is_integral_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid equality comparison operand types",
                        ));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::Equal(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::NotEqual => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    // both arithmetic, both pointer, or pointer compared to NULL (int 0)
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_integral_type())
                        && !(left_var_type.is_integral_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::InvalidOperation(
                            "Invalid equality comparison operand types",
                        ));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::NotEqual(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::BitwiseAnd => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::BitwiseAnd(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::BitwiseOr => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::BitwiseOr(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::BitwiseXor => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(&prog.program_metadata)?;
                    let right_var_type = right_var.get_type(&prog.program_metadata)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::BitwiseXor(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LogicalAnd => {
                    // no binary conversion for logical AND and OR
                    left_var_type.require_scalar_type()?;
                    right_var_type.require_scalar_type()?;
                    // result is always int 0 or 1
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::LogicalAnd(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LogicalOr => {
                    // no binary conversion for logical AND and OR
                    left_var_type.require_scalar_type()?;
                    right_var_type.require_scalar_type()?;
                    // result is always int 0 or 1
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::LogicalOr(
                        prog.new_instr_id(),
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::Ternary(cond, true_expr, false_expr) => {
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            let dest = prog.new_var(ValueType::RValue);
            let false_label = prog.new_label();
            let end_label = prog.new_label();
            // if condition false, execute the false instructions
            instrs.push(Instruction::BrIfEq(
                prog.new_instr_id(),
                cond_var,
                Src::Constant(Constant::Int(0)),
                false_label.to_owned(),
            ));
            // if condition true, fall through to the true instructions
            let (mut true_instrs, true_var) = convert_expression_to_ir(true_expr, prog, context)?;
            instrs.append(&mut true_instrs);
            // unary convert result of the expression
            let (mut unary_convert_true_instrs, mut true_var) = unary_convert(true_var, prog)?;
            instrs.append(&mut unary_convert_true_instrs);
            let true_var_type = true_var.get_type(&prog.program_metadata)?;

            // convert the false expr already, so we can do type checking and conversion, but don't insert
            // the instructions just yet
            let (mut false_instrs, false_var) =
                convert_expression_to_ir(false_expr, prog, context)?;
            // unary convert result of the expression
            let (mut unary_convert_false_instrs, mut false_var) = unary_convert(false_var, prog)?;
            let false_var_type = false_var.get_type(&prog.program_metadata)?;

            let mut false_binary_convert_instrs = Vec::new();
            if true_var_type != false_var_type {
                let (
                    mut true_binary_convert_instrs,
                    false_convert_instrs,
                    converted_true_var,
                    converted_false_var,
                ) = binary_convert_separately(true_var, false_var, prog)?;
                true_var = converted_true_var;
                false_var = converted_false_var;
                false_binary_convert_instrs = false_convert_instrs;
                instrs.append(&mut true_binary_convert_instrs);
            }
            prog.add_var_type(dest.to_owned(), true_var.get_type(&prog.program_metadata)?)?;

            // assign the result to dest
            instrs.push(Instruction::SimpleAssignment(
                prog.new_instr_id(),
                dest.to_owned(),
                true_var,
            ));
            // jump over the false instructions
            instrs.push(Instruction::Br(prog.new_instr_id(), end_label.to_owned()));
            // false instructions
            instrs.push(Instruction::Label(prog.new_instr_id(), false_label));
            instrs.append(&mut false_instrs);
            instrs.append(&mut unary_convert_false_instrs);
            instrs.append(&mut false_binary_convert_instrs);
            // assign the result to dest
            instrs.push(Instruction::SimpleAssignment(
                prog.new_instr_id(),
                dest.to_owned(),
                false_var,
            ));
            instrs.push(Instruction::Label(prog.new_instr_id(), end_label));

            Ok((instrs, Src::Var(dest)))
        }
        Expression::Assignment(dest_expr, src_expr) => {
            let (mut src_expr_instrs, mut src_var) =
                convert_expression_to_ir(src_expr, prog, context)?;
            instrs.append(&mut src_expr_instrs);
            let src_var_type = src_var.get_type(&prog.program_metadata)?;

            context.directly_on_lhs_of_assignment = true;
            let (mut dest_expr_instrs, dest_var) =
                convert_expression_to_ir(dest_expr, prog, context)?;
            context.directly_on_lhs_of_assignment = false;
            instrs.append(&mut dest_expr_instrs);

            // check that we're assigning to an lvalue
            if !dest_var.get_value_type().is_lvalue() {
                return Err(MiddleEndError::AttemptToModifyNonLValue);
            }

            let mut dest_var_type = dest_var.get_type(&prog.program_metadata)?;
            match dest_var {
                Src::Var(_) => {}
                Src::StoreAddressVar(_) => {
                    dest_var_type = dest_var_type.dereference_pointer_type()?
                }
                Src::Constant(_) | Src::Fun(_) => return Err(MiddleEndError::InvalidAssignment),
            }

            if src_var_type != dest_var_type {
                let (mut convert_instrs, converted_var) =
                    convert_type_for_assignment(src_var, src_var_type, dest_var_type, prog)?;
                instrs.append(&mut convert_instrs);
                src_var = converted_var;
                //todo other options of possible type combinations
            }

            let (dest, is_store_to_address) = match dest_var {
                Src::Var(var) => (var, false),
                Src::StoreAddressVar(var) => (var, true),
                Src::Constant(_) | Src::Fun(_) => return Err(MiddleEndError::InvalidAssignment),
            };

            // either store to the memory address given by the pointer dest,
            // or store into the local var dest
            if is_store_to_address {
                instrs.push(Instruction::StoreToAddress(
                    prog.new_instr_id(),
                    dest.to_owned(),
                    src_var,
                ));
            } else {
                instrs.push(Instruction::SimpleAssignment(
                    prog.new_instr_id(),
                    dest.to_owned(),
                    src_var,
                ));
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::Cast(cast_type_decl, expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            // get type to cast into
            let (cast_type, _, _) =
                match get_type_info(&cast_type_decl.0, cast_type_decl.1, false, prog, context)? {
                    None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                    Some(x) => x,
                };
            // get conversion instrs into cast type
            let expr_var_type = expr_var.get_type(&prog.program_metadata)?;
            let (mut cast_instrs, dest) =
                get_type_conversion_instrs(expr_var, expr_var_type, cast_type, prog)?;
            instrs.append(&mut cast_instrs);
            Ok((instrs, dest))
        }
        Expression::ExpressionList(_, _) => {
            todo!("expression lists")
        }
    }
}
