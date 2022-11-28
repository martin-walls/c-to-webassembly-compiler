use crate::middle_end::compile_time_eval::eval_integral_constant_expression;
use crate::middle_end::context::{Context, IdentifierResolveResult, LoopContext, SwitchContext};
use crate::middle_end::ids::{ValueType, VarId};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::instructions::{Constant, Src};
use crate::middle_end::ir::{Function, Program};
use crate::middle_end::ir_types::{EnumConstant, IrType, StructType, UnionType};
use crate::middle_end::middle_end_error::{MiddleEndError, TypeError};
use crate::parser::ast;
use crate::parser::ast::{
    ArithmeticType, BinaryOperator, Constant as AstConstant, Declarator, DeclaratorInitialiser,
    EnumType, Enumerator, Expression, ExpressionOrDeclaration, Identifier, Initialiser,
    LabelledStatement, ParameterTypeList, Program as AstProgram, SpecifierQualifier, Statement,
    StorageClassSpecifier, StructType as AstStructType, TypeSpecifier, UnaryOperator,
    UnionType as AstUnionType,
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
                    let temp = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::SimpleAssignment(
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
                    let temp = prog.new_var(ValueType::RValue);
                    prog.add_var_type(temp.to_owned(), c.get_type(None))?;
                    instrs.push(Instruction::SimpleAssignment(
                        temp.to_owned(),
                        Src::Constant(c),
                    ));
                    temp
                }
                Src::Fun(_) | Src::StoreAddressVar(_) => unreachable!(),
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
                                    println!("Function declaration: {}", name);
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
                                    println!("Variable declaration: {}", name);
                                    let var = prog.new_var(ValueType::ModifiableLValue);
                                    context.add_variable_to_scope(
                                        name,
                                        var.to_owned(),
                                        type_info.to_owned(),
                                    )?;
                                    prog.add_var_type(var, type_info)?;
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
                        let dest_type_info = dest_type_info.resolve_array_size(&init_expr)?;

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
                                        let src_type = src.get_type(prog)?;
                                        println!(
                                            "src type: {}, dest type: {}",
                                            src_type, dest_type_info
                                        );
                                        if src_type != dest_type_info {
                                            if let Src::Constant(c) = &src {
                                                let temp = prog.new_var(ValueType::RValue);
                                                prog.add_var_type(
                                                    temp.to_owned(),
                                                    c.get_type(Some(dest_type_info.to_owned())),
                                                )?;
                                                instrs.push(Instruction::SimpleAssignment(
                                                    temp.to_owned(),
                                                    src,
                                                ));
                                                src = Src::Var(temp);
                                            }
                                            let (mut convert_instrs, converted_var) =
                                                convert_type_for_assignment(
                                                    src.to_owned(),
                                                    src.get_type(prog)?,
                                                    dest_type_info.to_owned(),
                                                    prog,
                                                )?;
                                            instrs.append(&mut convert_instrs);
                                            src = converted_var;
                                        }

                                        let dest = prog.new_var(ValueType::ModifiableLValue);
                                        prog.add_var_type(dest.to_owned(), src.get_type(prog)?)?;
                                        instrs.push(Instruction::SimpleAssignment(
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
                                            let dest = prog.new_var(ValueType::ModifiableLValue);
                                            let dest_type = IrType::ArrayOf(member_type, size);
                                            prog.add_var_type(
                                                dest.to_owned(),
                                                Box::new(dest_type.to_owned()),
                                            )?;
                                            instrs.push(Instruction::AllocateVariable(
                                                dest.to_owned(),
                                                dest_type.get_byte_size(prog),
                                            ));

                                            let mut init_instrs = array_initialiser(
                                                dest,
                                                Box::new(dest_type),
                                                initialisers,
                                                prog,
                                                context,
                                            )?;
                                            instrs.append(&mut init_instrs);
                                        }
                                        IrType::Struct(struct_id) => {
                                            let dest = prog.new_var(ValueType::ModifiableLValue);
                                            let dest_type = IrType::Struct(struct_id);
                                            prog.add_var_type(
                                                dest.to_owned(),
                                                Box::new(dest_type.to_owned()),
                                            )?;
                                            instrs.push(Instruction::AllocateVariable(
                                                dest.to_owned(),
                                                dest_type.get_byte_size(prog),
                                            ));

                                            let mut init_instrs = struct_initialiser(
                                                dest,
                                                Box::new(dest_type),
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
                    IrType::I8
                    | IrType::U8
                    | IrType::I16
                    | IrType::U16
                    | IrType::U32
                    | IrType::I64
                    | IrType::U64
                    | IrType::F32
                    | IrType::F64
                    | IrType::Void
                    | IrType::ArrayOf(_, _)
                    | IrType::PointerTo(_)
                    | IrType::Function(_, _) => return Err(MiddleEndError::InvalidDeclaration),
                    IrType::I32 => match &sq.type_specifier {
                        TypeSpecifier::Enum(_) => {}
                        _ => return Err(MiddleEndError::InvalidDeclaration),
                    },
                    IrType::Struct(_) | IrType::Union(_) => {}
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
                    let param_var = prog.new_var(ValueType::ModifiableLValue);
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
            let fun_id = prog.new_fun_body(name.to_owned(), fun)?;
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
            let dest = prog.new_var(ValueType::ModifiableLValue);
            instrs.push(Instruction::PointerToStringLiteral(
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
            let (mut unary_convert_index_instrs, index_var) = unary_convert(index_var, prog)?;
            instrs.append(&mut unary_convert_index_instrs);
            let arr_var_type = arr_var.get_type(prog)?;
            arr_var_type.require_pointer_type()?;
            let index_var_type = index_var.get_type(prog)?;
            index_var_type.require_pointer_type()?;

            // array variable is a pointer to the start of the array
            let ptr = prog.new_var(ValueType::None);
            prog.add_var_type(ptr.to_owned(), arr_var_type.to_owned())?;
            instrs.push(Instruction::Add(ptr.to_owned(), arr_var, index_var));
            let dest = prog.new_var(ValueType::ModifiableLValue);
            if let IrType::PointerTo(inner_type) = *arr_var_type {
                // always true because we already asserted arr_var_type is a pointer type
                prog.add_var_type(dest.to_owned(), inner_type)?;
            }
            instrs.push(Instruction::LoadFromAddress(dest.to_owned(), Src::Var(ptr)));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::FunctionCall(fun, params) => {
            context.in_function_name_expr = true;
            let (mut fun_instrs, fun_var) = convert_expression_to_ir(fun, prog, context)?;
            instrs.append(&mut fun_instrs);
            context.in_function_name_expr = false;

            // unary conversion
            let (mut unary_convert_fun_instrs, fun_var) = unary_convert(fun_var, prog)?;
            instrs.append(&mut unary_convert_fun_instrs);
            let fun_var_type = fun_var.get_type(prog)?;
            // must be a function pointer type
            fun_var_type.require_pointer_type()?;
            let dest_type = match *fun_var_type {
                IrType::PointerTo(t) => match *t {
                    IrType::Function(res, _) => res,
                    _ => {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Attempt to call a non-function type",
                        )))
                    }
                },
                _ => unreachable!("already asserted it's a pointer type"),
            };

            // let fun_identifier = match fun_var {
            //     Src::Var(_) | Src::Constant(_) => return Err(MiddleEndError::InvalidFunctionCall),
            //     Src::Fun(f) => f,
            // };
            let mut param_srcs: Vec<Src> = Vec::new();
            for param in params {
                let (mut param_instrs, param_var) = convert_expression_to_ir(param, prog, context)?;
                // todo function parameter passing type conversions
                instrs.append(&mut param_instrs);
                param_srcs.push(param_var);
            }
            let dest = prog.new_var(ValueType::RValue);
            prog.add_var_type(dest.to_owned(), dest_type)?;
            instrs.push(Instruction::Call(dest.to_owned(), fun_var, param_srcs));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::DirectMemberSelection(obj, Identifier(member_name)) => {
            let (mut obj_instrs, obj_var) = convert_expression_to_ir(obj, prog, context)?;
            instrs.append(&mut obj_instrs);
            let obj_var_type = obj_var.get_type(prog)?;
            obj_var_type.require_struct_or_union_type()?;

            // obj_ptr = &obj_var
            let obj_ptr = prog.new_var(obj_var.get_value_type());
            prog.add_var_type(
                obj_ptr.to_owned(),
                Box::new(IrType::PointerTo(obj_var_type.to_owned())),
            )?;
            instrs.push(Instruction::AddressOf(obj_ptr.to_owned(), obj_var));

            match *obj_var_type {
                IrType::Struct(struct_id) => {
                    let struct_type = prog.get_struct_type(&struct_id)?;
                    let member_type = struct_type.get_member_type(&member_name)?;
                    let member_byte_offset = struct_type.get_member_byte_offset(&member_name)?;

                    let ptr = prog.new_var(ValueType::None);
                    prog.add_var_type(
                        ptr.to_owned(),
                        Box::new(IrType::PointerTo(member_type.to_owned())),
                    )?;
                    // ptr = obj_ptr + (byte offset)
                    instrs.push(Instruction::Add(
                        ptr.to_owned(),
                        Src::Var(obj_ptr),
                        Src::Constant(Constant::Int(member_byte_offset as i128)),
                    ));

                    let dest = prog.new_var(ValueType::ModifiableLValue);
                    prog.add_var_type(dest.to_owned(), member_type)?;
                    // dest = *ptr
                    instrs.push(Instruction::LoadFromAddress(dest.to_owned(), Src::Var(ptr)));
                    Ok((instrs, Src::Var(dest)))
                }
                IrType::Union(union_id) => {
                    let union_type = prog.get_union_type(&union_id)?;
                    let member_type = union_type.get_member_type(&member_name)?;

                    let dest = prog.new_var(ValueType::ModifiableLValue);
                    prog.add_var_type(dest.to_owned(), member_type)?;
                    // dest = *obj_ptr
                    instrs.push(Instruction::LoadFromAddress(
                        dest.to_owned(),
                        Src::Var(obj_ptr),
                    ));
                    Ok((instrs, Src::Var(dest)))
                }
                _ => unreachable!(),
            }
        }
        Expression::IndirectMemberSelection(obj, Identifier(member_name)) => {
            let (mut obj_instrs, obj_var) = convert_expression_to_ir(obj, prog, context)?;
            instrs.append(&mut obj_instrs);
            let obj_var_type = obj_var.get_type(prog)?;
            obj_var_type.require_pointer_type()?;
            let inner_type = obj_var_type.dereference_pointer_type()?;
            inner_type.require_struct_or_union_type()?;
            match *inner_type {
                IrType::Struct(struct_id) => {
                    let struct_type = prog.get_struct_type(&struct_id)?;
                    let member_type = struct_type.get_member_type(&member_name)?;
                    let member_byte_offset = struct_type.get_member_byte_offset(&member_name)?;

                    let ptr = prog.new_var(ValueType::None);
                    prog.add_var_type(
                        ptr.to_owned(),
                        Box::new(IrType::PointerTo(member_type.to_owned())),
                    )?;
                    // ptr = (address of struct) + (byte offset)
                    instrs.push(Instruction::Add(
                        ptr.to_owned(),
                        obj_var,
                        Src::Constant(Constant::Int(member_byte_offset as i128)),
                    ));

                    let dest = prog.new_var(ValueType::ModifiableLValue);
                    prog.add_var_type(dest.to_owned(), member_type)?;
                    // dest = *ptr
                    instrs.push(Instruction::LoadFromAddress(dest.to_owned(), Src::Var(ptr)));
                    Ok((instrs, Src::Var(dest)))
                }
                IrType::Union(union_id) => {
                    let union_type = prog.get_union_type(&union_id)?;
                    let member_type = union_type.get_member_type(&member_name)?;

                    let dest = prog.new_var(ValueType::ModifiableLValue);
                    prog.add_var_type(dest.to_owned(), member_type)?;
                    // dest = *obj_ptr
                    instrs.push(Instruction::LoadFromAddress(dest.to_owned(), obj_var));
                    Ok((instrs, Src::Var(dest)))
                }
                _ => unreachable!(),
            }
        }
        Expression::PostfixIncrement(expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var(ValueType::RValue);
            // propagate the type of dest: same as src
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
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
            let dest = prog.new_var(ValueType::RValue);
            // propagate the type of dest: same as src
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
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
            // make sure the result is an rvalue
            let dest = prog.new_var(ValueType::RValue);
            // expr_var is the variable returned, after incrementing
            // check type is valid to be incremented
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
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
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    instrs.push(Instruction::SimpleAssignment(
                        dest.to_owned(),
                        Src::Var(var),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PrefixDecrement(expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            // make sure the result is an rvalue
            let dest = prog.new_var(ValueType::RValue);
            // expr_var is the variable returned, after decrementing
            // check type is valid to be incremented
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
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
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    instrs.push(Instruction::SimpleAssignment(
                        dest.to_owned(),
                        Src::Var(var),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::UnaryOp(op, expr) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            // unary convert type if necessary
            let (mut unary_convert_instrs, expr_var) = unary_convert(expr_var, prog)?;
            instrs.append(&mut unary_convert_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            match op {
                UnaryOperator::AddressOf => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::AddressOf(dest.to_owned(), expr_var));
                    // store type of dest
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(expr_var_type)))?;
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::Dereference => {
                    if this_expr_directly_on_lhs_of_assignment {
                        // store to memory address
                        let dest = prog.new_var(ValueType::ModifiableLValue);
                        match *expr_var_type {
                            IrType::PointerTo(_) => {
                                prog.add_var_type(dest.to_owned(), expr_var_type)?;
                            }
                            _ => {
                                return Err(MiddleEndError::TypeError(
                                    TypeError::DereferenceNonPointerType(expr_var_type),
                                ))
                            }
                        }
                        Ok((instrs, Src::StoreAddressVar(dest)))
                    } else {
                        // dereference load from memory address
                        let dest = prog.new_var(ValueType::ModifiableLValue);
                        instrs.push(Instruction::LoadFromAddress(dest.to_owned(), expr_var));
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
                        Ok((instrs, Src::Var(dest)))
                    }
                }
                UnaryOperator::Plus => {
                    let dest = prog.new_var(ValueType::RValue);
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
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::Minus => {
                    let dest = prog.new_var(ValueType::RValue);
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
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::BitwiseNot => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::BitwiseNot(dest.to_owned(), expr_var));
                    if !expr_var_type.is_integral_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Bitwise not of a non-integral type",
                        )));
                    }
                    // dest type is same as src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    Ok((instrs, Src::Var(dest)))
                }
                UnaryOperator::LogicalNot => {
                    let dest = prog.new_var(ValueType::RValue);
                    instrs.push(Instruction::LogicalNot(dest.to_owned(), expr_var));
                    if !expr_var_type.is_scalar_type() {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Logical not of a non-scalar type",
                        )));
                    }
                    // dest type is same as src
                    prog.add_var_type(dest.to_owned(), expr_var_type)?;
                    Ok((instrs, Src::Var(dest)))
                }
            }
        }
        Expression::SizeOfExpr(e) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(e, prog, context)?;
            instrs.append(&mut expr_instrs);
            let expr_var_type = expr_var.get_type(prog)?;
            let byte_size = expr_var_type.get_byte_size(prog);
            let dest = prog.new_var(ValueType::RValue);
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
            instrs.push(Instruction::SimpleAssignment(
                dest.to_owned(),
                Src::Constant(Constant::Int(byte_size as i128)),
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::SizeOfType(t) => {
            let (type_info, _, _) = match get_type_info(&t.0, t.1, false, prog, context)? {
                None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                Some(x) => x,
            };
            let byte_size = type_info.get_byte_size(prog);
            let dest = prog.new_var(ValueType::RValue);
            prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
            instrs.push(Instruction::SimpleAssignment(
                dest.to_owned(),
                Src::Constant(Constant::Int(byte_size as i128)),
            ));
            Ok((instrs, Src::Var(dest)))
        }
        Expression::BinaryOp(op, left, right) => {
            let (mut left_instrs, left_var) = convert_expression_to_ir(left, prog, context)?;
            instrs.append(&mut left_instrs);
            let (mut right_instrs, right_var) = convert_expression_to_ir(right, prog, context)?;
            instrs.append(&mut right_instrs);
            let dest = prog.new_var(ValueType::RValue);
            let left_var_type = left_var.get_type(prog)?;
            let right_var_type = right_var.get_type(prog)?;
            match op {
                BinaryOperator::Mult => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    left_var_type.require_arithmetic_type()?;
                    right_var_type.require_arithmetic_type()?;
                    // left_var_type and right_var_type are the same cos of binary conversion
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::Mult(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Div => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    left_var_type.require_arithmetic_type()?;
                    right_var_type.require_arithmetic_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::Div(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Mod => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::Mod(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Add => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    // must be either two arithmetic types, or a pointer and an integer
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_object_pointer_type()
                            && right_var_type.is_integral_type())
                        && !(left_var_type.is_integral_type()
                            && right_var_type.is_object_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid addition operand types",
                        )));
                    }
                    if left_var_type.is_arithmetic_type() {
                        prog.add_var_type(dest.to_owned(), left_var_type)?;
                    } else if left_var_type.is_object_pointer_type() {
                        // result is the pointer type
                        prog.add_var_type(dest.to_owned(), left_var_type)?;
                    } else {
                        prog.add_var_type(dest.to_owned(), right_var_type)?;
                    }
                    instrs.push(Instruction::Add(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::Sub => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    // must be either arithmetic - arithmetic, or pointer - integer, or pointer - pointer
                    // todo check for pointers being compatible types
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_object_pointer_type()
                            && right_var_type.is_integral_type())
                        && !(left_var_type.is_object_pointer_type()
                            && right_var_type.is_object_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid addition operand types",
                        )));
                    }
                    if left_var_type.is_arithmetic_type() {
                        prog.add_var_type(dest.to_owned(), left_var_type)?;
                    } else if right_var_type.is_integral_type() {
                        // pointer - integer
                        prog.add_var_type(dest.to_owned(), left_var_type)?;
                    } else {
                        // pointer - pointer -> long
                        prog.add_var_type(dest.to_owned(), Box::new(IrType::I64))?;
                    }
                    instrs.push(Instruction::Sub(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::LeftShift => {
                    // no binary conversion for shift operators
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::LeftShift(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::RightShift => {
                    // no binary conversion for shift operators
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::RightShift(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LessThan => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid comparison operand types",
                        )));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::LessThan(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::GreaterThan => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid comparison operand types",
                        )));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::GreaterThan(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::LessThanEq => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid comparison operand types",
                        )));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::LessThanEq(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::GreaterThanEq => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    // either both arithmetic or both pointers
                    // todo check pointer types are compatible
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid comparison operand types",
                        )));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::GreaterThanEq(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::Equal => {
                    // both arithmetic, both pointer, or pointer compared to NULL (int 0)
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_integral_type())
                        && !(left_var_type.is_integral_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid equality comparison operand types",
                        )));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::Equal(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::NotEqual => {
                    // both arithmetic, both pointer, or pointer compared to NULL (int 0)
                    if !(left_var_type.is_arithmetic_type() && right_var_type.is_arithmetic_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_pointer_type())
                        && !(left_var_type.is_pointer_type() && right_var_type.is_integral_type())
                        && !(left_var_type.is_integral_type() && right_var_type.is_pointer_type())
                    {
                        return Err(MiddleEndError::TypeError(TypeError::InvalidOperation(
                            "Invalid equality comparison operand types",
                        )));
                    }
                    // result of comparison is always int
                    prog.add_var_type(dest.to_owned(), Box::new(IrType::I32))?;
                    instrs.push(Instruction::NotEqual(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::BitwiseAnd => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::BitwiseAnd(
                        dest.to_owned(),
                        left_var,
                        right_var,
                    ));
                }
                BinaryOperator::BitwiseOr => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::BitwiseOr(dest.to_owned(), left_var, right_var));
                }
                BinaryOperator::BitwiseXor => {
                    let (mut convert_instrs, left_var, right_var) =
                        binary_convert(left_var, right_var, prog)?;
                    instrs.append(&mut convert_instrs);
                    let left_var_type = left_var.get_type(prog)?;
                    let right_var_type = right_var.get_type(prog)?;
                    left_var_type.require_integral_type()?;
                    right_var_type.require_integral_type()?;
                    prog.add_var_type(dest.to_owned(), left_var_type)?;
                    instrs.push(Instruction::BitwiseXor(
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
                    instrs.push(Instruction::LogicalOr(dest.to_owned(), left_var, right_var));
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
            let true_var_type = true_var.get_type(prog)?;

            // convert the false expr already, so we can do type checking and conversion, but don't insert
            // the instructions just yet
            let (mut false_instrs, false_var) =
                convert_expression_to_ir(false_expr, prog, context)?;
            // unary convert result of the expression
            let (mut unary_convert_false_instrs, mut false_var) = unary_convert(false_var, prog)?;
            let false_var_type = false_var.get_type(prog)?;

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
            prog.add_var_type(dest.to_owned(), true_var.get_type(prog)?)?;

            // assign the result to dest
            instrs.push(Instruction::SimpleAssignment(dest.to_owned(), true_var));
            // jump over the false instructions
            instrs.push(Instruction::Br(end_label.to_owned()));
            // false instructions
            instrs.push(Instruction::Label(false_label));
            instrs.append(&mut false_instrs);
            instrs.append(&mut unary_convert_false_instrs);
            instrs.append(&mut false_binary_convert_instrs);
            // assign the result to dest
            instrs.push(Instruction::SimpleAssignment(dest.to_owned(), false_var));
            instrs.push(Instruction::Label(end_label));

            Ok((instrs, Src::Var(dest)))
        }
        Expression::Assignment(dest_expr, src_expr) => {
            let (mut src_expr_instrs, mut src_var) =
                convert_expression_to_ir(src_expr, prog, context)?;
            instrs.append(&mut src_expr_instrs);
            let src_var_type = src_var.get_type(prog)?;

            context.directly_on_lhs_of_assignment = true;
            let (mut dest_expr_instrs, dest_var) =
                convert_expression_to_ir(dest_expr, prog, context)?;
            context.directly_on_lhs_of_assignment = false;
            instrs.append(&mut dest_expr_instrs);

            // check that we're assigning to an lvalue
            if !dest_var.get_value_type().is_modifiable_lvalue() {
                return Err(MiddleEndError::AttemptToModifyNonLValue);
            }

            let mut dest_var_type = dest_var.get_type(prog)?;
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
                instrs.push(Instruction::StoreToAddress(dest.to_owned(), src_var));
            } else {
                instrs.push(Instruction::SimpleAssignment(dest.to_owned(), src_var));
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
            let expr_var_type = expr_var.get_type(prog)?;
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

type FunctionParameterBindings = Vec<(String, Box<IrType>)>;

fn get_type_info(
    specifier: &SpecifierQualifier,
    declarator: Option<Box<Declarator>>,
    is_duplicate_specifier: bool,
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
                // check if this is referencing a previous struct declaration
                match context.resolve_struct_tag_to_struct_id(&struct_name) {
                    Ok(struct_id) => Box::new(IrType::Struct(struct_id)),
                    Err(MiddleEndError::UndeclaredStructTag(_)) => {
                        let struct_type_id =
                            prog.add_struct_type(StructType::named(struct_name.to_owned()))?;
                        context
                            .add_struct_tag(struct_name.to_owned(), struct_type_id.to_owned())?;
                        Box::new(IrType::Struct(struct_type_id))
                    }
                    Err(e) => return Err(e),
                }
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
                            false,
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
                if let Some(Identifier(name)) = struct_name {
                    context.add_struct_tag(name.to_owned(), struct_type_id.to_owned())?;
                }
                Box::new(IrType::Struct(struct_type_id))
            }
        },
        TypeSpecifier::Union(union_type) => match union_type {
            AstUnionType::Declaration(Identifier(union_name)) => {
                // check if this is referencing a previous union declaration
                match context.resolve_union_tag_to_union_id(&union_name) {
                    Ok(union_id) => Box::new(IrType::Union(union_id)),
                    Err(MiddleEndError::UndeclaredUnionTag(_)) => {
                        let union_type_id =
                            prog.add_union_type(UnionType::named(union_name.to_owned()))?;
                        context.add_union_tag(union_name.to_owned(), union_type_id.to_owned())?;
                        Box::new(IrType::Union(union_type_id))
                    }
                    Err(e) => return Err(e),
                }
            }
            AstUnionType::Definition(union_name, members) => {
                let mut union_type = match union_name {
                    Some(Identifier(name)) => UnionType::named(name.to_owned()),
                    None => UnionType::unnamed(),
                };
                for member in members {
                    for decl in &member.1 {
                        let (member_type_info, member_name, _params) = match get_type_info(
                            &member.0,
                            Some(Box::new(*decl.to_owned())),
                            false,
                            prog,
                            context,
                        )? {
                            None => return Err(MiddleEndError::InvalidTypedefDeclaration),
                            Some(x) => x,
                        };
                        if member_name == None {
                            return Err(MiddleEndError::UnnamedUnionMember);
                        }
                        union_type.push_member(member_name.unwrap(), member_type_info, prog)?;
                    }
                }
                let union_type_id = prog.add_union_type(union_type)?;
                if let Some(Identifier(name)) = union_name {
                    context.add_union_tag(name.to_owned(), union_type_id.to_owned())?;
                }
                Box::new(IrType::Union(union_type_id))
            }
        },
        TypeSpecifier::Enum(enum_type) => {
            match enum_type {
                EnumType::Declaration(Identifier(enum_name)) => {
                    context.resolve_identifier_to_enum_tag(&enum_name)?;
                    // enums are just integers
                    Box::new(IrType::I32)
                }
                EnumType::Definition(enum_name, enum_constants) => {
                    let mut skip_constant_definition = false;
                    if let Some(Identifier(enum_name)) = enum_name {
                        if is_duplicate_specifier {
                            context.resolve_identifier_to_enum_tag(&enum_name)?;
                            skip_constant_definition = true;
                        } else {
                            context.add_enum_tag(enum_name.to_owned())?;
                        }
                    }
                    if !skip_constant_definition {
                        let mut next_constant_value = 0;
                        for enum_constant in enum_constants {
                            match enum_constant {
                                Enumerator::Simple(Identifier(name)) => {
                                    context
                                        .add_enum_constant(name.to_owned(), next_constant_value)?;
                                    next_constant_value += 1;
                                }
                                Enumerator::WithValue(Identifier(name), value_expr) => {
                                    let value = eval_integral_constant_expression(
                                        value_expr.to_owned(),
                                        prog,
                                    )?
                                        as EnumConstant;
                                    context.add_enum_constant(name.to_owned(), value)?;
                                    // value of next constant without explicit value is one more than
                                    // the last constant
                                    next_constant_value = value + 1;
                                }
                            }
                        }
                    }
                    Box::new(IrType::I32)
                }
            }
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
            // todo storage class specifiers
            println!("ignoring storage class specifier: auto")
        }
        Some(StorageClassSpecifier::Extern) => {
            // todo storage class specifiers
            println!("ignoring storage class specifier: extern")
        }
        Some(StorageClassSpecifier::Register) => {
            // todo storage class specifiers
            println!("ignoring storage class specifier: register")
        }
        Some(StorageClassSpecifier::Static) => {
            // todo storage class specifiers
            println!("ignoring storage class specifier: static")
        }
    }

    match declarator {
        Some(decl) => {
            let (ir_type, decl_name, param_bindings) =
                add_type_info_from_declarator(decl, ir_type, prog, context)?;
            if is_typedef {
                context.add_typedef(decl_name.unwrap(), ir_type)?;
                return Ok(None);
            }
            Ok(Some((ir_type, decl_name, param_bindings)))
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
) -> Result<
    (
        Box<IrType>,
        Option<String>,
        Option<FunctionParameterBindings>,
    ),
    MiddleEndError,
> {
    match *decl {
        Declarator::Identifier(Identifier(name)) => Ok((type_info, Some(name), None)),
        Declarator::PointerDeclarator(d) => {
            add_type_info_from_declarator(d, type_info.wrap_with_pointer(), prog, context)
        }
        Declarator::AbstractPointerDeclarator => {
            // Err(MiddleEndError::InvalidAbstractDeclarator)
            // todo handle abstract parameters in function declaration
            Ok((type_info.wrap_with_pointer(), None, None))
        }
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
                    match get_type_info(&p.0, p.1, false, prog, context)? {
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

fn unary_convert(
    src: Src,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    let src_type = src.get_type(prog)?;
    let unary_converted_type = src_type.unary_convert();
    if src_type != unary_converted_type {
        let (instrs, converted_var) =
            get_type_conversion_instrs(src, src_type, unary_converted_type.to_owned(), prog)?;
        return Ok((instrs, converted_var));
    }
    Ok((Vec::new(), src))
}

fn binary_convert(
    left: Src,
    right: Src,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src, Src), MiddleEndError> {
    let (mut left_convert_instrs, mut right_convert_instrs, left_result, right_result) =
        binary_convert_separately(left, right, prog)?;
    left_convert_instrs.append(&mut right_convert_instrs);
    Ok((left_convert_instrs, left_result, right_result))
}

fn binary_convert_separately(
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
    let left_type = unary_left.get_type(prog)?;
    let right_type = unary_right.get_type(prog)?;
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

fn convert_type_for_assignment(
    src: Src,
    src_type: Box<IrType>,
    dest_type: Box<IrType>,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    if src_type.is_arithmetic_type() && dest_type.is_arithmetic_type() {
        let (convert_instrs, converted_var) =
            get_type_conversion_instrs(src, src_type, dest_type, prog)?;
        return Ok((convert_instrs, converted_var));
    }
    todo!("other options of possible type combinations")
}

fn get_type_conversion_instrs(
    src: Src,
    src_type: Box<IrType>,
    dest_type: Box<IrType>,
    prog: &mut Box<Program>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    println!("convert {}: {} to {}", src, src_type, dest_type);
    let mut instrs = Vec::new();
    if src_type == dest_type {
        return Ok((instrs, src));
    }
    match (*src_type, *dest_type) {
        // cast to void *
        (_src_type, IrType::PointerTo(t)) if *t == IrType::Void => Ok((instrs, src)),
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

        (IrType::Function(_, _), IrType::PointerTo(t))
        | (IrType::ArrayOf(_, _), IrType::PointerTo(t)) => {
            let dest = prog.new_var(src.get_value_type());
            prog.add_var_type(dest.to_owned(), Box::new(IrType::PointerTo(t)))?;
            instrs.push(Instruction::AddressOf(dest.to_owned(), src));
            Ok((instrs, Src::Var(dest)))
        }
        (s, d) => {
            return Err(MiddleEndError::TypeError(TypeError::TypeConversionError(
                "Cannot convert type",
                Box::new(s),
                Some(Box::new(d)),
            )))
        } // todo rest of types
    }
}

fn array_initialiser(
    dest: VarId,
    dest_type_info: Box<IrType>,
    initialiser_list: Vec<Box<Initialiser>>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<Vec<Instruction>, MiddleEndError> {
    let mut instrs = Vec::new();

    let array_member_type = dest_type_info.unwrap_array_type()?;
    let array_member_byte_size = array_member_type.get_byte_size(prog);

    // pointer to the array member we're currently initialising
    let member_ptr_var = prog.new_var(ValueType::ModifiableLValue);
    prog.add_var_type(
        member_ptr_var.to_owned(),
        Box::new(IrType::PointerTo(array_member_type.to_owned())),
    )?;
    instrs.push(Instruction::AddressOf(
        member_ptr_var.to_owned(),
        Src::Var(dest),
    ));

    // check that the array length matches the number of initialisers
    if dest_type_info.get_array_size()? as usize != initialiser_list.len() {
        return Err(MiddleEndError::TypeError(
            TypeError::MismatchedArrayInitialiserLength,
        ));
    }

    for array_member_initialiser in initialiser_list {
        match *array_member_initialiser {
            Initialiser::Expr(e) => {
                if array_member_type.is_aggregate_type() {
                    return Err(MiddleEndError::TypeError(
                        TypeError::AssignNonAggregateValueToAggregateType,
                    ));
                }
                let (mut expr_instrs, mut expr_var) = convert_expression_to_ir(e, prog, context)?;
                instrs.append(&mut expr_instrs);

                // check type of the expression and convert if necessary
                let expr_var_type = expr_var.get_type(prog)?;
                if expr_var_type != array_member_type {
                    if let Src::Constant(c) = &expr_var {
                        let temp = prog.new_var(ValueType::RValue);
                        prog.add_var_type(
                            temp.to_owned(),
                            c.get_type(Some(array_member_type.to_owned())),
                        )?;
                        instrs.push(Instruction::SimpleAssignment(temp.to_owned(), expr_var));
                        expr_var = Src::Var(temp);
                    }
                    let (mut convert_instrs, converted_var) = convert_type_for_assignment(
                        expr_var.to_owned(),
                        expr_var.get_type(prog)?,
                        array_member_type.to_owned(),
                        prog,
                    )?;
                    instrs.append(&mut convert_instrs);
                    expr_var = converted_var;
                }

                instrs.push(Instruction::StoreToAddress(
                    member_ptr_var.to_owned(),
                    expr_var,
                ));
            }
            Initialiser::List(sub_member_initialisers) => match *array_member_type.to_owned() {
                IrType::ArrayOf(sub_member_type, size) => {
                    // initialise nested array
                    let mut init_instrs = array_initialiser(
                        member_ptr_var.to_owned(),
                        Box::new(IrType::ArrayOf(sub_member_type, size)),
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
                        Box::new(IrType::Struct(struct_id)),
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
            member_ptr_var.to_owned(),
            Src::Var(member_ptr_var.to_owned()),
            Src::Constant(Constant::Int(array_member_byte_size as i128)),
        ));
    }

    Ok(instrs)
}

fn struct_initialiser(
    dest: VarId,
    dest_type_info: Box<IrType>,
    initialiser_list: Vec<Box<Initialiser>>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<Vec<Instruction>, MiddleEndError> {
    let mut instrs = Vec::new();

    let struct_type = dest_type_info.unwrap_struct_type(prog)?;

    // check that the number of initialisers matches the number of struct members
    if struct_type.member_count() != initialiser_list.len() {
        println!(
            "struct member count: {}, initialiser count: {}",
            struct_type.member_count(),
            initialiser_list.len()
        );
        return Err(MiddleEndError::TypeError(
            TypeError::MismatchedArrayInitialiserLength,
        ));
    }

    for member_index in 0..struct_type.member_count() {
        let mut member_initialiser = initialiser_list.get(member_index).unwrap().to_owned();
        let member_type = struct_type.get_member_type_by_index(member_index)?;
        let member_byte_offset = struct_type.get_member_byte_offset_by_index(member_index)?;

        // pointer to the struct member we're currently initialising
        let member_ptr_var = prog.new_var(ValueType::ModifiableLValue);
        prog.add_var_type(
            member_ptr_var.to_owned(),
            Box::new(IrType::PointerTo(member_type.to_owned())),
        )?;
        // member_ptr_var = &dest + byte_offset
        instrs.push(Instruction::AddressOf(
            member_ptr_var.to_owned(),
            Src::Var(dest.to_owned()),
        ));
        instrs.push(Instruction::Add(
            member_ptr_var.to_owned(),
            Src::Var(member_ptr_var.to_owned()),
            Src::Constant(Constant::Int(member_byte_offset as i128)),
        ));

        // check for case of initialising a char array with a string literal
        if let IrType::ArrayOf(_, _) = *member_type {
            match *member_initialiser.to_owned() {
                Initialiser::Expr(e) => {
                    if let Expression::StringLiteral(s) = *e.to_owned() {
                        // convert string literal to array of chars
                        member_initialiser = convert_string_literal_to_init_list_of_chars_ast(s);
                    }
                }
                Initialiser::List(inits) => {
                    if inits.len() == 1 {
                        if let Initialiser::Expr(e) = &**inits.first().unwrap() {
                            if let Expression::StringLiteral(s) = *e.to_owned() {
                                // convert string literal in braces to array of chars
                                member_initialiser =
                                    convert_string_literal_to_init_list_of_chars_ast(s);
                            }
                        }
                    }
                }
            }
        }

        match *member_initialiser {
            Initialiser::Expr(e) => {
                if member_type.is_aggregate_type() {
                    return Err(MiddleEndError::TypeError(
                        TypeError::AssignNonAggregateValueToAggregateType,
                    ));
                }

                let (mut expr_instrs, mut expr_var) = convert_expression_to_ir(e, prog, context)?;
                instrs.append(&mut expr_instrs);

                // check type of the expression and convert if necessary
                let expr_var_type = expr_var.get_type(prog)?;
                if expr_var_type != member_type {
                    if let Src::Constant(c) = &expr_var {
                        let temp = prog.new_var(ValueType::RValue);
                        prog.add_var_type(
                            temp.to_owned(),
                            c.get_type(Some(member_type.to_owned())),
                        )?;
                        instrs.push(Instruction::SimpleAssignment(temp.to_owned(), expr_var));
                        expr_var = Src::Var(temp);
                    }
                    let (mut convert_instrs, converted_var) = convert_type_for_assignment(
                        expr_var.to_owned(),
                        expr_var.get_type(prog)?,
                        member_type.to_owned(),
                        prog,
                    )?;
                    instrs.append(&mut convert_instrs);
                    expr_var = converted_var;
                }

                instrs.push(Instruction::StoreToAddress(member_ptr_var, expr_var));
            }
            Initialiser::List(sub_member_initialisers) => match *member_type.to_owned() {
                IrType::ArrayOf(sub_member_type, size) => {
                    // initialise nested array
                    let mut init_instrs = array_initialiser(
                        member_ptr_var,
                        Box::new(IrType::ArrayOf(sub_member_type, size)),
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
                        Box::new(IrType::Struct(struct_id)),
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
fn convert_string_literal_to_init_list_of_chars_ast(s: String) -> Box<Initialiser> {
    let mut char_initialisers = Vec::new();
    for c in s.chars() {
        char_initialisers.push(Box::new(Initialiser::Expr(Box::new(Expression::Constant(
            AstConstant::Char(c),
        )))));
    }
    // string terminating char
    char_initialisers.push(Box::new(Initialiser::Expr(Box::new(Expression::Constant(
        AstConstant::Char('\0'),
    )))));
    Box::new(Initialiser::List(char_initialisers))
}
