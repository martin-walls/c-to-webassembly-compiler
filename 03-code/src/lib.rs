#![allow(dead_code)]

mod preprocessor;
mod parser;

use parser::parser::parse;

use std::error::Error;

#[macro_use] extern crate lalrpop_util;

pub struct Config {
  pub filepath: String,
}

impl Config {
  pub fn build(args: &[String]) -> Result<Config, &'static str> {
      if args.len() < 2 {
          // allow main() to handle error
          return Err("Not enough arguments");
      }
      let filepath = args[1].to_owned();
      Ok(Config { filepath })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  let source = preprocessor::preprocess(&config.filepath)?;
  parse(source);
  Ok(())
}

