#![allow(dead_code)]

mod parser;
mod preprocessor;

use clap::Parser as ClapParser;
use parser::parser::parse;
use preprocessor::preprocess;
use std::error::Error;

#[macro_use]
extern crate lalrpop_util;

#[derive(ClapParser, Debug)]
pub struct CliConfig {
    /// The path to the input file to compile
    filepath: String,
}

pub fn run(config: CliConfig) -> Result<(), Box<dyn Error>> {
    let source = preprocess(&config.filepath)?;
    parse(source);
    Ok(())
}
