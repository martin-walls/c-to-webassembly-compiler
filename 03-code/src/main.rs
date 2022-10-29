#![allow(dead_code)]

mod parser;

use std::fs;
use parser::parser::parse;

#[macro_use] extern crate lalrpop_util;

fn main() {
    let source = fs::read_to_string("test.c").unwrap();
    parse(source);
}

// #[test]
// fn parser() {
    
    
    

//     // assert!(parser::StringLiteralParser::new().parse("\"hello\"").unwrap() == "\"hello\"");
// }
