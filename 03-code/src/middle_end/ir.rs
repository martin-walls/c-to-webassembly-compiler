use std::collections::HashMap;

pub type Var = u32;

#[derive(Debug)]
pub enum Constant {
    Int(u128),
    Float(f64),
}

pub type Dest = Var;

#[derive(Debug)]
pub enum Src {
    Var(Var),
    Constant(Constant),
}

pub type Fun = u32;
pub type Label = u32;

#[derive(Debug)]
pub enum Instruction {
    // t = a
    SimpleAssignment(Dest, Src),
    // Unary operations
    // t = <op> a
    AddressOf(Dest, Src),
    Dereference(Dest, Src),
    BitwiseNot(Dest, Src),
    LogicalNot(Dest, Src),
    // Binary operations
    // t = a <op> b
    Mult(Dest, Src, Src),
    Div(Dest, Src, Src),
    Mod(Dest, Src, Src),
    Add(Dest, Src, Src),
    Sub(Dest, Src, Src),
    LeftShift(Dest, Src, Src),
    RightShift(Dest, Src, Src),
    BitwiseAnd(Dest, Src, Src),
    BitwiseOr(Dest, Src, Src),
    BitwiseXor(Dest, Src, Src),
    LogicalAnd(Dest, Src, Src),
    LogicalOr(Dest, Src, Src),

    // comparison
    LessThan(Dest, Src, Src),
    GreaterThan(Dest, Src, Src),
    LessThanEq(Dest, Src, Src),
    GreaterThanEq(Dest, Src, Src),
    Equal(Dest, Src, Src),
    NotEqual(Dest, Src, Src),

    // control flow
    Call(Option<Dest>, Fun, Vec<Src>),
    Ret(Option<Src>),
    Label(Label),
    Br(Label),
    BrIfEq(Src, Src, Label),
    BrIfNotEq(Src, Src, Label),
    BrIfGT(Src, Src, Label),
    BrIfLT(Src, Src, Label),
    BrIfGE(Src, Src, Label),
    BrIfLE(Src, Src, Label),
    Else(Label),

    StartBlock,
    EndBlock,
}

#[derive(Debug)]
pub struct Function {
    pub instrs: Vec<Instruction>,
}

#[derive(Debug)]
pub struct Program {
    // declarations
    pub label_identifiers: HashMap<String, Label>,
    /// the highest label value currently in use
    max_label: Option<Label>,
    pub functions: HashMap<String, Function>,
    max_var: Option<Var>,
    pub string_literals: HashMap<Var, String>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            label_identifiers: HashMap::new(),
            max_label: None,
            functions: HashMap::new(),
            max_var: None,
            string_literals: HashMap::new(),
        }
    }

    pub fn new_label(&mut self) -> Label {
        match self.max_label {
            None => self.max_label = Some(0),
            Some(label) => self.max_label = Some(label + 1),
        }
        self.max_label.unwrap()
    }

    pub fn new_identifier_label(&mut self, name: String) -> Label {
        let label = self.new_label();
        self.label_identifiers.insert(name, label);
        label
    }

    pub fn new_var(&mut self) -> Var {
        match self.max_var {
            None => self.max_var = Some(0),
            Some(var) => self.max_var = Some(var + 1),
        }
        self.max_var.unwrap()
    }

    pub fn new_string_literal(&mut self, s: String) -> Var {
        let var = self.new_var();
        self.string_literals.insert(var, s);
        var
    }
}
