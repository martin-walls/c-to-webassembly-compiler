pub trait AstNode {
    fn reconstruct_source(&self) -> ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(u128),
    Float(f64),
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

#[derive(Debug, Clone, PartialEq)]
pub enum LabelledStatement {
    Case,
    Default,
    Named(Identifier),
}

#[derive(Debug)]
pub struct StatementList(pub Vec<Box<Statement>>);

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

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    AddressOf,
    Dereference,
    Plus,
    Minus,
    BitwiseNot,
    LogicalNot,
    SizeOf,
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

#[derive(Debug, Clone, PartialEq)]
pub enum SpecifierQualifier {
    TypeSpecifier(TypeSpecifier),
    StorageClassSpecifier(StorageClassSpecifier),
    TypeQualifier(TypeQualifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecifier {
    ArithmeticType(ArithmeticTypeSpecifier),
    Void,
    Struct(StructType),
    Union(UnionType),
    Enum(EnumType),
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

#[derive(Debug, Clone, PartialEq)]
pub enum StructType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<StructMemberDeclaration>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructMemberDeclaration(pub Vec<SpecifierQualifier>, pub Vec<Box<Declarator>>);

#[derive(Debug, Clone, PartialEq)]
pub enum UnionType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<StructMemberDeclaration>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumType {
    Declaration(Identifier),
    Definition(Option<Identifier>, Vec<Enumerator>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Enumerator {
    Simple(Identifier),
    WithValue(Identifier, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClassSpecifier {
    Auto,
    Extern,
    Register,
    Static,
    Typedef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declarator {
    Identifier(Identifier),
    PointerDeclarator(Box<Declarator>),
    AbstractPointerDeclarator,
    ArrayDeclarator(Box<Declarator>, Option<Box<Expression>>),
    FunctionDeclarator(Box<Declarator>, Option<ParameterTypeList>),
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

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterDeclaration {
    Named(Vec<SpecifierQualifier>, Box<Declarator>),
    // Abstract(Vec<SpecifierQualifier>, Option<Box<AbstractDeclarator>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclaratorInitialiser {
    NoInit(Box<Declarator>),
    Init(Box<Declarator>, Box<Expression>),
    Function(Box<Declarator>, Box<Statement>),
    StructOrUnion(Box<Declarator>, Vec<Box<Expression>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeName(pub Vec<SpecifierQualifier>, pub Option<Box<Declarator>>);
