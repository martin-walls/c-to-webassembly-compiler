use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};

use log::{info, trace};

use crate::front_end::ast::Program;

use super::ast::AstNode;
use super::lexer::Lexer;

#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;

lalrpop_mod!(pub c_parser, "/front_end/c_parser.rs");

pub fn parse(source: String) -> Result<Program, ParseError> {
    info!("Running lexer and front_end");

    let lexer = Lexer::new(source.as_str());

    let ast = c_parser::ProgramParser::new().parse(lexer);

    match ast {
        Ok(ast) => {
            trace!("AST generated:\n{:#?}", ast);
            info!("Parser output:\n{}", ast.reconstruct_source());
            Ok(ast)
        }
        Err(e) => Err(ParseError(Box::new(e))),
    }
}

#[derive(Debug)]
pub struct ParseError(Box<dyn Error>);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error, due to the following error:\n{}", self.0)
    }
}

impl Error for ParseError {}
