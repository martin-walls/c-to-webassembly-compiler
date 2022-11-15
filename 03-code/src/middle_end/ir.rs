use crate::middle_end::middle_end_error::MiddleEndError;
use std::collections::HashMap;

const POINTER_SIZE: u64 = 4; // bytes

pub type Var = u64;

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i128),
    Float(f64),
}

pub type Dest = Var;

pub type Fun = u64;

#[derive(Debug, Clone)]
pub enum Src {
    Var(Var),
    Constant(Constant),
    Fun(Fun),
}

pub type Label = u64;

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
    Add(Dest, Src, Src), // todo if adding to a pointer, add by the size of the object it points to
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
    Call(Dest, Src, Vec<Src>),
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

#[derive(Debug, Clone)]
pub enum Type {
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
    Struct(Vec<Box<TypeInfo>>),
    Union(Vec<Box<TypeInfo>>),
    Void,
    PointerTo(Box<Type>),
    /// array type, array size
    ArrayOf(Box<Type>, Option<u64>),
    /// return type, parameter types
    Function(Box<Type>, Vec<Box<TypeInfo>>),
}

impl Type {
    pub fn get_byte_size(&self) -> u64 {
        match &self {
            Type::I8 | Type::U8 => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 => 4,
            Type::I64 | Type::U64 => 8,
            Type::F32 => 4,
            Type::F64 => 8,
            Type::Struct(members) | Type::Union(members) => {
                let mut total = 0;
                for type_info in members {
                    total += type_info.get_byte_size();
                }
                total
            }
            Type::Void => 0,
            Type::PointerTo(_) => POINTER_SIZE,
            Type::ArrayOf(t, count) => match count {
                None => {
                    unreachable!()
                }
                Some(count) => t.get_byte_size() * count,
            },
            Type::Function(_, _) => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub type_: Type,
    // todo mapping of struct/union fields to their type
}

impl TypeInfo {
    pub fn new() -> Self {
        TypeInfo { type_: Type::Void }
    }

    pub fn get_byte_size(&self) -> u64 {
        self.type_.get_byte_size()
    }

    pub fn wrap_with_pointer(&mut self) {
        self.type_ = Type::PointerTo(Box::new(self.type_.to_owned()));
    }

    pub fn wrap_with_array(&mut self, size: Option<u64>) {
        self.type_ = Type::ArrayOf(Box::new(self.type_.to_owned()), size);
    }

    pub fn wrap_with_fun(&mut self, params: Vec<Box<TypeInfo>>) {
        self.type_ = Type::Function(Box::new(self.type_.to_owned()), params);
    }
}

#[derive(Debug)]
pub struct Program {
    pub label_identifiers: HashMap<String, Label>,
    /// the highest label value currently in use
    max_label: Option<Label>,
    pub functions: HashMap<String, Function>,
    max_var: Option<Var>,
    pub string_literals: HashMap<Var, String>,
    pub declarations: HashMap<String, TypeInfo>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            label_identifiers: HashMap::new(),
            max_label: None,
            functions: HashMap::new(),
            max_var: None,
            string_literals: HashMap::new(),
            declarations: HashMap::new(),
        }
    }

    pub fn new_label(&mut self) -> Label {
        match self.max_label {
            None => self.max_label = Some(0),
            Some(label) => self.max_label = Some(label + 1),
        }
        println!("new label");
        self.max_label.unwrap()
    }

    pub fn new_identifier_label(&mut self, name: String) -> Label {
        let label = self.new_label();
        self.label_identifiers.insert(name, label);
        println!("adding label");
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

    pub fn add_declaration(
        &mut self,
        name: String,
        type_info: TypeInfo,
    ) -> Result<(), MiddleEndError> {
        if self.declarations.contains_key(&name) {
            return Err(MiddleEndError::DuplicateDeclaration(name));
        }
        self.declarations.insert(name, type_info);
        Ok(())
    }

    pub fn resolve_typedef(&self, typedef_name: &str) -> Result<TypeInfo, MiddleEndError> {
        todo!()
    }
}
