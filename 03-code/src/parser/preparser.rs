use regex::Regex;
use log::trace;
use super::ast;

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn process_typedefs(source: &String) -> Vec<String> {
    let mut typedef_identifiers: Vec<String> = Vec::new();

    let re = Regex::new(r"(typedef [^;]*;)").unwrap();

    for capture in re.captures_iter(&source) {
        trace!("Found typedef: {}", &capture[1]);
        let result = c_parser::DeclarationParser::new().parse(&capture[1]);
        if let Ok(stmt) = result {
            if let ast::Statement::Declaration(_, ds) = *stmt {
                if ds.len() == 1 {
                    if let Some(name) = ds[0].get_identifier_name().to_owned() {
                        trace!("Typedef identifier: {:?}", name);
                        typedef_identifiers.push(name);
                    }
                }
            }
        }
    }

    typedef_identifiers
}

