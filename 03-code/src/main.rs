#![allow(dead_code)]

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use c_to_wasm_compiler::CliConfig;
use clap::Parser;
use std::process;

fn main() {
    pretty_env_logger::init();

    // parse cli args using clap
    let args = CliConfig::parse();
    debug!("Cli arguments: {:?}", args);

    // run program and handle error
    if let Err(e) = c_to_wasm_compiler::run(args) {
        error!("Program error: {e}");
        process::exit(1);
    }
}
