use super::ir::Program;
use crate::middle_end::compile_time_eval::eval_constant_expression;
use crate::middle_end::ir::{
    Constant, Fun, Function, Instruction, Label, Src, Type, TypeInfo, Var,
};
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::parser::ast;
use crate::parser::ast::{
    ArithmeticType, BinaryOperator, Declarator, DeclaratorInitialiser, Expression,
    ExpressionOrDeclaration, Identifier, LabelledStatement, ParameterTypeList,
    Program as AstProgram, SpecifierQualifier, Statement, TypeSpecifier, UnaryOperator,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
struct LoopContext {
    start_label: Label,
    end_label: Label,
    continue_label: Label,
}

impl LoopContext {
    fn while_loop(start_label: Label, end_label: Label) -> Self {
        LoopContext {
            start_label,
            end_label,
            continue_label: start_label,
        }
    }

    fn do_while_loop(start_label: Label, end_label: Label, continue_label: Label) -> Self {
        LoopContext {
            start_label,
            end_label,
            continue_label,
        }
    }

    fn for_loop(start_label: Label, end_label: Label, continue_label: Label) -> Self {
        LoopContext {
            start_label,
            end_label,
            continue_label,
        }
    }
}

#[derive(Debug)]
struct SwitchContext {
    end_label: Label,
    switch_var: Var,
    default_case: Option<Vec<Instruction>>,
}

impl SwitchContext {
    fn new(end_label: Label, switch_var: Var) -> Self {
        SwitchContext {
            end_label,
            switch_var,
            default_case: None,
        }
    }

    fn add_default_case(&mut self, body: Vec<Instruction>) -> Result<(), MiddleEndError> {
        match self.default_case {
            None => {
                self.default_case = Some(body);
                Ok(())
            }
            Some(_) => Err(MiddleEndError::MultipleDefaultCasesInSwitch),
        }
    }
}

#[derive(Debug)]
enum LoopOrSwitchContext {
    Loop(LoopContext),
    Switch(SwitchContext),
}

#[derive(Debug)]
struct Scope {
    variables: HashMap<String, Var>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            variables: HashMap::new(),
        }
    }

    fn new_var(&mut self, identifier_name: String, var: Var) -> Result<(), MiddleEndError> {
        todo!()
    }

    fn resolve_identifier_to_var(&self, identifier_name: &str) -> Option<Var> {
        match self.variables.get(identifier_name) {
            None => None,
            Some(var) => Some(var.to_owned()),
        }
    }
}

#[derive(Debug)]
struct Context {
    loop_stack: Vec<LoopOrSwitchContext>,
    scope_stack: Vec<Scope>,
    in_function_name_expr: bool,
}

impl Context {
    fn new() -> Self {
        Context {
            loop_stack: Vec::new(),
            scope_stack: Vec::new(),
            in_function_name_expr: false,
        }
    }

    fn push_loop(&mut self, loop_context: LoopContext) {
        self.loop_stack
            .push(LoopOrSwitchContext::Loop(loop_context));
    }

    fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn push_switch(&mut self, switch_context: SwitchContext) {
        self.loop_stack
            .push(LoopOrSwitchContext::Switch(switch_context));
    }

    fn pop_switch(&mut self) -> Result<SwitchContext, MiddleEndError> {
        match self.loop_stack.pop() {
            None | Some(LoopOrSwitchContext::Loop(_)) => Err(MiddleEndError::LoopNestingError),
            Some(LoopOrSwitchContext::Switch(switch_context)) => Ok(switch_context),
        }
    }

    fn get_break_label(&self) -> Option<Label> {
        match self.loop_stack.last() {
            None => None,
            Some(LoopOrSwitchContext::Loop(loop_context)) => Some(loop_context.end_label),
            Some(LoopOrSwitchContext::Switch(switch_context)) => Some(switch_context.end_label),
        }
    }

    fn get_continue_label(&self) -> Option<Label> {
        if self.loop_stack.is_empty() {
            return None;
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return None,
                Some(LoopOrSwitchContext::Loop(loop_context)) => {
                    return Some(loop_context.continue_label);
                }
                Some(LoopOrSwitchContext::Switch(_)) => {}
            }
            // if context was a switch context, keep looking backwards for the top loop context
            i -= 1;
        }
    }

    fn is_in_switch_context(&self) -> bool {
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return false,
                Some(LoopOrSwitchContext::Switch(_)) => return true,
                Some(LoopOrSwitchContext::Loop(_)) => {}
            }
            i -= 1;
        }
    }

    fn get_switch_variable(&self) -> Option<Var> {
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return None,
                Some(LoopOrSwitchContext::Switch(switch_context)) => {
                    return Some(switch_context.switch_var);
                }
                _ => {}
            }
            i -= 1;
        }
    }

    fn add_default_switch_case(&mut self, body: Vec<Instruction>) -> Result<(), MiddleEndError> {
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get_mut(i) {
                None => return Err(MiddleEndError::DefaultOutsideSwitchContext),
                Some(loop_or_switch) => match loop_or_switch {
                    LoopOrSwitchContext::Loop(_) => {}
                    LoopOrSwitchContext::Switch(switch_context) => {
                        return switch_context.add_default_case(body);
                    }
                },
            }
            i -= 1;
        }
    }

    fn push_scope(&mut self, scope: Scope) {
        self.scope_stack.push(scope);
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn resolve_identifier_to_var(&self, identifier_name: &str) -> Result<Var, MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::UndeclaredIdentifier(
                identifier_name.to_owned(),
            ));
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => {
                    return Err(MiddleEndError::UndeclaredIdentifier(
                        identifier_name.to_owned(),
                    ))
                }
                Some(scope) => match scope.resolve_identifier_to_var(identifier_name) {
                    None => {}
                    Some(var) => return Ok(var),
                },
            }
            if i == 0 {
                return Err(MiddleEndError::UndeclaredIdentifier(
                    identifier_name.to_owned(),
                ));
            }
            i -= 1;
        }
    }

    fn resolve_identifier_to_fun(&self, identifier_name: &str) -> Result<Fun, MiddleEndError> {
        todo!()
    }
}

pub fn convert_to_ir(ast: AstProgram) {
    let mut program = Box::new(Program::new());
    let mut context = Box::new(Context::new());
    for stmt in ast.0 {
        let instrs = convert_statement_to_ir(stmt, &mut program, &mut context);
        println!("{:?}", instrs);
    }
    println!("Program: {:?}\nContext: {:?}", program, context);
}

// fn convert_function_to_ir(stmt: Statement) {
//     let mut function = Function { instrs: Vec::new() };
// }

fn convert_statement_to_ir(
    stmt: Box<Statement>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<Vec<Instruction>, MiddleEndError> {
    let mut instrs: Vec<Instruction> = Vec::new();
    match *stmt {
        Statement::Block(stmts) => {
            instrs.push(Instruction::StartBlock);
            for s in stmts {
                instrs.append(&mut convert_statement_to_ir(s, prog, context)?);
            }
            instrs.push(Instruction::EndBlock);
        }
        Statement::Goto(x) => match prog.label_identifiers.get(&x.0) {
            Some(label) => instrs.push(Instruction::Br(label.to_owned())),
            None => {
                let label = prog.new_label();
                prog.label_identifiers.insert(x.0, label);
                instrs.push(Instruction::Br(label));
            }
        },
        Statement::Continue => match context.get_continue_label() {
            None => {
                return Err(MiddleEndError::ContinueOutsideLoopContext);
            }
            Some(label) => {
                instrs.push(Instruction::Br(label));
            }
        },
        Statement::Break => match context.get_break_label() {
            None => {
                return Err(MiddleEndError::BreakOutsideLoopOrSwitchContext);
            }
            Some(label) => {
                instrs.push(Instruction::Br(label));
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
            instrs.push(Instruction::Label(loop_start_label));
            context.push_loop(LoopContext::while_loop(loop_start_label, loop_end_label));
            // while condition
            let (mut cond_instrs, cond_var) = convert_expression_to_ir(cond, prog, context)?;
            instrs.append(&mut cond_instrs);
            // jump out of loop if condition false
            instrs.push(Instruction::BrIfEq(
                cond_var,
                Src::Constant(Constant::Int(0)),
                loop_end_label,
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
            instrs.push(Instruction::Label(loop_start_label));
            context.push_loop(LoopContext::do_while_loop(
                loop_start_label,
                loop_end_label,
                loop_continue_label,
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
            instrs.push(Instruction::Label(loop_start_label));
            context.push_loop(LoopContext::for_loop(
                loop_start_label,
                loop_end_label,
                loop_continue_label,
            ));
            // condition
            let cond_var = match cond {
                None => {
                    let temp = prog.new_var();
                    instrs.push(Instruction::SimpleAssignment(
                        temp,
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
                loop_end_label,
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
                if_end_label,
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
                else_label,
            ));
            // if body
            instrs.append(&mut convert_statement_to_ir(true_body, prog, context)?);
            // jump to after else body
            let else_end_label = prog.new_label();
            instrs.push(Instruction::Br(else_end_label));
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
                    instrs.push(Instruction::SimpleAssignment(temp, Src::Constant(c)));
                    temp
                }
                Src::Fun(fun) => unreachable!(),
            };
            context.push_switch(SwitchContext::new(switch_end_label, switch_var));
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
                LabelledStatement::Named(ast::Identifier(label_name), stmt) => {
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
                        end_of_case_label,
                    ));
                    // case body
                    instrs.append(&mut convert_statement_to_ir(stmt, prog, context)?);
                    // end of case label
                    instrs.push(Instruction::Label(end_of_case_label));
                }
                LabelledStatement::Default(stmt) => {
                    let body_instrs = convert_statement_to_ir(stmt, prog, context)?;
                    context.add_default_switch_case(body_instrs)?;
                }
            }
        }
        Statement::Expr(e) => {
            println!(
                "evaluated: {:?}",
                eval_constant_expression(e.to_owned(), prog)
            );
            let (mut expr_instrs, _) = convert_expression_to_ir(e, prog, context)?;
            instrs.append(&mut expr_instrs);
        }
        Statement::Declaration(specifier, declarators) => {
            for declarator in declarators {
                let type_info = TypeInfo::new();
                match declarator {
                    DeclaratorInitialiser::NoInit(d) => {
                        todo!()
                    }
                    DeclaratorInitialiser::Init(d, init_expr) => {
                        todo!()
                    }
                }
            }
        }
        Statement::EmptyDeclaration(_) => {
            todo!()
        }
        Statement::FunctionDeclaration(_, _, _) => {
            todo!()
        }
        Statement::Empty => {}
    }
    Ok(instrs)
}

/// returns the list of instructions generated, and the name of the temp variable
/// the result is assigned to
fn convert_expression_to_ir(
    expr: Box<Expression>,
    prog: &mut Box<Program>,
    context: &mut Box<Context>,
) -> Result<(Vec<Instruction>, Src), MiddleEndError> {
    let mut instrs: Vec<Instruction> = Vec::new();
    match *expr {
        Expression::Identifier(ast::Identifier(name)) => {
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
            let var = prog.new_string_literal(s);
            Ok((instrs, Src::Var(var)))
        }
        Expression::Index(arr, index) => {
            let (mut arr_instrs, arr_var) = convert_expression_to_ir(arr, prog, context)?;
            instrs.append(&mut arr_instrs);
            let (mut index_instrs, index_var) = convert_expression_to_ir(index, prog, context)?;
            instrs.append(&mut index_instrs);
            // array variable is a pointer to the start of the array
            let ptr = prog.new_var();
            instrs.push(Instruction::Add(ptr, arr_var, index_var));
            let dest = prog.new_var();
            instrs.push(Instruction::Dereference(dest, Src::Var(ptr)));
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
                dest,
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
            instrs.push(Instruction::SimpleAssignment(dest, expr_var.to_owned()));
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Add(
                        var,
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PostfixDecrement(_) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            let dest = prog.new_var();
            instrs.push(Instruction::SimpleAssignment(dest, expr_var.to_owned()));
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Sub(
                        var,
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
            Ok((instrs, Src::Var(dest)))
        }
        Expression::PrefixIncrement(_) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Add(
                        var,
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                    Ok((instrs, Src::Var(var)))
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
        }
        Expression::PrefixDecrement(_) => {
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            match expr_var {
                Src::Var(var) => {
                    instrs.push(Instruction::Sub(
                        var,
                        Src::Var(var),
                        Src::Constant(Constant::Int(1)),
                    ));
                    Ok((instrs, Src::Var(var)))
                }
                _ => return Err(MiddleEndError::InvalidLValue),
            }
        }
        Expression::UnaryOp(op, expr) => {
            let dest = prog.new_var();
            let (mut expr_instrs, expr_var) = convert_expression_to_ir(expr, prog, context)?;
            instrs.append(&mut expr_instrs);
            match op {
                UnaryOperator::AddressOf => {
                    instrs.push(Instruction::AddressOf(dest, expr_var));
                }
                UnaryOperator::Dereference => {
                    instrs.push(Instruction::Dereference(dest, expr_var));
                }
                UnaryOperator::Plus => {
                    instrs.push(Instruction::Add(
                        dest,
                        Src::Constant(Constant::Int(0)),
                        expr_var,
                    ));
                }
                UnaryOperator::Minus => {
                    instrs.push(Instruction::Sub(
                        dest,
                        Src::Constant(Constant::Int(0)),
                        expr_var,
                    ));
                }
                UnaryOperator::BitwiseNot => {
                    instrs.push(Instruction::BitwiseNot(dest, expr_var));
                }
                UnaryOperator::LogicalNot => {
                    instrs.push(Instruction::LogicalNot(dest, expr_var));
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
        Expression::BinaryOp(op, left, right) => match op {
            BinaryOperator::Mult => {
                let dest = prog.new_var();
                let (mut left_instrs, left_var) = convert_expression_to_ir(left, prog, context)?;
                instrs.append(&mut left_instrs);
                let (mut right_instrs, right_var) = convert_expression_to_ir(right, prog, context)?;
                instrs.append(&mut right_instrs);
                instrs.push(Instruction::Mult(dest, left_var, right_var));
                Ok((instrs, Src::Var(dest)))
            }
            BinaryOperator::Div => {
                todo!()
            }
            BinaryOperator::Mod => {
                todo!()
            }
            BinaryOperator::Add => {
                todo!()
            }
            BinaryOperator::Sub => {
                todo!()
            }
            BinaryOperator::LeftShift => {
                todo!()
            }
            BinaryOperator::RightShift => {
                todo!()
            }
            BinaryOperator::LessThan => {
                todo!()
            }
            BinaryOperator::GreaterThan => {
                todo!()
            }
            BinaryOperator::LessThanEq => {
                todo!()
            }
            BinaryOperator::GreaterThanEq => {
                todo!()
            }
            BinaryOperator::Equal => {
                todo!()
            }
            BinaryOperator::NotEqual => {
                todo!()
            }
            BinaryOperator::BitwiseAnd => {
                todo!()
            }
            BinaryOperator::BitwiseOr => {
                todo!()
            }
            BinaryOperator::BitwiseXor => {
                todo!()
            }
            BinaryOperator::LogicalAnd => {
                todo!()
            }
            BinaryOperator::LogicalOr => {
                todo!()
            }
        },
        Expression::Ternary(_, _, _) => {
            todo!()
        }
        Expression::Assignment(_, _, _) => {
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

fn get_type_info(
    specifier: SpecifierQualifier,
    declarator: Option<Box<Declarator>>,
    prog: &mut Box<Program>,
) -> Result<(Box<TypeInfo>, Option<String>), MiddleEndError> {
    let mut type_info = Box::new(TypeInfo::new());
    match specifier.type_specifier {
        TypeSpecifier::ArithmeticType(t) => match t {
            ArithmeticType::I8 => {
                type_info.type_ = Type::I8;
            }
            ArithmeticType::U8 => {
                type_info.type_ = Type::U8;
            }
            ArithmeticType::I16 => {
                type_info.type_ = Type::I16;
            }
            ArithmeticType::U16 => {
                type_info.type_ = Type::U16;
            }
            ArithmeticType::I32 => {
                type_info.type_ = Type::I32;
            }
            ArithmeticType::U32 => {
                type_info.type_ = Type::U32;
            }
            ArithmeticType::I64 => {
                type_info.type_ = Type::I64;
            }
            ArithmeticType::U64 => {
                type_info.type_ = Type::U64;
            }
            ArithmeticType::F32 => {
                type_info.type_ = Type::F32;
            }
            ArithmeticType::F64 => {
                type_info.type_ = Type::F64;
            }
        },
        TypeSpecifier::Void => {
            type_info.type_ = Type::Void;
        }
        TypeSpecifier::Struct(_) => {
            todo!()
        }
        TypeSpecifier::Union(_) => {
            todo!()
        }
        TypeSpecifier::Enum(_) => {
            todo!()
        }
        TypeSpecifier::CustomType(Identifier(name)) => {
            type_info = Box::new(prog.resolve_typedef(&name)?)
        }
    }

    match declarator {
        Some(decl) => {
            let decl_name = get_type_info_from_declarator(decl, &mut type_info, prog)?;
            Ok((type_info, Some(decl_name)))
        }
        None => Ok((type_info, None)),
    }
}

/// Modifies the TypeInfo struct it's given, and returns the identifier name
fn get_type_info_from_declarator(
    decl: Box<Declarator>,
    type_info: &mut Box<TypeInfo>,
    prog: &mut Box<Program>,
) -> Result<String, MiddleEndError> {
    match *decl {
        Declarator::Identifier(Identifier(name)) => Ok(name),
        Declarator::PointerDeclarator(d) => {
            type_info.wrap_with_pointer();
            get_type_info_from_declarator(d, type_info, prog)
        }
        Declarator::AbstractPointerDeclarator => Err(MiddleEndError::InvalidAbstractDeclarator),
        Declarator::ArrayDeclarator(d, size_expr) => {
            let size = match size_expr {
                None => None,
                Some(size_expr) => Some(eval_constant_expression(size_expr, prog)? as u64),
            };
            type_info.wrap_with_array(size);
            get_type_info_from_declarator(d, type_info, prog)
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

            let mut param_types: Vec<Box<TypeInfo>> = Vec::new();
            for p in param_decls {
                let (param_type, param_name) = get_type_info(p.0, p.1, prog)?;
                param_types.push(param_type);
            }

            type_info.wrap_with_fun(param_types);

            get_type_info_from_declarator(d, type_info, prog)
        }
    }
}
