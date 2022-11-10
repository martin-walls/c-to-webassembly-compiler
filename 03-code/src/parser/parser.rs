#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;

use super::ast::AstNode;
use super::lexer::Lexer;
use log::{error, info, trace};

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn parse(source: String) {
    info!("Running lexer and parser");

    let lexer = Lexer::new(source.as_str());

    let result = c_parser::ProgramParser::new().parse(lexer);

    match result {
        Ok(ast) => {
            trace!("AST generated:\n{:#?}", ast);
            info!("Parser output:\n{}", ast.reconstruct_source());
        }
        Err(e) => error!("Parser failed: {:?}", e),
    }
}
