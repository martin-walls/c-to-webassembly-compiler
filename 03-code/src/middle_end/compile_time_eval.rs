use crate::middle_end::instructions::Constant as IrConstant;
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::parser::ast::{BinaryOperator, Expression};
use crate::parser::ast::{Constant as AstConstant, UnaryOperator};

/// constant expression used for array bounds, explicit enum values,
/// values of case labels. Must evaluate to an integer
pub fn eval_integral_constant_expression(expr: Box<Expression>) -> Result<i128, MiddleEndError> {
    match eval(expr)? {
        ConstantExpressionType::Int(i) => Ok(i),
        _ => Err(MiddleEndError::InvalidConstantExpression),
    }
}

pub fn eval_initialiser_constant_expression(
    expr: Box<Expression>,
) -> Result<IrConstant, MiddleEndError> {
    match eval(expr)? {
        ConstantExpressionType::Int(i) => Ok(IrConstant::Int(i)),
        ConstantExpressionType::Float(f) => Ok(IrConstant::Float(f)),
    }
}

enum ConstantExpressionType {
    Int(i128),
    Float(f64),
}

fn eval(expr: Box<Expression>) -> Result<ConstantExpressionType, MiddleEndError> {
    match *expr {
        Expression::Identifier(_) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }
        Expression::Constant(c) => match c {
            AstConstant::Int(i) => Ok(ConstantExpressionType::Int(i as i128)),
            AstConstant::Float(f) => Ok(ConstantExpressionType::Float(f)),
            AstConstant::Char(c) => Ok(ConstantExpressionType::Int(c as i128)),
        },
        Expression::StringLiteral(_) => Err(MiddleEndError::InvalidConstantExpression),
        Expression::Index(_, _) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }
        Expression::DirectMemberSelection(_, _) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }
        Expression::IndirectMemberSelection(_, _) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }
        Expression::UnaryOp(op, expr) => {
            let expr_result = eval(expr)?;
            match op {
                UnaryOperator::AddressOf => {
                    // todo
                    Err(MiddleEndError::CantEvaluateAtCompileTime)
                }
                UnaryOperator::Dereference => {
                    // todo
                    Err(MiddleEndError::CantEvaluateAtCompileTime)
                }
                UnaryOperator::Plus => match expr_result {
                    ConstantExpressionType::Int(_) | ConstantExpressionType::Float(_) => {
                        Ok(expr_result)
                    }
                },
                UnaryOperator::Minus => match expr_result {
                    ConstantExpressionType::Int(i) => Ok(ConstantExpressionType::Int(-i)),
                    ConstantExpressionType::Float(f) => Ok(ConstantExpressionType::Float(-f)),
                },
                UnaryOperator::BitwiseNot => match expr_result {
                    ConstantExpressionType::Int(i) => Ok(ConstantExpressionType::Int(!i)),
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                UnaryOperator::LogicalNot => match expr_result {
                    ConstantExpressionType::Int(i) => {
                        Ok(ConstantExpressionType::Int(!(i > 0) as i128))
                    }
                    ConstantExpressionType::Float(f) => {
                        Ok(ConstantExpressionType::Int(!(f > 0.) as i128))
                    }
                },
            }
        }
        Expression::SizeOfExpr(_) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }
        Expression::SizeOfType(_) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }
        Expression::BinaryOp(op, left, right) => {
            let left_result = eval(left)?;
            let right_result = eval(right)?;
            match op {
                BinaryOperator::Mult => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l * r))
                    }
                    (ConstantExpressionType::Float(f), ConstantExpressionType::Int(i))
                    | (ConstantExpressionType::Int(i), ConstantExpressionType::Float(f)) => {
                        Ok(ConstantExpressionType::Float(f * i as f64))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Float(l * r))
                    }
                },
                BinaryOperator::Div => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l / r))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Float(l / r as f64))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Float(l as f64 / r))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Float(l / r))
                    }
                },
                BinaryOperator::Mod => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l % r))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::Add => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l + r))
                    }
                    (ConstantExpressionType::Float(f), ConstantExpressionType::Int(i))
                    | (ConstantExpressionType::Int(i), ConstantExpressionType::Float(f)) => {
                        Ok(ConstantExpressionType::Float(f + i as f64))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Float(l + r))
                    }
                },
                BinaryOperator::Sub => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l - r))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Float(l - r as f64))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Float(l as f64 - r))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Float(l - r))
                    }
                },
                BinaryOperator::LeftShift => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l << r))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::RightShift => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l >> r))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::LessThan => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l < r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l < r as f64) as i128))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int(((l as f64) < r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l < r) as i128))
                    }
                },
                BinaryOperator::GreaterThan => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l > r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l > r as f64) as i128))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l as f64 > r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l > r) as i128))
                    }
                },
                BinaryOperator::LessThanEq => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l <= r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l <= r as f64) as i128))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l as f64 <= r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l <= r) as i128))
                    }
                },
                BinaryOperator::GreaterThanEq => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l >= r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l >= r as f64) as i128))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l as f64 >= r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l >= r) as i128))
                    }
                },
                BinaryOperator::Equal => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l == r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l == r as f64) as i128))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l as f64 == r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l == r) as i128))
                    }
                },
                BinaryOperator::NotEqual => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l != r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int((l != r as f64) as i128))
                    }
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l as f64 != r) as i128))
                    }
                    (ConstantExpressionType::Float(l), ConstantExpressionType::Float(r)) => {
                        Ok(ConstantExpressionType::Int((l != r) as i128))
                    }
                },
                BinaryOperator::BitwiseAnd => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l & r))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::BitwiseOr => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l | r))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::BitwiseXor => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(l ^ r))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::LogicalAnd => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(((l != 0) && (r != 0)) as i128))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
                BinaryOperator::LogicalOr => match (left_result, right_result) {
                    (ConstantExpressionType::Int(l), ConstantExpressionType::Int(r)) => {
                        Ok(ConstantExpressionType::Int(((l != 0) || (r != 0)) as i128))
                    }
                    _ => Err(MiddleEndError::InvalidConstantExpression),
                },
            }
        }
        Expression::Ternary(cond, true_expr, false_expr) => {
            let cond_result = eval(cond)?;
            let cond_value = match cond_result {
                ConstantExpressionType::Int(i) => i != 0,
                ConstantExpressionType::Float(f) => f != 0.,
            };
            if cond_value {
                eval(true_expr)
            } else {
                eval(false_expr)
            }
        }
        Expression::Cast(_, _) => {
            // todo
            Err(MiddleEndError::CantEvaluateAtCompileTime)
        }

        Expression::ExpressionList(_, _)
        | Expression::FunctionCall(_, _)
        | Expression::PostfixIncrement(_)
        | Expression::PostfixDecrement(_)
        | Expression::PrefixIncrement(_)
        | Expression::PrefixDecrement(_)
        | Expression::Assignment(_, _) => Err(MiddleEndError::InvalidConstantExpression),
    }
}
