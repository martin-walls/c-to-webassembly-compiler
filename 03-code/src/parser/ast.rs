use std::error::Error;
use std::fmt;
use std::fmt::{Formatter, Write};

pub trait AstNode {
    fn reconstruct_source(&self) -> String;
    fn normalise(self) -> Self;
}

#[derive(Debug)]
pub enum AstError {
    InvalidTypeDeclaration(&'static str),
    TooManyStorageClassSpecifiers(StorageClassSpecifier),
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AstError::InvalidTypeDeclaration(msg) => write!(f, "{}", msg),
            AstError::TooManyStorageClassSpecifiers(_) => {
                write!(f, "Too many storage class specifiers")
            }
        }
    }
}

impl Error for AstError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

impl AstNode for Identifier {
    fn reconstruct_source(&self) -> String {
        self.0.to_owned()
    }

    fn normalise(self) -> Self {
        self
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

    fn normalise(self) -> Self {
        self
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
        Option<ExpressionOrDeclaration>,
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
    NormalisedDeclaration(NormalisedSpecifierQualifier, Vec<DeclaratorInitialiser>),
    EmptyDeclaration(Vec<SpecifierQualifier>),
    NormalisedEmptyDeclaration(NormalisedSpecifierQualifier),
    FunctionDeclaration(Vec<SpecifierQualifier>, Box<Declarator>, Box<Statement>),
    NormalisedFunctionDeclaration(
        NormalisedSpecifierQualifier,
        Box<Declarator>,
        Box<Statement>,
    ),
    Empty,
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
            }
            Statement::Continue => format!("continue;"),
            Statement::Break => format!("break;"),
            Statement::Return(e) => match e {
                Some(e) => format!("return {};", e.reconstruct_source()),
                None => format!("return;"),
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
                    for specifier in s {
                        write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                    }
                    write!(&mut st, "{};", declarator.reconstruct_source()).unwrap();
                }
                st
            }
            Statement::EmptyDeclaration(s) => {
                let mut st = String::new();
                for specifier in s {
                    write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                }
                write!(&mut st, ";").unwrap();
                st
            }
            Statement::FunctionDeclaration(s, d, b) => {
                let mut st = String::new();
                for specifier in s {
                    write!(&mut st, "{} ", specifier.reconstruct_source()).unwrap();
                }
                write!(
                    &mut st,
                    "{} {}",
                    d.reconstruct_source(),
                    b.reconstruct_source()
                )
                .unwrap();
                st
            }
            Statement::Empty => "".to_owned(),
            Statement::NormalisedDeclaration(sq, d) => {
                let mut st = String::new();
                for declarator in d {
                    write!(&mut st, "{} ", sq.reconstruct_source()).unwrap();
                    write!(&mut st, "{};", declarator.reconstruct_source()).unwrap();
                }
                st
            }
            Statement::NormalisedEmptyDeclaration(sq) => sq.reconstruct_source(),
            Statement::NormalisedFunctionDeclaration(sq, d, s) => {
                let mut st = String::new();
                write!(&mut st, "{} ", sq.reconstruct_source()).unwrap();
                write!(
                    &mut st,
                    "{} {}",
                    d.reconstruct_source(),
                    s.reconstruct_source()
                )
                .unwrap();
                st
            }
        }
    }

    fn normalise(self) -> Self {
        match self {
            Statement::Goto(_) | Statement::Continue | Statement::Break => self,
            Statement::Block(stmts) => {
                let mut new_stmts = Vec::new();
                for s in stmts {
                    match *s {
                        Statement::Empty => continue,
                        s => new_stmts.push(Box::new(s.normalise())),
                    }
                }
                Statement::Block(new_stmts)
            }
            Statement::Return(e) => match e {
                None => Statement::Return(None),
                Some(e) => Statement::Return(Some(Box::new(e.normalise()))),
            },
            Statement::While(e, s) => {
                Statement::While(Box::new(e.normalise()), Box::new(s.normalise()))
            }
            Statement::DoWhile(s, e) => {
                Statement::DoWhile(Box::new(s.normalise()), Box::new(e.normalise()))
            }
            Statement::For(e1, e2, e3, s) => {
                let e1n = match e1 {
                    None => None,
                    Some(e) => Some(e.normalise()),
                };
                let e2n = match e2 {
                    None => None,
                    Some(e) => Some(Box::new(e.normalise())),
                };
                let e3n = match e3 {
                    None => None,
                    Some(e) => Some(Box::new(e.normalise())),
                };
                Statement::For(e1n, e2n, e3n, Box::new(s.normalise()))
            }
            Statement::If(e, s) => Statement::If(Box::new(e.normalise()), Box::new(s.normalise())),
            Statement::IfElse(e, s1, s2) => Statement::IfElse(
                Box::new(e.normalise()),
                Box::new(s1.normalise()),
                Box::new(s2.normalise()),
            ),
            Statement::Switch(e, s) => {
                Statement::Switch(Box::new(e.normalise()), Box::new(s.normalise()))
            }
            Statement::Labelled(s) => Statement::Labelled(s.normalise()),
            Statement::Expr(e) => Statement::Expr(Box::new(e.normalise())),
            Statement::Declaration(sqs, ds) => {
                let mut new_ds = Vec::new();
                for d in ds {
                    new_ds.push(d.normalise());
                }
                Statement::NormalisedDeclaration(
                    NormalisedSpecifierQualifier::create(sqs).unwrap(),
                    new_ds,
                )
            }
            Statement::EmptyDeclaration(sqs) => Statement::NormalisedEmptyDeclaration(
                NormalisedSpecifierQualifier::create(sqs).unwrap(),
            ),
            Statement::FunctionDeclaration(sqs, d, s) => Statement::NormalisedFunctionDeclaration(
                NormalisedSpecifierQualifier::create(sqs).unwrap(),
                Box::new(d.normalise()),
                Box::new(s.normalise()),
            ),
            Statement::Empty => Statement::Empty,
            Statement::NormalisedDeclaration(_, _)
            | Statement::NormalisedEmptyDeclaration(_)
            | Statement::NormalisedFunctionDeclaration(_, _, _) => self,
        }
    }
}

// for 'for' statements, where the first expression can be either an expression
// or a declaration
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionOrDeclaration {
    Expression(Box<Expression>),
    Declaration(Box<Statement>), // the Statement should be of value statement::Declarator
}

impl AstNode for ExpressionOrDeclaration {
    fn reconstruct_source(&self) -> String {
        match self {
            ExpressionOrDeclaration::Expression(e) => e.reconstruct_source(),
            ExpressionOrDeclaration::Declaration(d) => d.reconstruct_source(),
        }
    }

    fn normalise(self) -> Self {
        match self {
            ExpressionOrDeclaration::Expression(e) => {
                ExpressionOrDeclaration::Expression(Box::new(e.normalise()))
            }
            ExpressionOrDeclaration::Declaration(d) => {
                ExpressionOrDeclaration::Declaration(Box::new(d.normalise()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelledStatement {
    Case(Box<Expression>, Box<Statement>),
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

    fn normalise(self) -> Self {
        match self {
            LabelledStatement::Case(e, s) => {
                LabelledStatement::Case(Box::new(e.normalise()), Box::new(s.normalise()))
            }
            LabelledStatement::Default(s) => LabelledStatement::Default(Box::new(s.normalise())),
            LabelledStatement::Named(i, s) => LabelledStatement::Named(i, Box::new(s.normalise())),
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

    fn normalise(self) -> Self {
        let mut v: Vec<Box<Statement>> = Vec::new();
        for s in self.0 {
            v.push(Box::new(s.normalise()));
        }
        StatementList(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Constant(Constant),
    StringLiteral(String),
    Index(Box<Expression>, Box<Expression>),
    FunctionCall(Box<Expression>, Vec<Box<Expression>>),
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
            Expression::Index(e1, e2) => {
                format!("{}[{}]", e1.reconstruct_source(), e2.reconstruct_source())
            }
            Expression::FunctionCall(e1, e2) => {
                let mut s = String::new();
                write!(&mut s, "{}(", e1.reconstruct_source()).unwrap();
                for e in &e2[..e2.len() - 1] {
                    write!(&mut s, "{}, ", e.reconstruct_source()).unwrap();
                }
                if e2.len() > 0 {
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
            Expression::Assignment(l, r, op) => match op {
                None => format!("{} = {}", l.reconstruct_source(), r.reconstruct_source()),
                Some(op) => format!(
                    "{} {}= {}",
                    l.reconstruct_source(),
                    op.reconstruct_source(),
                    r.reconstruct_source()
                ),
            },
            Expression::Cast(t, e) => {
                format!("({}) {}", t.reconstruct_source(), e.reconstruct_source())
            }
            Expression::ExpressionList(e1, e2) => {
                format!("{}, {}", e1.reconstruct_source(), e2.reconstruct_source())
            }
        }
    }

    fn normalise(self) -> Self {
        match self {
            Expression::Identifier(_) | Expression::Constant(_) | Expression::StringLiteral(_) => {
                self
            }
            Expression::Index(e1, e2) => {
                Expression::Index(Box::new(e1.normalise()), Box::new(e2.normalise()))
            }
            Expression::FunctionCall(e, args) => {
                let mut new_args = Vec::new();
                for a in args {
                    new_args.push(Box::new(a.normalise()));
                }
                Expression::FunctionCall(Box::new(e.normalise()), new_args)
            }
            Expression::DirectMemberSelection(e, i) => {
                Expression::DirectMemberSelection(Box::new(e.normalise()), i)
            }
            Expression::IndirectMemberSelection(e, i) => {
                Expression::IndirectMemberSelection(Box::new(e.normalise()), i)
            }
            Expression::PostfixIncrement(e) => {
                Expression::PostfixIncrement(Box::new(e.normalise()))
            }
            Expression::PostfixDecrement(e) => {
                Expression::PostfixDecrement(Box::new(e.normalise()))
            }
            Expression::PrefixIncrement(e) => Expression::PrefixIncrement(Box::new(e.normalise())),
            Expression::PrefixDecrement(e) => Expression::PrefixDecrement(Box::new(e.normalise())),
            Expression::UnaryOp(op, e) => Expression::UnaryOp(op, Box::new(e.normalise())),
            Expression::SizeOfExpr(e) => Expression::SizeOfExpr(Box::new(e.normalise())),
            Expression::SizeOfType(t) => Expression::SizeOfType(t.normalise()),
            Expression::BinaryOp(op, e1, e2) => {
                Expression::BinaryOp(op, Box::new(e1.normalise()), Box::new(e2.normalise()))
            }
            Expression::Ternary(e1, e2, e3) => Expression::Ternary(
                Box::new(e1.normalise()),
                Box::new(e2.normalise()),
                Box::new(e3.normalise()),
            ),
            Expression::Assignment(e1, e2, op) => {
                Expression::Assignment(Box::new(e1.normalise()), Box::new(e2.normalise()), op)
            }
            Expression::Cast(t, e) => Expression::Cast(t.normalise(), Box::new(e.normalise())),
            Expression::ExpressionList(e1, e2) => {
                Expression::ExpressionList(Box::new(e1.normalise()), Box::new(e2.normalise()))
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

    fn normalise(self) -> Self {
        self
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

    fn normalise(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalisedSpecifierQualifier {
    type_specifier: NormalisedTypeSpecifier,
    storage_class_specifier: Option<StorageClassSpecifier>,
    const_: bool,
    inline: bool,
}

impl NormalisedSpecifierQualifier {
    fn create(sqs: Vec<SpecifierQualifier>) -> Result<Self, AstError> {
        let mut type_specifiers: Vec<TypeSpecifier> = Vec::new();
        let mut storage_class_specifier = None;
        let mut const_ = false;
        let mut inline = false;

        for sq in sqs {
            match sq {
                SpecifierQualifier::TypeSpecifier(t) => type_specifiers.push(t),
                SpecifierQualifier::StorageClassSpecifier(s) => {
                    if storage_class_specifier == None {
                        storage_class_specifier = Some(s);
                    } else {
                        return Err(AstError::TooManyStorageClassSpecifiers(s));
                    }
                }
                SpecifierQualifier::TypeQualifier(q) => match q {
                    TypeQualifier::Const => const_ = true,
                },
                SpecifierQualifier::FunctionSpecifier(f) => match f {
                    FunctionSpecifier::Inline => inline = true,
                },
            }
        }

        let type_specifier = NormalisedTypeSpecifier::create(type_specifiers)?;

        Ok(NormalisedSpecifierQualifier {
            type_specifier,
            storage_class_specifier,
            const_,
            inline,
        })
    }
}

impl AstNode for NormalisedSpecifierQualifier {
    fn reconstruct_source(&self) -> String {
        let mut st = String::new();
        write!(&mut st, "{}", self.type_specifier.reconstruct_source()).unwrap();
        match &self.storage_class_specifier {
            None => {}
            Some(s) => write!(&mut st, " {}", s.reconstruct_source()).unwrap(),
        }
        if self.const_ {
            write!(&mut st, " const").unwrap()
        }
        if self.inline {
            write!(&mut st, " inline").unwrap()
        }
        st
    }

    fn normalise(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalisedTypeSpecifier {
    ArithmeticType(ArithmeticType),
    Void,
    Struct(NormalisedStructType),
    Union(NormalisedUnionType),
    Enum(EnumType),
    CustomType(Identifier),
}

impl NormalisedTypeSpecifier {
    fn create(types: Vec<TypeSpecifier>) -> Result<Self, AstError> {
        if types.len() == 0 {
            return Err(AstError::InvalidTypeDeclaration("No type specified"));
        }
        match &types[0] {
            TypeSpecifier::ArithmeticType(_) => {
                let arithmetic_type = ArithmeticType::create_from_type_specifiers(types);
                match arithmetic_type {
                    Ok(t) => Ok(NormalisedTypeSpecifier::ArithmeticType(t)),
                    Err(e) => Err(e),
                }
            }
            TypeSpecifier::Void => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(NormalisedTypeSpecifier::Void)
            }
            TypeSpecifier::Struct(t) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                match NormalisedStructType::create(t.to_owned()) {
                    Ok(st) => Ok(NormalisedTypeSpecifier::Struct(st)),
                    Err(e) => Err(e),
                }
            }
            TypeSpecifier::Union(t) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                match NormalisedUnionType::create(t.to_owned()) {
                    Ok(ut) => Ok(NormalisedTypeSpecifier::Union(ut)),
                    Err(e) => Err(e),
                }
            }
            TypeSpecifier::Enum(t) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(NormalisedTypeSpecifier::Enum(t.to_owned()))
            }
            TypeSpecifier::CustomType(i) => {
                if types.len() > 1 {
                    return Err(AstError::InvalidTypeDeclaration(
                        "Conflicting types declared",
                    ));
                }
                Ok(NormalisedTypeSpecifier::CustomType(i.to_owned()))
            }
        }
    }
}

impl AstNode for NormalisedTypeSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            NormalisedTypeSpecifier::ArithmeticType(t) => t.reconstruct_source(),
            NormalisedTypeSpecifier::Void => "void".to_owned(),
            NormalisedTypeSpecifier::Struct(_) => "<struct type>".to_owned(),
            NormalisedTypeSpecifier::Union(_) => "<union type>".to_owned(),
            NormalisedTypeSpecifier::Enum(e) => e.reconstruct_source(),
            NormalisedTypeSpecifier::CustomType(i) => i.reconstruct_source(),
        }
    }

    fn normalise(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalisedStructType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<NormalisedStructMemberDeclaration>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalisedStructMemberDeclaration(
    pub NormalisedSpecifierQualifier,
    pub Vec<Box<Declarator>>,
);

impl NormalisedStructType {
    fn create(t: StructType) -> Result<Self, AstError> {
        match t {
            StructType::Declaration(i) => Ok(NormalisedStructType::Declaration(i)),
            StructType::Definition(i, members) => {
                let mut normalised_members: Vec<NormalisedStructMemberDeclaration> = Vec::new();
                for m in members {
                    let normalised_sq = NormalisedSpecifierQualifier::create(m.0)?;
                    normalised_members.push(NormalisedStructMemberDeclaration(normalised_sq, m.1));
                }
                Ok(NormalisedStructType::Definition(
                    i.to_owned(),
                    normalised_members,
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NormalisedUnionType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<NormalisedStructMemberDeclaration>),
}

impl NormalisedUnionType {
    fn create(t: UnionType) -> Result<Self, AstError> {
        match t {
            UnionType::Declaration(i) => Ok(NormalisedUnionType::Declaration(i)),
            UnionType::Definition(i, members) => {
                let mut normalised_members: Vec<NormalisedStructMemberDeclaration> = Vec::new();
                for m in members {
                    let normalised_sq = NormalisedSpecifierQualifier::create(m.0)?;
                    normalised_members.push(NormalisedStructMemberDeclaration(normalised_sq, m.1));
                }
                Ok(NormalisedUnionType::Definition(
                    i.to_owned(),
                    normalised_members,
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecifierQualifier {
    TypeSpecifier(TypeSpecifier),
    StorageClassSpecifier(StorageClassSpecifier),
    TypeQualifier(TypeQualifier),
    FunctionSpecifier(FunctionSpecifier),
}

impl AstNode for SpecifierQualifier {
    fn reconstruct_source(&self) -> String {
        match self {
            SpecifierQualifier::TypeSpecifier(t) => t.reconstruct_source(),
            SpecifierQualifier::StorageClassSpecifier(s) => s.reconstruct_source(),
            SpecifierQualifier::TypeQualifier(q) => q.reconstruct_source(),
            SpecifierQualifier::FunctionSpecifier(f) => f.reconstruct_source(),
        }
    }

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecifier {
    ArithmeticType(ArithmeticTypeSpecifier),
    Void,
    Struct(StructType),
    Union(UnionType),
    Enum(EnumType),
    CustomType(Identifier),
}

impl AstNode for TypeSpecifier {
    fn reconstruct_source(&self) -> String {
        match self {
            TypeSpecifier::ArithmeticType(t) => t.reconstruct_source(),
            TypeSpecifier::Void => "void".to_owned(),
            TypeSpecifier::Struct(t) => t.reconstruct_source(),
            TypeSpecifier::Union(t) => t.reconstruct_source(),
            TypeSpecifier::Enum(t) => t.reconstruct_source(),
            TypeSpecifier::CustomType(i) => format!("<TypedefName {}>", i.reconstruct_source()),
        }
    }

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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

impl ArithmeticType {
    fn create_from_type_specifiers(types: Vec<TypeSpecifier>) -> Result<Self, AstError> {
        // bitfield: double float | unsigned signed | long int short char
        let mut bitfield = 0;
        for t in types {
            match t {
                TypeSpecifier::ArithmeticType(at) => match at {
                    ArithmeticTypeSpecifier::Char => bitfield |= 0b1,
                    ArithmeticTypeSpecifier::Short => bitfield |= 0b10,
                    ArithmeticTypeSpecifier::Int => bitfield |= 0b100,
                    ArithmeticTypeSpecifier::Long => bitfield |= 0b1000,
                    ArithmeticTypeSpecifier::Signed => bitfield |= 0b1_0000,
                    ArithmeticTypeSpecifier::Unsigned => bitfield |= 0b10_0000,
                    ArithmeticTypeSpecifier::Float => bitfield |= 0b100_0000,
                    ArithmeticTypeSpecifier::Double => bitfield |= 0b1000_0000,
                },
                TypeSpecifier::Void
                | TypeSpecifier::Struct(_)
                | TypeSpecifier::Union(_)
                | TypeSpecifier::Enum(_)
                | TypeSpecifier::CustomType(_) => {
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

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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
                    None => write!(&mut s, "struct {{\n").unwrap(),
                }
                for member in ms {
                    write!(&mut s, "{}", member.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}\n").unwrap();
                s
            }
        }
    }

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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
                    None => write!(&mut s, "union {{\n").unwrap(),
                }
                for member in ms {
                    write!(&mut s, "{}", member.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}\n").unwrap();
                s
            }
        }
    }

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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
                    None => write!(&mut s, "enum {{\n").unwrap(),
                }
                for enumerator in es {
                    write!(&mut s, "{}\n", enumerator.reconstruct_source()).unwrap();
                }
                write!(&mut s, "}}").unwrap();
                s
            }
        }
    }

    fn normalise(self) -> Self {
        match self {
            EnumType::Declaration(_) => self,
            EnumType::Definition(i, members) => {
                let mut new_members = Vec::new();
                for m in members {
                    new_members.push(m.normalise());
                }
                EnumType::Definition(i, new_members)
            }
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
            Enumerator::WithValue(i, e) => {
                format!("{} = {}, ", i.reconstruct_source(), e.reconstruct_source())
            }
        }
    }

    fn normalise(self) -> Self {
        match self {
            Enumerator::Simple(_) => self,
            Enumerator::WithValue(i, e) => Enumerator::WithValue(i, Box::new(e.normalise())),
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

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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

    // this will never be called, because we construct normalised specifier
    // qualifier from the vector of specifier qualifiers
    fn normalise(self) -> Self {
        unreachable!()
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

    fn normalise(self) -> Self {
        match self {
            Declarator::Identifier(_) | Declarator::AbstractPointerDeclarator => self,
            Declarator::PointerDeclarator(d) => {
                Declarator::PointerDeclarator(Box::new(d.normalise()))
            }
            Declarator::ArrayDeclarator(d, e) => match e {
                None => Declarator::ArrayDeclarator(Box::new(d.normalise()), None),
                Some(e) => Declarator::ArrayDeclarator(
                    Box::new(d.normalise()),
                    Some(Box::new(e.normalise())),
                ),
            },
            Declarator::FunctionDeclarator(d, params) => match params {
                None => Declarator::FunctionDeclarator(Box::new(d.normalise()), None),
                Some(p) => {
                    Declarator::FunctionDeclarator(Box::new(d.normalise()), Some(p.normalise()))
                }
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

    fn normalise(self) -> Self {
        match self {
            ParameterTypeList::Normal(params) => {
                let mut new_params = Vec::new();
                for p in params {
                    new_params.push(p.normalise());
                }
                ParameterTypeList::Normal(new_params)
            }
            ParameterTypeList::Variadic(params) => {
                let mut new_params = Vec::new();
                for p in params {
                    new_params.push(p.normalise());
                }
                ParameterTypeList::Variadic(new_params)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterDeclaration {
    Named(Vec<SpecifierQualifier>, Box<Declarator>),
    NormalisedNamed(NormalisedSpecifierQualifier, Box<Declarator>),
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
            ParameterDeclaration::NormalisedNamed(sq, d) => {
                format!("{} {}", sq.reconstruct_source(), d.reconstruct_source())
            }
        }
    }

    fn normalise(self) -> Self {
        match self {
            ParameterDeclaration::Named(sqs, d) => ParameterDeclaration::NormalisedNamed(
                NormalisedSpecifierQualifier::create(sqs).unwrap(),
                Box::new(d.normalise()),
            ),
            ParameterDeclaration::NormalisedNamed(_, _) => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclaratorInitialiser {
    NoInit(Box<Declarator>),
    Init(Box<Declarator>, Box<Initialiser>),
    Function(Box<Declarator>, Box<Statement>),
    // StructOrUnion(Box<Declarator>, Vec<Box<Expression>>),
}

impl DeclaratorInitialiser {
    pub fn get_identifier_name(&self) -> Option<String> {
        match self {
            DeclaratorInitialiser::NoInit(d)
            | DeclaratorInitialiser::Init(d, _)
            | DeclaratorInitialiser::Function(d, _)
            // | DeclaratorInitialiser::StructOrUnion(d, _)
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
            }
            DeclaratorInitialiser::Function(d, s) => {
                format!("{} {}", d.reconstruct_source(), s.reconstruct_source())
            } // DeclaratorInitialiser::StructOrUnion(d, es) => {
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

    fn normalise(self) -> Self {
        match self {
            DeclaratorInitialiser::NoInit(d) => {
                DeclaratorInitialiser::NoInit(Box::new(d.normalise()))
            }
            DeclaratorInitialiser::Init(d, i) => {
                DeclaratorInitialiser::Init(Box::new(d.normalise()), Box::new(i.normalise()))
            }
            DeclaratorInitialiser::Function(d, s) => {
                DeclaratorInitialiser::Function(Box::new(d.normalise()), Box::new(s.normalise()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Initialiser {
    Expr(Box<Expression>),
    List(Vec<Box<Initialiser>>),
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

    fn normalise(self) -> Self {
        match self {
            Initialiser::Expr(e) => Initialiser::Expr(Box::new(e.normalise())),
            Initialiser::List(inits) => {
                let mut new_inits = Vec::new();
                for i in inits {
                    new_inits.push(Box::new(i.normalise()));
                }
                Initialiser::List(new_inits)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Unnormalised(Vec<SpecifierQualifier>, Option<Box<Declarator>>),
    Normalised(NormalisedSpecifierQualifier, Option<Box<Declarator>>),
}

impl AstNode for TypeName {
    fn reconstruct_source(&self) -> String {
        match self {
            TypeName::Unnormalised(sqs, d) => {
                let mut s = String::new();
                for specifier in sqs {
                    write!(&mut s, "{} ", specifier.reconstruct_source()).unwrap();
                }
                match d {
                    Some(d) => write!(&mut s, "{};\n", d.reconstruct_source()).unwrap(),
                    None => (),
                }
                s
            }
            TypeName::Normalised(sq, d) => {
                let mut s = String::new();
                write!(&mut s, "{} ", sq.reconstruct_source()).unwrap();
                match d {
                    Some(d) => write!(&mut s, "{};\n", d.reconstruct_source()).unwrap(),
                    None => (),
                }
                s
            }
        }
    }

    fn normalise(self) -> Self {
        match self {
            TypeName::Unnormalised(sqs, d) => match d {
                None => {
                    TypeName::Normalised(NormalisedSpecifierQualifier::create(sqs).unwrap(), None)
                }
                Some(d) => TypeName::Normalised(
                    NormalisedSpecifierQualifier::create(sqs).unwrap(),
                    Some(Box::new(d.normalise())),
                ),
            },
            TypeName::Normalised(_, _) => self,
        }
    }
}
