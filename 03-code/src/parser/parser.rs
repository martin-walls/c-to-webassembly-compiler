#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;

use super::ast::AstNode;
use super::preparser::process_typedefs as preparse_typedefs;
use log::{error, info, trace};

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn parse(source: String) {
    info!("Running parser");

    let typedef_names = preparse_typedefs(&source);

    info!("Typedef names: {:?}", typedef_names);

    let result = c_parser::ProgramParser::new().parse(&source);

    match result {
        Ok(ast) => {
            trace!("AST generated:\n{:#?}", ast);
            info!("Parser output:\n{}", ast.reconstruct_source());
        }
        Err(e) => error!("Parser failed: {}", e),
    }
}
