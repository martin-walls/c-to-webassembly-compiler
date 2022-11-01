use log::info;

lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");

pub fn parse(source: String) {
  info!("Running parser");

  let result = c_parser::ProgramParser::new().parse(&source);

  info!("Parser output:\n{:#?}", result);
}