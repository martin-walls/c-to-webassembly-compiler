#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;

use super::ast::AstNode;
use super::lexer::Lexer;
use crate::parser::ast::Program;
use log::{info, trace};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn parse(source: String) -> Result<Program, ParseError> {
    info!("Running lexer and parser");

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
