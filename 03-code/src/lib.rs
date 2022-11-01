#![allow(dead_code)]

mod preprocessor;
mod parser;

use parser::parser::parse;
use std::error::Error;
use clap::Parser as ClapParser;


#[macro_use] extern crate lalrpop_util;

#[derive(ClapParser, Debug)]
pub struct CliConfig {
    /// The path to the input file to compile
    filepath: String,
}

pub fn run(config: CliConfig) -> Result<(), Box<dyn Error>> {
  let source = preprocessor::preprocess(&config.filepath)?;
  parse(source);
  Ok(())
}

