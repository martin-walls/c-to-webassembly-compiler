use std::error::Error;
use std::fmt;
use std::fmt::{Formatter, Write};

pub trait AstNode {
    fn reconstruct_source(&self) -> String;
}

#[derive(Debug)]
pub enum AstError {
    InvalidTypeDeclaration(&'static str),
    TooManyStorageClassSpecifiers(StorageClassSpecifier),
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AstError::InvalidTypeDeclaration(msg) => write!(f, "{msg}"),
            AstError::TooManyStorageClassSpecifiers(_) => {
                write!(f, "Too many storage class specifiers")
            }
        }
    }
}

impl Error for AstError {}

pub type Program = StatementList;

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
    Char(char),
}

impl AstNode for Constant {
    fn reconstruct_source(&self) -> String {
        match self {
            Constant::Int(i) => format!("{i}"),
            Constant::Float(f) => format!("{f}"),
            Constant::Char(c) => format!("'{c}'"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Block(Vec<Statement>),
    Goto(Identifier),
    Continue,
    Break,
    Return(Option<Expression>),
    While(Expression, Box<Statement>),
    DoWhile(Box<Statement>, Expression),
    For(
        Option<ExpressionOrDeclaration>,
        Option<Expression>,
        Option<Expression>,
        Box<Statement>,
    ),
    If(Expression, Box<Statement>),
    IfElse(Expression, Box<Statement>, Box<Statement>),
    Switch(Expression, Box<Statement>),
    Labelled(LabelledStatement),
    Expr(Expression),
    Declaration(SpecifierQualifier, Vec<DeclaratorInitialiser>),
    EmptyDeclaration(SpecifierQualifier),
    FunctionDeclaration(SpecifierQualifier, Declarator, Box<Statement>),
    Empty,
}

impl AstNode for Statement {
    fn reconstruct_source(&self) -> String {
        match self {
            Statement::Block(stmts) => {
                let mut s = String::new();
                writeln!(&mut s, "{{").unwrap();
                for stmt in stmts {
                    writeln!(&mut s, "{}", stmt.reconstruct_source()).unwrap();
                }
                writeln!(&mut s, "}}").unwrap();
                s
            }
            Statement::Goto(i) => {
                format!("goto {};", i.reconstruct_source())
            }
            Statement::Continue => "continue;".to_owned(),
            Statement::Break => "break;".to_owned(),
            Statement::Return(e) => match e {
                Some(e) => format!("return {};", e.reconstruct_source()),
                None => "return;".to_owned(),
            },
            Statement::While(e, s) => {
                format!(
                    "while ({}) {}",
                    e.reconstruct_source(),
                    s.reconstruct_source()
                )
            }
            Statement::DoWhile(s, e) => {
                format!(
                    "do {} while ({})",
                    s.reconstruct_source(),
                    e.reconstruct_source()
                )
            }
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
            }
            Statement::If(e, s) => {
                format!("if ({}) {}", e.reconstruct_source(), s.reconstruct_source())
            }
            Statement::IfElse(e, s1, s2) => {
                format!(
                    "if ({}) {} else {}",
                    e.reconstruct_source(),
                    s1.reconstruct_source(),
                    s2.reconstruct_source()
                )
            }
            Statement::Switch(e, s) => {
                format!(
                    "switch ({}) {}",
                    e.reconstruct_source(),
                    s.reconstruct_source()
                )
            }
            Statement::Labelled(s) => s.reconstruct_source(),
            Statement::Expr(e) => format!("{};", e.reconstruct_source()),
            Statement::Declaration(s, d) => {
                let mut st = String::new();
                for declarator in d {
                    write!(&mut st, "{} ", s.reconstruct_source()).unwrap();
                    write!(&mut st, "{};", declarator.reconstruct_source()).unwrap();
                }
                st
            }
            Statement::EmptyDeclaration(s) => {
                format!("{};", s.reconstruct_source())
            }
            Statement::FunctionDeclaration(s, d, b) => {
                format!(
                    "{} {} {}",
                    s.reconstruct_source(),
                    d.reconstruct_source(),
                    b.reconstruct_source(),
                )
            }
            Statement::Empty => "".to_owned(),
        }
    }
}

// for 'for' statements, where the first expression canc be either an expression
// or a declaration
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionOrDeclaration {
    Expression(Expression),
    Declaration(Box<Statement>), // the Statement should be of value statement::Declarator
}

impl AstNode for ExpressionOrDeclaration {
    fn reconstruct_source(&self) -> String {
        match self {
            ExpressionOrDeclaration::Expression(e) => e.reconstruct_source(),
            ExpressionOrDeclaration::Declaration(d) => d.reconstruct_source(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelledStatement {
    Case(Expression, Box<Statement>),
    Default(Box<Statement>),
    Named(Identifier, Box<Statement>),
}

impl AstNode for LabelledStatement {
    fn reconstruct_source(&self) -> String {
        match self {
            LabelledStatement::Case(e, s) => format!(
                "case {}: {}",
                e.reconstruct_source(),
                s.reconstruct_source()
            ),
            LabelledStatement::Default(s) => format!("default: {}", s.reconstruct_source()),
            LabelledStatement::Named(i, s) => {
                format!("{}: {}", i.reconstruct_source(), s.reconstruct_source())
            }
        }
    }
}

#[derive(Debug)]
pub struct StatementList(pub Vec<Statement>);

impl AstNode for StatementList {
    fn reconstruct_source(&self) -> String {
        let mut s = String::new();
        for stmt in &self.0 {
            writeln!(&mut s, "{}", stmt.reconstruct_source()).unwrap();
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
    FunctionCall(Box<Expression>, Vec<Expression>),
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
    Assignment(Box<Expression>, Box<Expression>),
    Cast(TypeName, Box<Expression>),
    ExpressionList(Box<Expression>, Box<Expression>),
}

impl AstNode for Expression {
    fn reconstruct_source(&self) -> String {
        match self {
            Expression::Identifier(i) => i.reconstruct_source(),
            Expression::Constant(c) => c.reconstruct_source(),
            Expression::StringLiteral(s) => format!("\"{}\"", s.to_owned()),
            Expression::Index(e1, e2) => {
                format!("{}[{}]", e1.reconstruct_source(), e2.reconstruct_source())
            }
            Expression::FunctionCall(e1, e2) => {
                let mut s = String::new();
                write!(&mut s, "{}(", e1.reconstruct_source()).unwrap();
                if !e2.is_empty() {
                    for e in &e2[..e2.len() - 1] {
                        write!(&mut s, "{}, ", e.reconstruct_source()).unwrap();
                    }
                    write!(&mut s, "{}", &e2[e2.len() - 1].reconstruct_source()).unwrap();
                }
                write!(&mut s, ")").unwrap();
                s
            }
            Expression::DirectMemberSelection(e, i) => {
                format!("{}.{}", e.reconstruct_source(), i.reconstruct_source())
            }
            Expression::IndirectMemberSelection(e, i) => {
                format!("{}->{}", e.reconstruct_source(), i.reconstruct_source())
            }
            Expression::PostfixIncrement(e) => format!("{}++", e.reconstruct_source()),
            Expression::PostfixDecrement(e) => format!("{}--", e.reconstruct_source()),
            Expression::PrefixIncrement(e) => format!("++{}", e.reconstruct_source()),
            Expression::PrefixDecrement(e) => format!("--{}", e.reconstruct_source()),
            Expression::UnaryOp(op, e) => {
                format!("{}({})", op.reconstruct_source(), e.reconstruct_source())
            }
            Expression::SizeOfExpr(e) => format!("sizeof {}", e.reconstruct_source()),
            Expression::SizeOfType(t) => format!("sizeof ({})", t.reconstruct_source()),
            Expression::BinaryOp(op, l, r) => format!(
                "({}) {} ({})",
                l.reconstruct_source(),
                op.reconstruct_source(),
                r.reconstruct_source()
            ),
            Expression::Ternary(c, t, f) => format!(
                "{} ? {} : {}",
                c.reconstruct_source(),
                t.reconstruct_source(),
                f.reconstruct_source()
            ),
            Expression::Assignment(l, r) => {
                format!("{} = {}", l.reconstruct_source(), r.reconstruct_source())
            }
            Expression::Cast(t, e) => {
                format!("({}) {}", t.reconstruct_source(), e.reconstruct_source())
            }
            Expression::ExpressionList(e1, e2) => {
                format!("{}, {}", e1.reconstruct_source(), e2.reconstruct_source())
            }
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
pub struct SpecifierQualifier {
    pub type_specifier: TypeSpecifier,
    pub storage_class_specifier: Option<StorageClassSpecifier>,
    pub const_: bool,
    pub inline: bool,
}

impl SpecifierQualifier {
    pub fn create(sqs: Vec<SpecifierQualifierToken>) -> Result<Self, AstError> {
        let mut type_specifiers: Vec<TypeSpecifierToken> = Vec::new();
        let mut storage_class_specifier = None;
        let mut const_ = false;
        let mut inline = false;

        for sq in sqs {
            match sq {
                SpecifierQualifierToken::TypeSpecifier(t) => type_specifiers.push(t),
                SpecifierQualifierToken::StorageClassSpecifier(s) => {
                    if storage_class_specifier.is_none() {
                        storage_class_specifier = Some(s);
                    } else {
                        return Err(AstError::TooManyStorageClassSpecifiers(s));
                    }
                }
                SpecifierQualifierToken::TypeQualifier(q) => match q {
                    TypeQualifier::Const => const_ = true,
                },
                SpecifierQualifierToken::FunctionSpecifier(f) => match f {
                    FunctionSpecifier::Inline => inline = true,
                },
            }
        }

        let type_specifier = TypeSpecifier::create(type_specifiers)?;

        Ok(SpecifierQualifier {
            type_specifier,
            storage_class_specifier,
            const_,
            inline,
        })
    }
}

impl AstNode for SpecifierQualifier {
    fn reconstruct_source(&self) -> String {
        let mut st = String::new();
        match &self.storage_class_specifier {
            None => {}
            Some(s) => write!(&mut st, "{} ", s.reconstruct_source()).unwrap(),
        }
        if self.const_ {
            write!(&mut st, "const ").unwrap()
        }
        if self.inline {
            write!(&mut st, "inline ").unwrap()
        }
        write!(&mut st, "{}", self.type_specifier.reconstruct_source()).unwrap();
        st
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecifier {
    ArithmeticType(ArithmeticType),
    Void,
    Struct(StructType),
    Union(UnionType),
    Enum(EnumType),
    CustomType(Identifier),
}

impl TypeSpecifier {
    fn create(types: Vec<TypeSpecifierToken>) -> Result<Self, AstError> {
        if types.is_empty() {
            return Err(AstError::InvalidTypeDeclaration("No type specified"));
        }
        match &types[0] {
            TypeSpecifierToken::ArithmeticType(_) => {
                let arithmetic_type = ArithmeticType::create_from_type_specifiers(types);
                match arithmetic_type {
                    Ok(t) => Ok(TypeSpecifier::ArithmeticType(t)),
                    Err(e) => Err(e),
                }
            }
            TypeSpecifierToken::Void => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(TypeSpecifier::Void)
            }
            TypeSpecifierToken::Struct(t) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(TypeSpecifier::Struct(t.to_owned()))
            }
            TypeSpecifierToken::Union(t) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(TypeSpecifier::Union(t.to_owned()))
            }
            TypeSpecifierToken::Enum(t) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(TypeSpecifier::Enum(t.to_owned()))
            }
            TypeSpecifierToken::CustomType(i) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(TypeSpecifier::CustomType(i.to_owned()))
            }
        }
    }
}

impl AstNode for TypeSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            TypeSpecifier::ArithmeticType(t) => t.reconstruct_source(),
            TypeSpecifier::Void => "void".to_owned(),
            TypeSpecifier::Struct(_) => "<struct type>".to_owned(),
            TypeSpecifier::Union(_) => "<union type>".to_owned(),
            TypeSpecifier::Enum(e) => e.reconstruct_source(),
            TypeSpecifier::CustomType(i) => i.reconstruct_source(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecifierQualifierToken {
    TypeSpecifier(TypeSpecifierToken),
    StorageClassSpecifier(StorageClassSpecifier),
    TypeQualifier(TypeQualifier),
    FunctionSpecifier(FunctionSpecifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecifierToken {
    ArithmeticType(ArithmeticTypeSpecifierToken),
    Void,
    Struct(StructType),
    Union(UnionType),
    Enum(EnumType),
    CustomType(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticTypeSpecifierToken {
    Char,
    Short,
    Int,
    Long,
    Signed,
    Unsigned,
    Float,
    Double,
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

impl ArithmeticType {
    fn create_from_type_specifiers(types: Vec<TypeSpecifierToken>) -> Result<Self, AstError> {
        // bitfield: double float | unsigned signed | long int short char
        let mut bitfield = 0;
        for t in types {
            match t {
                TypeSpecifierToken::ArithmeticType(at) => match at {
                    ArithmeticTypeSpecifierToken::Char => bitfield |= 0b1,
                    ArithmeticTypeSpecifierToken::Short => bitfield |= 0b10,
                    ArithmeticTypeSpecifierToken::Int => bitfield |= 0b100,
                    ArithmeticTypeSpecifierToken::Long => bitfield |= 0b1000,
                    ArithmeticTypeSpecifierToken::Signed => bitfield |= 0b1_0000,
                    ArithmeticTypeSpecifierToken::Unsigned => bitfield |= 0b10_0000,
                    ArithmeticTypeSpecifierToken::Float => bitfield |= 0b100_0000,
                    ArithmeticTypeSpecifierToken::Double => bitfield |= 0b1000_0000,
                },
                TypeSpecifierToken::Void
                | TypeSpecifierToken::Struct(_)
                | TypeSpecifierToken::Union(_)
                | TypeSpecifierToken::Enum(_)
                | TypeSpecifierToken::CustomType(_) => {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ))
                }
            }
        }
        match bitfield {
            // signed char (make char by itself be signed, like GCC)
            0b00_00_0001 | 0b00_01_0001 => Ok(ArithmeticType::I8),
            // unsigned char
            0b00_10_0001 => Ok(ArithmeticType::U8),
            // signed short
            0b00_00_0010 | 0b00_01_0010 => Ok(ArithmeticType::I16),
            // unsigned short
            0b00_10_0010 => Ok(ArithmeticType::U16),
            // signed int
            0b00_00_0100 | 0b00_01_0100 => Ok(ArithmeticType::I32),
            // unsigned int
            0b00_10_0100 => Ok(ArithmeticType::U32),
            // signed long
            0b00_00_1000 | 0b00_01_1000 => Ok(ArithmeticType::I64),
            // unsigned long
            0b00_10_1000 => Ok(ArithmeticType::U64),
            // float
            0b01_00_0000 => Ok(ArithmeticType::F32),
            // double
            0b10_00_0000 => Ok(ArithmeticType::F64),
            _ => Err(AstError::InvalidTypeDeclaration("Invalid arithmetic type")),
        }
    }
}

impl AstNode for ArithmeticType {
    fn reconstruct_source(&self) -> String {
        match self {
            ArithmeticType::I8 => "signed char".to_owned(),
            ArithmeticType::U8 => "unsigned char".to_owned(),
            ArithmeticType::I16 => "signed short".to_owned(),
            ArithmeticType::U16 => "unsigned short".to_owned(),
            ArithmeticType::I32 => "signed int".to_owned(),
            ArithmeticType::U32 => "unsigned int".to_owned(),
            ArithmeticType::I64 => "signed long".to_owned(),
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
                    Some(i) => writeln!(&mut s, "struct {} {{", i.reconstruct_source()).unwrap(),
                    None => writeln!(&mut s, "struct {{").unwrap(),
                }
                for member in ms {
                    write!(&mut s, "{}", member.reconstruct_source()).unwrap();
                }
                writeln!(&mut s, "}}").unwrap();
                s
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructMemberDeclaration(pub SpecifierQualifier, pub Vec<Declarator>);

impl AstNode for StructMemberDeclaration {
    fn reconstruct_source(&self) -> String {
        let mut s = String::new();
        for declarator in &self.1 {
            write!(&mut s, "{} ", self.0.reconstruct_source()).unwrap();
            writeln!(&mut s, "{};", declarator.reconstruct_source()).unwrap();
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
                    Some(i) => writeln!(&mut s, "union {} {{", i.reconstruct_source()).unwrap(),
                    None => writeln!(&mut s, "union {{").unwrap(),
                }
                for member in ms {
                    write!(&mut s, "{}", member.reconstruct_source()).unwrap();
                }
                writeln!(&mut s, "}}").unwrap();
                s
            }
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
                    Some(i) => writeln!(&mut s, "enum {} {{", i.reconstruct_source()).unwrap(),
                    None => writeln!(&mut s, "enum {{").unwrap(),
                }
                for enumerator in es {
                    writeln!(&mut s, "{}", enumerator.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}").unwrap();
                s
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Enumerator {
    Simple(Identifier),
    WithValue(Identifier, Expression),
}

impl AstNode for Enumerator {
    fn reconstruct_source(&self) -> String {
        match self {
            Enumerator::Simple(i) => format!("{}, ", i.reconstruct_source()),
            Enumerator::WithValue(i, e) => {
                format!("{} = {}, ", i.reconstruct_source(), e.reconstruct_source())
            }
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
pub enum FunctionSpecifier {
    Inline,
}

impl AstNode for FunctionSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            FunctionSpecifier::Inline => "inline".to_owned(),
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

impl Declarator {
    pub fn get_identifier_name(&self) -> Option<String> {
        match self {
            Declarator::Identifier(Identifier(i)) => Some(i.to_owned()),
            Declarator::PointerDeclarator(decl)
            | Declarator::ArrayDeclarator(decl, _)
            | Declarator::FunctionDeclarator(decl, _) => decl.get_identifier_name(),
            Declarator::AbstractPointerDeclarator => None,
        }
    }
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
            }
            ParameterTypeList::Variadic(ps) => {
                let mut s = String::new();
                for parameter in ps {
                    write!(&mut s, "{}, ", parameter.reconstruct_source()).unwrap();
                }
                write!(&mut s, "...").unwrap();
                s
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDeclaration(pub SpecifierQualifier, pub Option<Declarator>);

impl AstNode for ParameterDeclaration {
    fn reconstruct_source(&self) -> String {
        match &self.1 {
            None => self.0.reconstruct_source(),
            Some(d) => format!("{} {}", self.0.reconstruct_source(), d.reconstruct_source()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclaratorInitialiser {
    NoInit(Declarator),
    Init(Declarator, Initialiser),
    // Function(Box<Declarator>, Box<Statement>),
    // StructOrUnion(Box<Declarator>, Vec<Box<Expression>>),
}

impl DeclaratorInitialiser {
    pub fn get_identifier_name(&self) -> Option<String> {
        match self {
            DeclaratorInitialiser::NoInit(d)
            | DeclaratorInitialiser::Init(d, _)
            // | DeclaratorInitialiser::Function(d, _)
            // // | DeclaratorInitialiser::StructOrUnion(d, _)
                => d.get_identifier_name(),
        }
    }
}

impl AstNode for DeclaratorInitialiser {
    fn reconstruct_source(&self) -> String {
        match self {
            DeclaratorInitialiser::NoInit(d) => d.reconstruct_source(),
            DeclaratorInitialiser::Init(d, i) => {
                format!("{} = {}", d.reconstruct_source(), i.reconstruct_source())
            } // DeclaratorInitialiser::Function(d, s) => {
              //     format!("{} {}", d.reconstruct_source(), s.reconstruct_source())
              // } // DeclaratorInitialiser::StructOrUnion(d, es) => {
              //     let mut s = String::new();
              //     write!(&mut s, "{} = {{\n", d.reconstruct_source()).unwrap();
              //     for e in es {
              //         write!(&mut s, "{},\n", e.reconstruct_source()).unwrap();
              //     }
              //     write!(&mut s, "}}").unwrap();
              //     s
              // }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Initialiser {
    Expr(Expression),
    List(Vec<Initialiser>),
}

impl AstNode for Initialiser {
    fn reconstruct_source(&self) -> String {
        match self {
            Initialiser::Expr(e) => e.reconstruct_source(),
            Initialiser::List(is) => {
                let mut s = String::new();
                write!(&mut s, "{{").unwrap();
                for i in is {
                    write!(&mut s, "{}, ", i.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}").unwrap();
                s
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName(pub SpecifierQualifier, pub Option<Declarator>);

impl AstNode for TypeName {
    fn reconstruct_source(&self) -> String {
        let mut s = String::new();
        write!(&mut s, "{} ", self.0.reconstruct_source()).unwrap();
        match &self.1 {
            Some(d) => writeln!(&mut s, "{};", d.reconstruct_source()).unwrap(),
            None => (),
        }
        s
    }
}
