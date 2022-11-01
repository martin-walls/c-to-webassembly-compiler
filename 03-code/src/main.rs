#![allow(dead_code)]


use std::process;
use c_to_wasm_compiler::CliConfig;
use clap::Parser;


fn main() {
    // parse cli args using clap
    let args = CliConfig::parse();

    // run program and handle error
    if let Err(e) = c_to_wasm_compiler::run(args) {
        println!("Program error: {e}");
        process::exit(1);
    }
}

