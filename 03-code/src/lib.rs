#![allow(dead_code)]

mod backend;
mod fmt_indented;
mod middle_end;
mod parser;
mod preprocessor;
mod relooper;

use crate::backend::target_code_generation::generate_target_code;
use crate::relooper::relooper::reloop;
use clap::Parser as ClapParser;
use log::info;
use middle_end::ast_to_ir::convert_to_ir;
use parser::parser::parse;
use preprocessor::preprocess;
use std::error::Error;
use std::path::Path;

#[macro_use]
extern crate lalrpop_util;

#[derive(ClapParser, Debug)]
pub struct CliConfig {
    /// The path to the input file to compile
    filepath: String,
}

pub fn run(config: CliConfig) -> Result<(), Box<dyn Error>> {
    let source = preprocess(Path::new(&config.filepath))?;
    let ast = parse(source)?;
    let ir = convert_to_ir(ast)?;
    info!("IR: {}", ir);
    let relooped_ir = reloop(ir);
    generate_target_code(relooped_ir);
    Ok(())
}
