use super::ast;
use super::lexer::Lexer;
use log::trace;
use regex::Regex;

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

struct TypedefDeclaration {
    name: String,
    identifier_position: u32,
}

pub fn process_typedefs(source: &String) -> Vec<TypedefDeclaration> {
    let mut typedef_identifiers: Vec<String> = Vec::new();

    let re = Regex::new(r"(typedef [^;]*;)").unwrap();

    for capture in re.captures_iter(&source) {
        trace!("Found typedef: {}", &capture[1]);
        let empty_typedef_names: Vec<String> = vec![];
        let lexer = Lexer::new(&capture[1], &empty_typedef_names);
        let result = c_parser::DeclarationParser::new().parse(lexer);
        println!("{:?}", result);
        if let Ok(stmt) = result {
            if let ast::Statement::Declaration(_, ds) = *stmt {
                if ds.len() == 1 {
                    if let Some(name) = ds[0].get_identifier_name().to_owned() {
                        trace!("Typedef identifier: {:?}", name);
                        typedef_identifiers.push(TypedefDeclaration { name });
                    }
                }
            }
        }
    }

    typedef_identifiers
}
