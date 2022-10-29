#[cfg(test)]
mod parser_tests {
    lalrpop_mod!(pub c_parser, "/parser/c_parser.rs");
    use super::super::ast::*;

    #[test]
    fn identifier_parser() {
        assert!(c_parser::IdentifierParser::new().parse("foo").is_ok());
        assert!(c_parser::IdentifierParser::new().parse("foo_bar").is_ok());
        assert!(c_parser::IdentifierParser::new().parse("FOO_BAR").is_ok());
        assert!(c_parser::IdentifierParser::new().parse("FooBar123").is_ok());
        assert!(c_parser::IdentifierParser::new().parse("Foo__Bar_123").is_ok());
        assert!(c_parser::IdentifierParser::new().parse("_foo").is_ok());
        assert!(c_parser::IdentifierParser::new().parse("12FooBar").is_err());
    }

    #[test]
    fn constant_parser() {
        assert!(c_parser::ConstantParser::new().parse("0b1100").unwrap() == Constant::Int(12));
        assert!(c_parser::ConstantParser::new().parse("012").unwrap() == Constant::Int(10));
        assert!(c_parser::ConstantParser::new().parse("0xF3").unwrap() == Constant::Int(243));
        assert!(c_parser::ConstantParser::new().parse("0x03a").unwrap() == Constant::Int(58));
        assert!(c_parser::ConstantParser::new().parse("123").unwrap() == Constant::Int(123));
        assert!(c_parser::ConstantParser::new().parse("0").unwrap() == Constant::Int(0));

        assert!(c_parser::ConstantParser::new().parse("12.3").unwrap() == Constant::Float(12.3));
        assert!(c_parser::ConstantParser::new().parse("1.2e-2").unwrap() == Constant::Float(0.012));
        assert!(c_parser::ConstantParser::new().parse("5E3").unwrap() == Constant::Float(5000.));
    }

    #[test]
    fn string_literal_parser() {
        assert!(*c_parser::ExpressionParser::new().parse("\"hello world\"").unwrap() == Expression::StringLiteral("hello world".to_owned()));
    }

    #[test]
    fn expression_parser() {
        let valid_exprs = [
            "abc123",
            "\"foo bar\"",
            "12.34e2",
            "foo++",
            "s.member--",
            "arr[0]",
            "sizeof(int)",
            "sizeof arr[0]",
            "*ptr",
            "!(++foo)",
            "(unsigned long int) (23 + 5)",
            "(1 * (2 + 3)) % (6 / 2)",
            "(foo >> 3) & 1",
            "foo >= 2 * 3",
            "3 % 2 != foo_bar1 + 2 || x == 5",
            "foo ? 3 : 1 + 2",
            "x = 123",
            "foo += 7*2",
            "bar |= 0b1001",
        ];

        for expr in valid_exprs {
            println!("{expr}");
            println!("{:?}", c_parser::ExpressionParser::new().parse(expr));
            assert!(c_parser::ExpressionParser::new().parse(expr).is_ok());
        }
    }

    #[test]
    fn statement_parser() {
        let valid_stmts = [
            "while (a <= 12)
                a++;",
            "do {
                foo++;
                bar--;
                continue;
            } while (bar > 0);",
            "if (1) {
                y++;
                a += 3.7;
            } else {
                break;
            }",
            "if (x > 1) x = 1;",
            "goto label;",
            "for (i = 0; i < 5; i++) {}",
            "for(;;) {
                x++;
            }",
        ];

        for stmt in valid_stmts {
            println!("{stmt}");
            assert!(c_parser::StatementParser::new().parse(stmt).is_ok());
        }
    }

    #[test]
    fn type_specifier_parser() {
        assert!(c_parser::TypeSpecifierParser::new().parse("void").unwrap() == TypeSpecifier::Void);
        assert!(c_parser::TypeSpecifierParser::new().parse("unsigned short int").unwrap() == TypeSpecifier::ArithmeticType(ArithmeticType::U16));
        assert!(c_parser::TypeSpecifierParser::new().parse("long int").unwrap() == TypeSpecifier::ArithmeticType(ArithmeticType::I64));
        assert!(c_parser::TypeSpecifierParser::new().parse("char").unwrap() == TypeSpecifier::ArithmeticType(ArithmeticType::I8));
        assert!(c_parser::TypeSpecifierParser::new().parse("unsigned").unwrap() == TypeSpecifier::ArithmeticType(ArithmeticType::U32));
        assert!(c_parser::TypeSpecifierParser::new().parse("signed int").unwrap() == TypeSpecifier::ArithmeticType(ArithmeticType::I32));
    }

    #[test]
    fn struct_parser() {
        let valid = [
            "struct s",
            "struct s {int n; double d;}",
            "struct {int a; int b;}",
            "struct s {int n; union u {char c; short i;};}",
        ];

        for s in valid {
            println!("{s}");
            assert!(c_parser::TypeSpecifierParser::new().parse(s).is_ok());
        }
    }

    #[test]
    fn union_parser() {
        let valid = [
            "union s",
            "union s {int n; double d;}",
            "union {char c; int n;}",
        ];

        for s in valid {
            println!("{s}");
            assert!(c_parser::TypeSpecifierParser::new().parse(s).is_ok());
        }
    }

    #[test]
    fn enum_parser() {
        let valid = [
            "enum Color {Red, Green, Blue}",
            "enum COLOR {Red}",
            "enum {R, G, B,}"
        ];

        for s in valid {
            println!("{s}");
            assert!(c_parser::TypeSpecifierParser::new().parse(s).is_ok());
        }
    }
}