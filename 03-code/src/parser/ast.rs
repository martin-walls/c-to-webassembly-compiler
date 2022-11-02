use std::fmt::Write;

pub trait AstNode {
    fn reconstruct_source(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

impl AstNode for Identifier {
    fn reconstruct_source(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(u128),
    Float(f64),
}

impl AstNode for Constant {
    fn reconstruct_source(&self) -> String {
        match self {
            Constant::Int(i) => format!("{i}"),
            Constant::Float(f) => format!("{f}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Block(Vec<Box<Statement>>),
    Goto(Identifier),
    Continue,
    Break,
    Return(Option<Box<Expression>>),
    While(Box<Expression>, Box<Statement>),
    DoWhile(Box<Statement>, Box<Expression>),
    For(
        Option<Box<Expression>>,
        Option<Box<Expression>>,
        Option<Box<Expression>>,
        Box<Statement>,
    ),
    If(Box<Expression>, Box<Statement>),
    IfElse(Box<Expression>, Box<Statement>, Box<Statement>),
    Switch(Box<Expression>, Box<Statement>),
    Labelled(LabelledStatement),
    Expr(Box<Expression>),
    Declaration(Vec<SpecifierQualifier>, Vec<DeclaratorInitialiser>),
    EmptyDeclaration(Vec<SpecifierQualifier>),
    FunctionDeclaration(Vec<SpecifierQualifier>, Box<Declarator>, Box<Statement>),
}

impl AstNode for Statement {
    fn reconstruct_source(&self) -> String {
        match self {
            Statement::Block(stmts) => {
                let mut s = String::new();
                write!(&mut s, "{{\n").unwrap();
                for stmt in stmts {
                    write!(&mut s, "{}\n", stmt.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}\n").unwrap();
                s
            }
            Statement::Goto(i) => {
                format!("goto {};", i.reconstruct_source())
            },
            Statement::Continue => format!("continue;"),
            Statement::Break => format!("break;"),
            Statement::Return(e) => match e {
                Some(e) => format!("return {};", e.reconstruct_source()),
                None => format!("return;"),
            },
            Statement::While(e, s) => {
                format!("while ({}) {}", e.reconstruct_source(), s.reconstruct_source())
            },
            Statement::DoWhile(s, e) => {
                format!("do {} while ({})", s.reconstruct_source(), e.reconstruct_source())
            },
            Statement::For(e1, e2, e3, stmt) => {
                let mut s = String::new();
                write!(&mut s, "for (").unwrap();
                if let Some(e) = e1 {
                    write!(&mut s, "{}", e.reconstruct_source()).unwrap();
                }
                write!(&mut s, "; ").unwrap();
                if let Some(e) = e2 {
                    write!(&mut s, "{}", e.reconstruct_source()).unwrap();
                }
                write!(&mut s, "; ").unwrap();
                if let Some(e) = e3 {
                    write!(&mut s, "{}", e.reconstruct_source()).unwrap();
                }
                write!(&mut s, ") {}", stmt.reconstruct_source()).unwrap();
                s
            },
            Statement::If(e, s) => {
                format!("if ({}) {}", e.reconstruct_source(), s.reconstruct_source())
            },
            Statement::IfElse(e, s1, s2) => {
                format!("if ({}) {} else {}", e.reconstruct_source(), s1.reconstruct_source(), s2.reconstruct_source())
            },
            Statement::Switch(e, s) => {
                format!("switch ({}) {}", e.reconstruct_source(), s.reconstruct_source())
            },
            Statement::Labelled(s) => s.reconstruct_source(),
            Statement::Expr(e) => format!("{};", e.reconstruct_source()),
            Statement::Declaration(s, d) => {
                let mut st = String::new();
                for declarator in d {
                    for specifier in s {
                        write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                    }
                    write!(&mut st, "{};", declarator.reconstruct_source()).unwrap();
                }
                st
            },
            Statement::EmptyDeclaration(s) => {
                let mut st = String::new();
                for specifier in s {
                    write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                }
                write!(&mut st, ";").unwrap();
                st
            },
            Statement::FunctionDeclaration(s, d, b) => {
                let mut st = String::new();
                for specifier in s {
                    write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                }
                write!(&mut st, "{} {}", d.reconstruct_source(), b.reconstruct_source()).unwrap();
                st
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelledStatement {
    Case,
    Default,
    Named(Identifier),
}

impl AstNode for LabelledStatement {
    fn reconstruct_source(&self) -> String {
        match self {
            LabelledStatement::Case => "case :".to_owned(),
            LabelledStatement::Default => "default: ".to_owned(),
            LabelledStatement::Named(i) => format!("{}: ", i.reconstruct_source()),
        }
    }
}

#[derive(Debug)]
pub struct StatementList(pub Vec<Box<Statement>>);

impl AstNode for StatementList {
    fn reconstruct_source(&self) -> String {
        let mut s = String::new();
        for stmt in &self.0 {
            write!(&mut s, "{}\n", stmt.reconstruct_source()).unwrap();
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Constant(Constant),
    StringLiteral(String),
    Index(Box<Expression>, Box<Expression>),
    FunctionCall(Box<Expression>, Option<Vec<Box<Expression>>>),
    DirectMemberSelection(Box<Expression>, Identifier),
    IndirectMemberSelection(Box<Expression>, Identifier),
    PostfixIncrement(Box<Expression>),
    PostfixDecrement(Box<Expression>),
    PrefixIncrement(Box<Expression>),
    PrefixDecrement(Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
    SizeOfExpr(Box<Expression>),
    SizeOfType(TypeName),
    BinaryOp(BinaryOperator, Box<Expression>, Box<Expression>),
    Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
    Assignment(Box<Expression>, Box<Expression>, Option<BinaryOperator>),
    Cast(TypeName, Box<Expression>),
    ExpressionList(Box<Expression>, Box<Expression>),
}

impl AstNode for Expression {
    fn reconstruct_source(&self) -> String {
        match self {
            Expression::Identifier(i) => i.reconstruct_source(),
            Expression::Constant(c) => c.reconstruct_source(),
            Expression::StringLiteral(s) => format!("\"{}\"", s.to_owned()),
            Expression::Index(e1, e2) => format!("{}[{}]", e1.reconstruct_source(), e2.reconstruct_source()),
            Expression::FunctionCall(e1, e2) => {
                match e2 {
                    Some(e2) => {
                        let mut s = String::new();
                        write!(&mut s, "{}(", e1.reconstruct_source()).unwrap();
                        for e in &e2[..e2.len() - 1] {
                            write!(&mut s, "{}, ", e.reconstruct_source()).unwrap();
                        }
                        write!(&mut s, "{}", &e2[e2.len() - 1].reconstruct_source()).unwrap();
                        write!(&mut s, ")").unwrap();
                        s
                    },
                    None => format!("{}()", e1.reconstruct_source()),
                }
            },
            Expression::DirectMemberSelection(e, i) => format!("{}.{}", e.reconstruct_source(), i.reconstruct_source()),
            Expression::IndirectMemberSelection(e, i) => format!("{}->{}", e.reconstruct_source(), i.reconstruct_source()),
            Expression::PostfixIncrement(e) => format!("{}++", e.reconstruct_source()),
            Expression::PostfixDecrement(e) => format!("{}--", e.reconstruct_source()),
            Expression::PrefixIncrement(e) => format!("++{}", e.reconstruct_source()),
            Expression::PrefixDecrement(e) => format!("--{}", e.reconstruct_source()),
            Expression::UnaryOp(op, e) => format!("{}({})", op.reconstruct_source(), e.reconstruct_source()),
            Expression::SizeOfExpr(e) => format!("sizeof {}", e.reconstruct_source()),
            Expression::SizeOfType(t) => format!("sizeof ({})", t.reconstruct_source()),
            Expression::BinaryOp(op, l, r) => format!("({}) {} ({})", l.reconstruct_source(), op.reconstruct_source(), r.reconstruct_source()),
            Expression::Ternary(c, t, f) => format!("{} ? {} : {}", c.reconstruct_source(), t.reconstruct_source(), f.reconstruct_source()),
            Expression::Assignment(l, r, op) => match op {
                None => format!("{} = {}", l.reconstruct_source(), r.reconstruct_source()),
                Some(op) => format!("{} {}= {}", l.reconstruct_source(), op.reconstruct_source(), r.reconstruct_source()),
            },
            Expression::Cast(t, e) => format!("({}) {}", t.reconstruct_source(), e.reconstruct_source()),
            Expression::ExpressionList(e1, e2) => format!("{}, {}", e1.reconstruct_source(), e2.reconstruct_source()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    AddressOf,
    Dereference,
    Plus,
    Minus,
    BitwiseNot,
    LogicalNot,
}

impl AstNode for UnaryOperator {
    fn reconstruct_source(&self) -> String {
        match self {
            UnaryOperator::AddressOf => "&".to_owned(),
            UnaryOperator::Dereference => "*".to_owned(),
            UnaryOperator::Plus => "+".to_owned(),
            UnaryOperator::Minus => "-".to_owned(),
            UnaryOperator::BitwiseNot => "~".to_owned(),
            UnaryOperator::LogicalNot => "!".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Mult,
    Div,
    Mod,
    Add,
    Sub,
    LeftShift,
    RightShift,
    LessThan,
    GreaterThan,
    LessThanEq,
    GreaterThanEq,
    Equal,
    NotEqual,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LogicalAnd,
    LogicalOr,
}

impl AstNode for BinaryOperator {
    fn reconstruct_source(&self) -> String {
        match self {
            BinaryOperator::Mult => "*".to_owned(),
            BinaryOperator::Div => "/".to_owned(),
            BinaryOperator::Mod => "%".to_owned(),
            BinaryOperator::Add => "+".to_owned(),
            BinaryOperator::Sub => "-".to_owned(),
            BinaryOperator::LeftShift => "<<".to_owned(),
            BinaryOperator::RightShift => ">>".to_owned(),
            BinaryOperator::LessThan => "<".to_owned(),
            BinaryOperator::GreaterThan => ">".to_owned(),
            BinaryOperator::LessThanEq => "<=".to_owned(),
            BinaryOperator::GreaterThanEq => ">=".to_owned(),
            BinaryOperator::Equal => "==".to_owned(),
            BinaryOperator::NotEqual => "!=".to_owned(),
            BinaryOperator::BitwiseAnd => "&".to_owned(),
            BinaryOperator::BitwiseOr => "|".to_owned(),
            BinaryOperator::BitwiseXor => "^".to_owned(),
            BinaryOperator::LogicalAnd => "&&".to_owned(),
            BinaryOperator::LogicalOr => "||".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecifierQualifier {
    TypeSpecifier(TypeSpecifier),
    StorageClassSpecifier(StorageClassSpecifier),
    TypeQualifier(TypeQualifier),
}

impl AstNode for SpecifierQualifier {
    fn reconstruct_source(&self) -> String {
        match self {
            SpecifierQualifier::TypeSpecifier(t) => t.reconstruct_source(),
            SpecifierQualifier::StorageClassSpecifier(s) => s.reconstruct_source(),
            SpecifierQualifier::TypeQualifier(q) => q.reconstruct_source(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecifier {
    ArithmeticType(ArithmeticTypeSpecifier),
    Void,
    Struct(StructType),
    Union(UnionType),
    Enum(EnumType),
}

impl AstNode for TypeSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            TypeSpecifier::ArithmeticType(t) => t.reconstruct_source(),
            TypeSpecifier::Void => "void".to_owned(),
            TypeSpecifier::Struct(t) => t.reconstruct_source(),
            TypeSpecifier::Union(t) => t.reconstruct_source(),
            TypeSpecifier::Enum(t) => t.reconstruct_source(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticTypeSpecifier {
    Char,
    Short,
    Int,
    Long,
    Signed,
    Unsigned,
    Float,
    Double,
}

impl AstNode for ArithmeticTypeSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            ArithmeticTypeSpecifier::Char => "char".to_owned(),
            ArithmeticTypeSpecifier::Short => "short".to_owned(),
            ArithmeticTypeSpecifier::Int => "int".to_owned(),
            ArithmeticTypeSpecifier::Long => "long".to_owned(),
            ArithmeticTypeSpecifier::Signed => "signed".to_owned(),
            ArithmeticTypeSpecifier::Unsigned => "unsigned".to_owned(),
            ArithmeticTypeSpecifier::Float => "float".to_owned(),
            ArithmeticTypeSpecifier::Double => "double".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticType {
    I8,  // signed char
    U8,  // unsigned char
    I16, // signed short
    U16, // unsigned short
    I32, // signed int
    U32, // unsigned int
    I64, // signed long
    U64, // unsigned long
    F32, // float
    F64, // double
}

impl AstNode for ArithmeticType {
    fn reconstruct_source(&self) -> String {
        match self {
            ArithmeticType::I8 => "signed char".to_owned(),
            ArithmeticType::U8 => "unsigned char".to_owned(),
            ArithmeticType::I16 => "short".to_owned(),
            ArithmeticType::U16 => "unsigned short".to_owned(),
            ArithmeticType::I32 => "int".to_owned(),
            ArithmeticType::U32 => "unsigned int".to_owned(),
            ArithmeticType::I64 => "long".to_owned(),
            ArithmeticType::U64 => "unsigned long".to_owned(),
            ArithmeticType::F32 => "float".to_owned(),
            ArithmeticType::F64 => "double".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<StructMemberDeclaration>),
}

impl AstNode for StructType {
    fn reconstruct_source(&self) -> String {
        match self {
            StructType::Declaration(i) => format!("struct {}", i.reconstruct_source()),
            StructType::Definition(i, ms) => {
                let mut s = String::new();
                match i {
                    Some(i) => write!(&mut s, "struct {} {{\n", i.reconstruct_source()).unwrap(),
                    None => write!(&mut s, "struct {{\n").unwrap()
                }
                for member in ms {
                    write!(&mut s, "{}", member.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}\n").unwrap();
                s
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructMemberDeclaration(pub Vec<SpecifierQualifier>, pub Vec<Box<Declarator>>);

impl AstNode for StructMemberDeclaration {
    fn reconstruct_source(&self) -> String {
        let mut s = String::new();
        for declarator in &self.1 {
            for specifier in &self.0 {
                write!(&mut s, "{} ", specifier.reconstruct_source()).unwrap();
            }
            write!(&mut s, "{};\n", declarator.reconstruct_source()).unwrap();
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnionType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<StructMemberDeclaration>),
}

impl AstNode for UnionType {
    fn reconstruct_source(&self) -> String {
        match self {
            UnionType::Declaration(i) => format!("union {}", i.reconstruct_source()),
            UnionType::Definition(i, ms) => {
                let mut s = String::new();
                match i {
                    Some(i) => write!(&mut s, "union {} {{\n", i.reconstruct_source()).unwrap(),
                    None => write!(&mut s, "union {{\n").unwrap()
                }
                for member in ms {
                    write!(&mut s, "{}", member.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}\n").unwrap();
                s
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<Enumerator>),
}

impl AstNode for EnumType {
    fn reconstruct_source(&self) -> String {
        match self {
            EnumType::Declaration(i) => format!("enum {}", i.reconstruct_source()),
            EnumType::Definition(i, es) => {
                let mut s = String::new();
                match i {
                    Some(i) => write!(&mut s, "enum {} {{\n", i.reconstruct_source()).unwrap(),
                    None => write!(&mut s, "enum {{\n").unwrap()
                }
                for enumerator in es {
                    write!(&mut s, "{}\n", enumerator.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}").unwrap();
                s
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Enumerator {
    Simple(Identifier),
    WithValue(Identifier, Box<Expression>),
}

impl AstNode for Enumerator {
    fn reconstruct_source(&self) -> String {
        match self {
            Enumerator::Simple(i) => format!("{}, ", i.reconstruct_source()),
            Enumerator::WithValue(i, e) => format!("{} = {}, ", i.reconstruct_source(), e.reconstruct_source()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClassSpecifier {
    Auto,
    Extern,
    Register,
    Static,
    Typedef,
}

impl AstNode for StorageClassSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            StorageClassSpecifier::Auto => "auto".to_owned(),
            StorageClassSpecifier::Extern => "extern".to_owned(),
            StorageClassSpecifier::Register => "register".to_owned(),
            StorageClassSpecifier::Static => "static".to_owned(),
            StorageClassSpecifier::Typedef => "typedef".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
}

impl AstNode for TypeQualifier {
    fn reconstruct_source(&self) -> String {
        match self {
            TypeQualifier::Const => "const".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declarator {
    Identifier(Identifier),
    PointerDeclarator(Box<Declarator>),
    AbstractPointerDeclarator,
    ArrayDeclarator(Box<Declarator>, Option<Box<Expression>>),
    FunctionDeclarator(Box<Declarator>, Option<ParameterTypeList>),
}

impl AstNode for Declarator {
    fn reconstruct_source(&self) -> String {
        match self {
            Declarator::Identifier(i) => i.reconstruct_source(),
            Declarator::PointerDeclarator(d) => format!("*({})", d.reconstruct_source()),
            Declarator::AbstractPointerDeclarator => "*".to_owned(),
            Declarator::ArrayDeclarator(d, e) => match e {
                Some(e) => format!("{}[{}]", d.reconstruct_source(), e.reconstruct_source()),
                None => format!("{}[]", d.reconstruct_source()),
            },
            Declarator::FunctionDeclarator(d, ps) => match ps {
                Some(ps) => format!("{}({})", d.reconstruct_source(), ps.reconstruct_source()),
                None => format!("{}()", d.reconstruct_source()),
            },
        }
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum AbstractDeclarator {
//   PointerDeclarator,
//   ArrayDeclarator(Box<AbstractDeclarator>, Option<Box<Expression>>),
//   FunctionDeclarator(Box<AbstractDeclarator>, Option<ParameterTypeList>),
// }

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterTypeList {
    Normal(Vec<ParameterDeclaration>),
    Variadic(Vec<ParameterDeclaration>),
}

impl AstNode for ParameterTypeList {
    fn reconstruct_source(&self) -> String {
        match self {
            ParameterTypeList::Normal(ps) => {
                let mut s = String::new();
                for parameter in &ps[..ps.len() - 1] {
                    write!(&mut s, "{}, ", parameter.reconstruct_source()).unwrap();
                }
                write!(&mut s, "{}", &ps[ps.len() - 1].reconstruct_source()).unwrap();
                s
            },
            ParameterTypeList::Variadic(ps) => {
                let mut s = String::new();
                for parameter in &ps[..ps.len() - 1] {
                    write!(&mut s, "{}, ", parameter.reconstruct_source()).unwrap();
                }
                write!(&mut s, "{}", &ps[ps.len() - 1].reconstruct_source()).unwrap();
                write!(&mut s, "...").unwrap();
                s
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterDeclaration {
    Named(Vec<SpecifierQualifier>, Box<Declarator>),
    // Abstract(Vec<SpecifierQualifier>, Option<Box<AbstractDeclarator>>),
}

impl AstNode for ParameterDeclaration {
    fn reconstruct_source(&self) -> String {
        match self {
            ParameterDeclaration::Named(s, d) => {
                let mut st = String::new();
                for specifier in s {
                    write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                }
                write!(&mut st, "{}", d.reconstruct_source()).unwrap();
                st
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclaratorInitialiser {
    NoInit(Box<Declarator>),
    Init(Box<Declarator>, Box<Expression>),
    Function(Box<Declarator>, Box<Statement>),
    StructOrUnion(Box<Declarator>, Vec<Box<Expression>>),
}

impl AstNode for DeclaratorInitialiser {
    fn reconstruct_source(&self) -> String {
        match self {
            DeclaratorInitialiser::NoInit(d) => d.reconstruct_source(),
            DeclaratorInitialiser::Init(d, e) => format!("{} = {}", d.reconstruct_source(), e.reconstruct_source()),
            DeclaratorInitialiser::Function(d, s) => format!("{} {}", d.reconstruct_source(), s.reconstruct_source()),
            DeclaratorInitialiser::StructOrUnion(d, es) => {
                let mut s = String::new();
                write!(&mut s, "{} = {{\n", d.reconstruct_source()).unwrap();
                for e in es {
                    write!(&mut s, "{},\n", e.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}").unwrap();
                s
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName(pub Vec<SpecifierQualifier>, pub Option<Box<Declarator>>);

impl AstNode for TypeName {
    fn reconstruct_source(&self) -> String {
        let mut s = String::new();
        for specifier in &self.0 {
            write!(&mut s, "{} ", specifier.reconstruct_source()).unwrap();
        }
        match &self.1 {
            Some(d) => write!(&mut s, "{};\n", d.reconstruct_source()).unwrap(),
            None => (),
        }        
        s
    }
}
