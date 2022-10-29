lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn parse(source: String) {
  println!("{source}");

  let result = c_parser::StatementParser::new().parse(&source);
  println!("{:?}", result);
}