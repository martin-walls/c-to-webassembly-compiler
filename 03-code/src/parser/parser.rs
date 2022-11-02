use log::{info,error, trace};
use super::ast::AstNode;

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn parse(source: String) {
    info!("Running parser");

    let result = c_parser::ProgramParser::new().parse(&source);

    match result {
        Ok(ast) => {
            trace!("AST generated:\n{:#?}", ast);
            info!("Parser output:\n{}", ast.reconstruct_source());
        },
        Err(e) => error!("Parser failed: {}", e),
    }
}
