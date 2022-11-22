use crate::middle_end::compile_time_eval::eval_integral_constant_expression;
use crate::middle_end::context::{Context, LoopContext, SwitchContext};
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::instructions::{Constant, Src};
use crate::middle_end::ir::{Function, Program};
use crate::middle_end::ir_types::{IrType, StructType};
use crate::middle_end::middle_end_error::{MiddleEndError, TypeError};
use crate::parser::ast;
use crate::parser::ast::{
    ArithmeticType, BinaryOperator, Declarator, DeclaratorInitialiser, Expression,
    ExpressionOrDeclaration, Identifier, Initialiser, LabelledStatement, ParameterTypeList,
    Program as AstProgram, SpecifierQualifier, Statement, StorageClassSpecifier,
    StructType as AstStructType, TypeSpecifier, UnaryOperator,
};

pub fn convert_to_ir(ast: AstProgram) -> Result<Box<Program>, MiddleEndError> {
    let mut program = Box::new(Program::new());
    let mut context = Box::new(Context::new());
    for stmt in ast.0 {
        let global_instrs = convert_statement_to_ir(stmt, &mut program, &mut context);
        match global_instrs {
            Ok(mut instrs) => program.global_instrs.append(&mut instrs),
            Err(e) => return Err(e),
        }
    }
    Ok(program)
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
        Statement::Goto(x) => match prog.label_identifiers.get(&x.0) {
            Some(label) => instrs.push(Instruction::Br(label.to_owned())),
            None => {
                let label = prog.new_label();
                prog.label_identifiers.insert(x.0, label.to_owned());
                instrs.push(Instruction::Br(label));
            }
        },
        Statement::Continue => match context.get_continue_label() {
            None => {
                return Err(MiddleEndError::ContinueOutsideLoopContext);
            }
            Some(label) => {
                instrs.push(Instruction::Br(label.to_owned()));
            }
        },
        Statement::Break => match context.get_break_label() {
            None => {
                return Err(MiddleEndError::BreakOutsideLoopOrSwitchContext);
            }
            Some(label) => {
                instrs.push(Instruction::Br(label.to_owned()));
            }
        },
        Statement::Return(expr) => match expr {
            None => {
                instrs.push(Instruction::Ret(None));
            }
            Some(expr) => {
                let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
                instrs.append(&mut expr_instrs);
                instrs.push(Instruction::Ret(Some(expr_var)));
            }
        },
        Statement::While(cond, body) => {
            let loop_start_label = prog.new_label();
            let loop_end_label = prog.new_label();
            // start of loop label
            instrs.push(Instruction::Label(loop_start_label.to_owned()));
            context.push_loop(LoopContext::while_loop(
                loop_start_label.to_owned(),
                loop_end_label.to_owned(),
            ));
            // while condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // jump out of loop if condition false
            instrs.push(Instruction::BrIfEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_end_label.to_owned(),
            ));
            // loop body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // jump back to start of loop to evaluate condition again
            instrs.push(Instruction::Br(loop_start_label));
            instrs.push(Instruction::Label(loop_end_label));
            context.pop_loop();
        }
        Statement::DoWhile(body, cond) => {
            let loop_start_label = prog.new_label();
            let loop_end_label = prog.new_label();
            let loop_continue_label = prog.new_label();
            // start of loop label
            instrs.push(Instruction::Label(loop_start_label.to_owned()));
            context.push_loop(LoopContext::do_while_loop(
                loop_start_label.to_owned(),
                loop_end_label.to_owned(),
                loop_continue_label.to_owned(),
            ));
            // loop body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // continue label
            instrs.push(Instruction::Label(loop_continue_label));
            // loop condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // jump back to start of loop if condition true
            instrs.push(Instruction::BrIfNotEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_start_label,
            ));
            // end of loop
            instrs.push(Instruction::Label(loop_end_label));
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
            instrs.push(Instruction::Label(loop_start_label.to_owned()));
            context.push_loop(LoopContext::for_loop(
                loop_start_label.to_owned(),
                loop_end_label.to_owned(),
                loop_continue_label.to_owned(),
            ));
            // condition
            let cond_var = match cond {
                None => {
                    let temp = prog.new_var();
                    instrs.push(Instruction::SimpleAssignment(
                        temp.to_owned(),
                        Src::Constant(Constant::Int(1)),
                    ));
                    Src::Var(temp)
                }
                Some(e) => {
                    let (mut expr_instrs, expr_var) = convert_expression_to_ir(e, prog, context)?;
                    instrs.append(&mut expr_instrs);
                    expr_var
                }
            };
            instrs.push(Instruction::BrIfEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_end_label.to_owned(),
            ));
            // loop body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // continue label
            instrs.push(Instruction::Label(loop_continue_label));
            // end-of-loop expression, before looping back to condition again
            match end {
                None => {}
                Some(e) => {
                    let (mut expr_instrs, _) = convert_expression_to_ir(e, prog, context)?;
                    instrs.append(&mut expr_instrs);
                }
            }
            // loop back to condition
            instrs.push(Instruction::Br(loop_start_label));
            // end of loop label
            instrs.push(Instruction::Label(loop_end_label));
            context.pop_loop();
        }
        Statement::If(cond, body) => {
            // if statement condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // if condition is false, jump to after body
            let if_end_label = prog.new_label();
            instrs.push(Instruction::BrIfEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                if_end_label.to_owned(),
            ));
            // if statement body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // end of if statement label
            instrs.push(Instruction::Label(if_end_label));
        }
        Statement::IfElse(cond, true_body, false_body) => {
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // if condition is false, jump to else body
            let else_label = prog.new_label();
            instrs.push(Instruction::BrIfEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                else_label.to_owned(),
            ));
            // if body
            instrs.append(&mut convert_statement_to_ir(true_body, prog, context)?);
            // jump to after else body
            let else_end_label = prog.new_label();
            instrs.push(Instruction::Br(else_end_label.to_owned()));
            // else body
            instrs.push(Instruction::Label(else_label));
            instrs.append(&mut convert_statement_to_ir(false_body, prog, context)?);
            instrs.push(Instruction::Label(else_end_label));
        }
        Statement::Switch(switch_expr, body) => {
            let switch_end_label = prog.new_label();
            let (mut expr_instrs, switch_src) =
                convert_expression_to_ir(switch_expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let switch_var = match switch_src {
                Src::Var(var) => var,
                Src::Constant(c) => {
                    let temp = prog.new_var();
                    instrs.push(Instruction::SimpleAssignment(
                        temp.to_owned(),
                        Src::Constant(c),
                    ));
                    temp
                }
                Src::Fun(_) => unreachable!(),
            };
            context.push_switch(SwitchContext::new(switch_end_label.to_owned(), switch_var));
            // switch body
            instrs.append(&mut convert_statement_to_ir(body, prog, context)?);
            // add default case after all other cases, if it exists
            let switch_context = context.pop_switch()?;
            match switch_context.default_case {
                None => {}
                Some(mut default_instrs) => {
                    instrs.append(&mut default_instrs);
                }
            }
            // end of switch label
            instrs.push(Instruction::Label(switch_end_label));
        }
        Statement::Labelled(stmt) => {
            match stmt {
                LabelledStatement::Named(Identifier(label_name), stmt) => {
                    let label = prog.new_identifier_label(label_name);
                    instrs.push(Instruction::Label(label));
                    instrs.append(&mut convert_statement_to_ir(stmt, prog, context)?);
                }
                LabelledStatement::Case(expr, stmt) => {
                    // case statements are only allowed in a switch context
                    if !context.is_in_switch_context() {
                        return Err(MiddleEndError::CaseOutsideSwitchContext);
                    }
                    let (mut expr_instrs, expr_var) =
                        convert_expression_to_ir(expr, prog, context)?;
                    instrs.append(&mut expr_instrs);
                    let end_of_case_label = prog.new_label();
                    // check if case condition matches the switch expression
                    instrs.push(Instruction::BrIfNotEq(
                        expr_var,
                        Src::Var(context.get_switch_variable().unwrap()),
                        end_of_case_label.to_owned(),
                    ));
                    // case body
                    instrs.append(&mut convert_statement_to_ir(stmt, prog, context)?);
                    // end of case label
                    instrs.push(Instruction::Label(end_of_case_label));
                }
                LabelledStatement::Default(stmt) => {
                    // todo default statement may contain other cases
                    let body_instrs = convert_statement_to_ir(stmt, prog, context)?;
                    context.add_default_switch_case(body_instrs)?;
                }
            }
        }
        Statement::Expr(e) => {
            let (mut expr_instrs, _) = convert_expression_to_ir(e, prog, context)?;
            instrs.append(&mut expr_instrs);
        }
        Statement::Declaration(sq, declarators) => {
            for declarator in declarators {
                match declarator {
                    DeclaratorInitialiser::NoInit(d) => {
                        match get_type_info(&sq, Some(d), prog, context)? {
                            None => {
                                // typedef declaration, so no need to do anything more here
                            }
                            Some((type_info, name, _params)) => match name {
                                Some(name) => {
                                    let var = prog.new_var();
                                    context.add_variable_to_scope(
                                        name,
                                        var.to_owned(),
                                        type_info.to_owned(),
                                    )?;
                                    prog.add_var_type(var, type_info)?;
                                }
                                None => {
                                    // If declarator has no name, it should be a
                                    // Statement::EmptyDeclaration, so this case should never
                                    // be reached
                                    unreachable!()
                                }
                            },
                        };
                    }
                    DeclaratorInitialiser::Init(d, init_expr) => {
                        let (type_info, name, _) = match get_type_info(&sq, Some(d), prog, context)?
                        {
                            None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                            Some(x) => x,
                        };
                        match name {
                            None => return Err(MiddleEndError::InvalidAbstractDeclarator),
                            Some(name) => {
                                let var =
                                    match *init_expr {
                                        Initialiser::Expr(e) => {
                                            let (mut expr_instrs, expr_var) =
                                                convert_expression_to_ir(e, prog, context)?;
                                            instrs.append(&mut expr_instrs);
                                            match expr_var {
                                                Src::Var(var) => var,
                                                Src::Constant(c) => {
                                                    let v = prog.new_var();
                                                    instrs.push(Instruction::SimpleAssignment(
                                                        v.to_owned(),
                                                        Src::Constant(c),
                                                    ));
                                                    v
                                                }
                                                Src::Fun(_) => return Err(
                                                    MiddleEndError::InvalidInitialiserExpression,
                                                ),
                                            }
                                        }
                                        Initialiser::List(_) => {
                                            todo!("struct/union/array initialiser")
                                        }
                                    };
                                context.add_variable_to_scope(
                                    name,
                                    var.to_owned(),
                                    type_info.to_owned(),
                                )?;
                                prog.add_var_type(var, type_info)?;
                            }
                        }
                    }
                }
            }
        }
        Statement::EmptyDeclaration(sq) => {
            match get_type_info(&sq, None, prog, context)? {
                None => {
                    // typedef declaration is invalid without a name
                    return Err(MiddleEndError::InvalidTypedefDeclaration);
                }
                Some((type_info, _name, _params)) => match *type_info {
                    IrType::I8
                    | IrType::U8
                    | IrType::I16
                    | IrType::U16
                    | IrType::I32
                    | IrType::U32
                    | IrType::I64
                    | IrType::U64
                    | IrType::F32
                    | IrType::F64
                    | IrType::Void
                    | IrType::ArrayOf(_, _)
                    | IrType::PointerTo(_)
                    | IrType::Function(_, _) => return Err(MiddleEndError::InvalidDeclaration),
                    IrType::Struct(_) | IrType::Union(_) => {}
                },
            }
        }
        Statement::FunctionDeclaration(sq, decl, body) => {
            let (type_info, name, param_bindings) =
                match get_type_info(&sq, Some(decl), prog, context)? {
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
                    let param_var = prog.new_var();
                    param_var_mappings.push(param_var.to_owned());
                    context.add_variable_to_scope(
                        param_name,
                        param_var.to_owned(),
                        param_type.to_owned(),
                    )?;
                    prog.add_var_type(param_var, param_type)?;
                }
            }
            // function body instructions
            let instrs = convert_statement_to_ir(body, prog, context)?;
            let fun = Function::new(instrs, type_info, param_var_mappings);
            let fun_id = prog.new_fun(name.to_owned(), fun);
            context.add_function_declaration(name, fun_id)?;
            context.pop_scope()
        }
        Statement::Empty => {}
    }
    Ok(instrs)
}

/// returns the list of instructions generated, and the name of the temp variable
/// the result is assigned to
fn convert_expression_to_ir(
    src_expr: Box<Expression>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    let mut instrs: Vec<Instruction> = Vec::new();
    match *src_expr {
        Expression::Identifier(Identifier(name)) => {
            if context.in_function_name_expr {
                let fun = context.resolve_identifier_to_fun(&name)?;
                Ok((instrs, Src::Fun(fun)))
            } else {
                let var = context.resolve_identifier_to_var(&name)?;
                Ok((instrs, Src::Var(var)))
            }
        }
        Expression::Constant(c) => match c {
            ast::Constant::Int(i) => Ok((instrs, Src::Constant(Constant::Int(i as i128)))),
            ast::Constant::Float(f) => Ok((instrs, Src::Constant(Constant::Float(f)))),
            ast::Constant::Char(ch) => Ok((instrs, Src::Constant(Constant::Int(ch as i128)))),
        },
        Expression::StringLiteral(s) => {
            let string_id = prog.new_string_literal(s);
            let dest = prog.new_var();
            instrs.push(Instruction::PointerToStringLiteral(
                dest.to_owned(),
                string_id,
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::Index(arr, index) => {
            let (mut arr_instrs, arr_var) = convert_expression_to_ir(arr, prog, context)?;
            instrs.append(&mut arr_instrs);
            let (mut index_instrs, index_var) = convert_expression_to_ir(index, prog, context)?;
            instrs.append(&mut index_instrs);
            // array variable is a pointer to the start of the array
            let ptr = prog.new_var();
            instrs.push(Instruction::Add(ptr.to_owned(), arr_var, index_var));
            let dest = prog.new_var();
            instrs.push(Instruction::Dereference(dest.to_owned(), Src::Var(ptr)));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::FunctionCall(fun, params) => {
            context.in_function_name_expr = true;
            let (mut fun_instrs, fun_var) = convert_expression_to_ir(fun, prog, context)?;
            instrs.append(&mut fun_instrs);
            context.in_function_name_expr = false;
            let fun_identifier = match fun_var {
                Src::Var(_) | Src::Constant(_) => return Err(MiddleEndError::InvalidFunctionCall),
                Src::Fun(f) => f,
            };
            let mut param_srcs: Vec<Src> = Vec::new();
            for param in params {
                let (mut param_instrs, param_var) = convert_expression_to_ir(param, prog, context)?;
                instrs.append(&mut param_instrs);
                param_srcs.push(param_var);
            }
            let dest = prog.new_var();
            instrs.push(Instruction::Call(
                dest.to_owned(),
                Src::Fun(fun_identifier),
                param_srcs,
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::DirectMemberSelection(_, _) => {
            todo!()
        }
        Expression::IndirectMemberSelection(_, _) => {
            todo!()
        }
        Expression::PostfixIncrement(expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var();
            // propagate the type of dest: same as src
            let (mut unary_convert_instrs, expr_var) = unary_convert_if_necessary(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            // check type is valid to be incremented
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                    "Incrementing a non-scalar type",
                )));
            }
            prog.add_var_type(dest.to_owned(), expr_var_type)?;
            // the returned value is the variable before incrementing
            instrs.push(Instruction::SimpleAssignment(
                dest.to_owned(),
                expr_var.to_owned(),
            ));
            // check for a valid lvalue before adding add instr
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Add(
                        var.to_owned(),
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PostfixDecrement(expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var();
            // propagate the type of dest: same as src
            let (mut unary_convert_instrs, expr_var) = unary_convert_if_necessary(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            // check type is valid to be incremented
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                    "Decrementing a non-scalar type",
                )));
            }
            prog.add_var_type(dest.to_owned(), expr_var_type)?;
            // the returned value is the variable before decrementing
            instrs.push(Instruction::SimpleAssignment(
                dest.to_owned(),
                expr_var.to_owned(),
            ));
            // check for valid lvalue before adding sub instr
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Sub(
                        var.to_owned(),
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PrefixIncrement(expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            // expr_var is the variable returned, after incrementing - no need to store a new type in prog
            // check type is valid to be incremented
            let (mut unary_convert_instrs, expr_var) = unary_convert_if_necessary(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                    "Incrementing a non-scalar type",
                )));
            }
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Add(
                        var.to_owned(),
                        Src::Var(var.to_owned()),
                        Src::Constant(Constant::Int(1)),
                    ));
                    Ok((instrs, Src::Var(var)))
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
        }
        Expression::PrefixDecrement(expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            // expr_var is the variable returned, after decrementing - no need to store a new type in prog
            // check type is valid to be incremented
            let (mut unary_convert_instrs, expr_var) = unary_convert_if_necessary(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            if !expr_var_type.is_scalar_type() {
                return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                    "Incrementing a non-scalar type",
                )));
            }
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Sub(
                        var.to_owned(),
                        Src::Var(var.to_owned()),
                        Src::Constant(Constant::Int(1)),
                    ));
                    Ok((instrs, Src::Var(var)))
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
        }
        Expression::UnaryOp(op, expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var();
            // unary convert type if necessary
            let (mut unary_convert_instrs, expr_var) = unary_convert_if_necessary(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            match op {
                UnaryOperator::AddressOf => {
                    instrs.push(Instruction::AddressOf(dest.to_owned(), expr_var));
                    // store type of dest
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(expr_var_type)))?;
                }
                UnaryOperator::Dereference => {
                    instrs.push(Instruction::Dereference(dest.to_owned(), expr_var));
                    // check whether the var is allowed to be dereferenced;
                    // if so, store the type of dest
                    match *expr_var_type {
                        IrType::PointerTo(inner_type) => {
                            prog.add_var_type(dest.to_owned(), inner_type)?;
                        }
                        _ => {
                            return Err(MiddleEndError::TypeError(
                                TypeError::DereferenceNonPointerType(expr_var_type),
                            ))
                        }
                    }
                }
                UnaryOperator::Plus => {
                    instrs.push(Instruction::Add(
                        dest.to_owned(),
                        Src::Constant(Constant::Int(0)),
                        expr_var,
                    ));
                    if !expr_var_type.is_arithmetic_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Unary plus of a non-arithmetic type",
                        )));
                    }
                    // type of dest is same as type of src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                }
                UnaryOperator::Minus => {
                    instrs.push(Instruction::Sub(
                        dest.to_owned(),
                        Src::Constant(Constant::Int(0)),
                        expr_var,
                    ));
                    if !expr_var_type.is_arithmetic_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Unary minus of a non-arithmetic type",
                        )));
                    }
                    let dest_type = expr_var_type.smallest_signed_equivalent()?;
                    prog.add_var_type(dest.to_owned(), dest_type)?;
                }
                UnaryOperator::BitwiseNot => {
                    instrs.push(Instruction::BitwiseNot(dest.to_owned(), expr_var));
                    if !expr_var_type.is_integral_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Bitwise not of a non-integral type",
                        )));
                    }
                    // dest type is same as src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                }
                UnaryOperator::LogicalNot => {
                    instrs.push(Instruction::LogicalNot(dest.to_owned(), expr_var));
                    if !expr_var_type.is_scalar_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Logical not of a non-scalar type",
                        )));
                    }
                    // dest type is same as src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                }
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::SizeOfExpr(_) => {
            todo!()
        }
        Expression::SizeOfType(_) => {
            todo!()
        }
        Expression::BinaryOp(op, left, right) => {
            let (mut left_instrs, left_var) = convert_expression_to_ir(left, prog, context)?;
            instrs.append(&mut left_instrs);
            let (mut right_instrs, right_var) = convert_expression_to_ir(right, prog, context)?;
            instrs.append(&mut right_instrs);
            let dest = prog.new_var();
            // unary convert operands
            let (mut left_unary_convert_instrs, left_var) =
                unary_convert_if_necessary(left_var, prog)?;
            instrs.append(&mut left_unary_convert_instrs);
            let (mut right_unary_convert_instrs, right_var) =
                unary_convert_if_necessary(right_var, prog)?;
            instrs.append(&mut right_unary_convert_instrs);
            // todo binary conversions
            let left_var_type = left_var.get_type(prog)?;
            let right_var_type = right_var.get_type(prog)?;
            match op {
                BinaryOperator::Mult => {
                    instrs.push(Instruction::Mult(dest.to_owned(), left_var, right_var));
                    // check left and right types match and allow mult
                    if !left_var_type.is_arithmetic_type() || !right_var_type.is_arithmetic_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Mult of non-arithmetic type",
                        )));
                    }
                }
                BinaryOperator::Div => {
                    instrs.push(Instruction::Div(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Mod => {
                    instrs.push(Instruction::Mod(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Add => {
                    instrs.push(Instruction::Add(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Sub => {
                    instrs.push(Instruction::Sub(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::LeftShift => {
                    instrs.push(Instruction::LeftShift(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::RightShift => {
                    instrs.push(Instruction::RightShift(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LessThan => {
                    instrs.push(Instruction::LessThan(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::GreaterThan => {
                    instrs.push(Instruction::GreaterThan(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LessThanEq => {
                    instrs.push(Instruction::LessThanEq(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::GreaterThanEq => {
                    instrs.push(Instruction::GreaterThanEq(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Equal => {
                    instrs.push(Instruction::Equal(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::NotEqual => {
                    instrs.push(Instruction::NotEqual(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::BitwiseAnd => {
                    instrs.push(Instruction::BitwiseAnd(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::BitwiseOr => {
                    instrs.push(Instruction::BitwiseOr(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::BitwiseXor => {
                    instrs.push(Instruction::BitwiseXor(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LogicalAnd => {
                    instrs.push(Instruction::LogicalAnd(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LogicalOr => {
                    instrs.push(Instruction::LogicalOr(dest.to_owned(), left_var, right_var));
                }
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::Ternary(cond, true_expr, false_expr) => {
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            let dest = prog.new_var();
            let false_label = prog.new_label();
            let end_label = prog.new_label();
            // if condition false, execute the false instructions
            instrs.push(Instruction::BrIfEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                false_label.to_owned(),
            ));
            // if condition true, fall through to the true instructions
            let (mut true_instrs, true_var) = convert_expression_to_ir(true_expr, prog, context)?;
            instrs.append(&mut true_instrs);
            // assign the result to dest
            instrs.push(Instruction::SimpleAssignment(dest.to_owned(), true_var));
            // jump over the false instructions
            instrs.push(Instruction::Br(end_label.to_owned()));
            // false instructions
            instrs.push(Instruction::Label(false_label));
            let (mut false_instrs, false_var) =
                convert_expression_to_ir(false_expr, prog, context)?;
            instrs.append(&mut false_instrs);
            // assign the result to dest
            instrs.push(Instruction::SimpleAssignment(dest.to_owned(), false_var));
            instrs.push(Instruction::Label(end_label));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::Assignment(dest_expr, src_expr, op) => {
            // todo can dest_expr be anything other than an identifier?
            //      pointers and array access i guess
            todo!()
        }
        Expression::Cast(_, _) => {
            todo!()
        }
        Expression::ExpressionList(_, _) => {
            todo!()
        }
    }
}

type FunctionParameterBindings = Vec<(String, Box<IrType>)>;

fn get_type_info(
    specifier: &SpecifierQualifier,
    declarator: Option<Box<Declarator>>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<
    Option<(
        Box<IrType>,
        Option<String>,
        // parameter bindings, if this is a function definition
        Option<FunctionParameterBindings>,
    )>,
    MiddleEndError,
> {
    let ir_type = match &specifier.type_specifier {
        TypeSpecifier::ArithmeticType(t) => match t {
            ArithmeticType::I8 => Box::new(IrType::I8),
            ArithmeticType::U8 => Box::new(IrType::U8),
            ArithmeticType::I16 => Box::new(IrType::I16),
            ArithmeticType::U16 => Box::new(IrType::U16),
            ArithmeticType::I32 => Box::new(IrType::I32),
            ArithmeticType::U32 => Box::new(IrType::U32),
            ArithmeticType::I64 => Box::new(IrType::I64),
            ArithmeticType::U64 => Box::new(IrType::U64),
            ArithmeticType::F32 => Box::new(IrType::F32),
            ArithmeticType::F64 => Box::new(IrType::F64),
        },
        TypeSpecifier::Void => Box::new(IrType::Void),
        TypeSpecifier::Struct(struct_type) => match struct_type {
            AstStructType::Declaration(Identifier(struct_name)) => {
                let struct_type_id =
                    prog.add_struct_type(StructType::named(struct_name.to_owned()))?;
                Box::new(IrType::Struct(struct_type_id))
            }
            AstStructType::Definition(struct_name, members) => {
                let mut struct_type = match struct_name {
                    Some(Identifier(name)) => StructType::named(name.to_owned()),
                    None => StructType::unnamed(),
                };
                for member in members {
                    for decl in &member.1 {
                        let (member_type_info, member_name, _params) = match get_type_info(
                            &member.0,
                            Some(Box::new(*decl.to_owned())),
                            prog,
                            context,
                        )? {
                            None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                            Some(x) => x,
                        };
                        if member_name == None {
                            return Err(MiddleEndError::UnnamedStructMember);
                        }
                        struct_type.push_member(member_name.unwrap(), member_type_info, prog)?;
                    }
                }
                let struct_type_id = prog.add_struct_type(struct_type)?;
                Box::new(IrType::Struct(struct_type_id))
            }
        },
        TypeSpecifier::Union(_) => {
            todo!("get_type_info for union")
        }
        TypeSpecifier::Enum(_) => {
            todo!("get_type_info for enum")
        }
        TypeSpecifier::CustomType(Identifier(name)) => context.resolve_typedef(&name)?,
    };

    let mut is_typedef = false;
    match specifier.storage_class_specifier {
        None => {}
        Some(StorageClassSpecifier::Typedef) => {
            is_typedef = true;
        }
        Some(StorageClassSpecifier::Auto) => {
            todo!()
        }
        Some(StorageClassSpecifier::Extern) => {
            todo!()
        }
        Some(StorageClassSpecifier::Register) => {
            todo!()
        }
        Some(StorageClassSpecifier::Static) => {
            todo!()
        }
    }

    match declarator {
        Some(decl) => {
            let (ir_type, decl_name, param_bindings) =
                add_type_info_from_declarator(decl, ir_type, prog, context)?;
            if is_typedef {
                context.add_typedef(decl_name, ir_type)?;
                return Ok(None);
            }
            Ok(Some((ir_type, Some(decl_name), param_bindings)))
        }
        None => Ok(Some((ir_type, None, None))),
    }
}

/// Modifies the TypeInfo struct it's given, and returns the identifier name,
/// and the types of any parameters
fn add_type_info_from_declarator(
    decl: Box<Declarator>,
    type_info: Box<IrType>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<(Box<IrType>, String, Option<FunctionParameterBindings>), MiddleEndError> {
    match *decl {
        Declarator::Identifier(Identifier(name)) => Ok((type_info, name, None)),
        Declarator::PointerDeclarator(d) => {
            add_type_info_from_declarator(d, type_info.wrap_with_pointer(), prog, context)
        }
        Declarator::AbstractPointerDeclarator => Err(MiddleEndError::InvalidAbstractDeclarator),
        Declarator::ArrayDeclarator(d, size_expr) => {
            let size = match size_expr {
                None => 0, //todo maybe better way of handling this (get array size from initialiser)
                Some(size_expr) => eval_integral_constant_expression(size_expr, prog)? as u64,
            };
            add_type_info_from_declarator(d, type_info.wrap_with_array(size), prog, context)
        }
        Declarator::FunctionDeclarator(d, params) => {
            let mut is_variadic = false;
            let param_decls = match params {
                None => Vec::new(),
                Some(params) => match params {
                    ParameterTypeList::Normal(params) => params,
                    ParameterTypeList::Variadic(params) => {
                        is_variadic = true;
                        params
                    }
                },
            };

            let mut param_types: Vec<Box<IrType>> = Vec::new();
            let mut param_bindings: FunctionParameterBindings = Vec::new();
            for p in param_decls {
                let (param_type, param_name, _sub_param_bindings) =
                    match get_type_info(&p.0, p.1, prog, context)? {
                        None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                        Some(x) => x,
                    };
                param_types.push(param_type.to_owned());
                if let Some(name) = param_name {
                    param_bindings.push((name, param_type));
                }
            }

            let (type_info, name, _) = add_type_info_from_declarator(
                d,
                type_info.wrap_with_fun(param_types),
                prog,
                context,
            )?;
            Ok((type_info, name, Some(param_bindings)))
        }
    }
}

fn unary_convert_if_necessary(
    src: Src,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    let src_type = src.get_type(prog)?;
    let unary_converted_type = src_type.unary_convert();
    if src_type != unary_converted_type {
        let converted_var = prog.new_var();
        let instrs = vec![Instruction::get_conversion_instr(
            src,
            src_type,
            converted_var.to_owned(),
            unary_converted_type.to_owned(),
        )?];
        prog.add_var_type(converted_var.to_owned(), unary_converted_type)?;
        return Ok((instrs, Src::Var(converted_var)));
    }
    Ok((Vec::new(), src))
}
