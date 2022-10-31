#![allow(dead_code)]


use std::env;
use std::process;
use c_to_wasm_compiler::Config;



fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Error parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = c_to_wasm_compiler::run(config) {
        println!("Program error: {e}");
        process::exit(2);
    }
}