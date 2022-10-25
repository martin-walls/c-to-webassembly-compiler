pub mod ast;

#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub c);

fn main() {
    println!("Hello, world!");
}

#[test]
fn parser() {
    assert!(c::KeywordParser::new().parse("auto").is_ok());
    assert!(c::KeywordParser::new().parse("unsigned").is_ok());
    assert!(c::IdentifierParser::new().parse("foo").is_ok());
    assert!(c::IdentifierParser::new().parse("foo_bar").is_ok());
    assert!(c::IdentifierParser::new().parse("FOO_BAR").is_ok());
    assert!(c::IdentifierParser::new().parse("FooBar123").is_ok());
    assert!(c::IdentifierParser::new().parse("Foo__Bar_123").is_ok());
    assert!(c::IntegerConstantParser::new().parse("0b1100").unwrap() == 12);
    assert!(c::IntegerConstantParser::new().parse("012").unwrap() == 10);
    assert!(c::IntegerConstantParser::new().parse("0xF3").unwrap() == 243);
    assert!(c::IntegerConstantParser::new().parse("0x03a").unwrap() == 58);
    assert!(c::IntegerConstantParser::new().parse("123").unwrap() == 123);
    assert!(c::DecimalFloatingConstantParser::new().parse("12.3").unwrap() == 12.3);
    assert!(c::DecimalFloatingConstantParser::new().parse("1.2e-2").unwrap() == 0.012);
    assert!(matches!(c::ConstantParser::new().parse("5E3").unwrap(), ast::Constant::FloatingConstant(_)));

    assert!(c::StringLiteralParser::new().parse("\"hello\"").unwrap() == "\"hello\"");
}