#[derive(Debug, Clone)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub enum Constant {
  IntegerConstant(u128),
  FloatingConstant(f64),
}

#[derive(Debug, Clone)]
pub enum Statement {
  Block(Vec<Box<Statement>>),
  Goto(Identifier),
  Continue,
  Break,
  Return(Option<Box<Expression>>),
  While(Box<Expression>, Box<Statement>),
  DoWhile(Box<Statement>, Box<Expression>),
  For(Option<Box<Expression>>, Option<Box<Expression>>, Option<Box<Expression>>, Box<Statement>),
  If(Box<Expression>, Box<Statement>),
  IfElse(Box<Expression>, Box<Statement>, Box<Statement>),
  Switch(Box<Expression>, Box<Statement>),
  Labelled(LabelledStatement),
  Expr(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum LabelledStatement {
  Case,
  Default,
  Named(Identifier),
}

#[derive(Debug)]
pub struct StatementList(pub Vec<Statement>);

#[derive(Debug, Clone)]
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
  BinaryOp(BinaryOperator, Box<Expression>, Box<Expression>),
  Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
  Assignment(Box<Expression>, Box<Expression>, Option<BinaryOperator>),
  Cast(TypeSpecifier, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  AddressOf,
  Dereference,
  Plus,
  Minus,
  BitwiseNot,
  LogicalNot,
  SizeOf,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum TypeSpecifier {
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
  Void,
  Bool,
  Struct(Identifier),
  Union(Identifier),
}
