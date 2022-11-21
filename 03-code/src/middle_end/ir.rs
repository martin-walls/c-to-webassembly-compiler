use crate::middle_end::middle_end_error::MiddleEndError;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

const POINTER_SIZE: u64 = 4; // bytes

/// A type representing an identifier in the IR.
/// E.g. variable identifiers, function identifiers.
///
/// The trait is an abstraction for generating new identifiers.
trait Id {
    /// Generate the initial id, when no IDs exist yet. (Id 0)
    fn initial_id() -> Self;
    /// Generate a new id, given the current max id. (Id n+1)
    fn next_id(&self) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var(u64);

impl Id for Var {
    fn initial_id() -> Self {
        Var(0)
    }

    fn next_id(&self) -> Self {
        Var(self.0 + 1)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i128),
    Float(f64),
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Int(i) => {
                write!(f, "{}", i)
            }
            Constant::Float(fl) => {
                write!(f, "{}", fl)
            }
        }
    }
}

pub type Dest = Var;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunId(u64);

impl Id for FunId {
    fn initial_id() -> Self {
        FunId(0)
    }

    fn next_id(&self) -> Self {
        FunId(self.0 + 1)
    }
}

impl fmt::Display for FunId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "f{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum Src {
    Var(Var),
    Constant(Constant),
    Fun(FunId),
}

impl fmt::Display for Src {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Src::Var(var) => {
                write!(f, "{}", var)
            }
            Src::Constant(c) => {
                write!(f, "{}", c)
            }
            Src::Fun(fun) => {
                write!(f, "{}", fun)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(u64);

impl Id for Label {
    fn initial_id() -> Self {
        Label(0)
    }

    fn next_id(&self) -> Self {
        Label(self.0 + 1)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "l{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringLiteralId(u64);

impl Id for StringLiteralId {
    fn initial_id() -> Self {
        StringLiteralId(0)
    }

    fn next_id(&self) -> Self {
        StringLiteralId(self.0 + 1)
    }
}

impl fmt::Display for StringLiteralId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "s{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructTypeId(u64);

impl Id for StructTypeId {
    fn initial_id() -> Self {
        StructTypeId(0)
    }

    fn next_id(&self) -> Self {
        StructTypeId(self.0 + 1)
    }
}

impl fmt::Display for StructTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "struct{}", self.0)
    }
}

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

    PointerToStringLiteral(Dest, StringLiteralId),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::SimpleAssignment(dest, src) => {
                write!(f, "{} = {}", dest, src)
            }
            Instruction::AddressOf(dest, src) => {
                write!(f, "{} = &{}", dest, src)
            }
            Instruction::Dereference(dest, src) => {
                write!(f, "{} = *{}", dest, src)
            }
            Instruction::BitwiseNot(dest, src) => {
                write!(f, "{} = ~{}", dest, src)
            }
            Instruction::LogicalNot(dest, src) => {
                write!(f, "{} = !{}", dest, src)
            }
            Instruction::Mult(dest, left, right) => {
                write!(f, "{} = {} * {}", dest, left, right)
            }
            Instruction::Div(dest, left, right) => {
                write!(f, "{} = {} / {}", dest, left, right)
            }
            Instruction::Mod(dest, left, right) => {
                write!(f, "{} = {} % {}", dest, left, right)
            }
            Instruction::Add(dest, left, right) => {
                write!(f, "{} = {} + {}", dest, left, right)
            }
            Instruction::Sub(dest, left, right) => {
                write!(f, "{} = {} - {}", dest, left, right)
            }
            Instruction::LeftShift(dest, left, right) => {
                write!(f, "{} = {} << {}", dest, left, right)
            }
            Instruction::RightShift(dest, left, right) => {
                write!(f, "{} = {} >> {}", dest, left, right)
            }
            Instruction::BitwiseAnd(dest, left, right) => {
                write!(f, "{} = {} & {}", dest, left, right)
            }
            Instruction::BitwiseOr(dest, left, right) => {
                write!(f, "{} = {} | {}", dest, left, right)
            }
            Instruction::BitwiseXor(dest, left, right) => {
                write!(f, "{} = {} ^ {}", dest, left, right)
            }
            Instruction::LogicalAnd(dest, left, right) => {
                write!(f, "{} = {} && {}", dest, left, right)
            }
            Instruction::LogicalOr(dest, left, right) => {
                write!(f, "{} = {} || {}", dest, left, right)
            }
            Instruction::LessThan(dest, left, right) => {
                write!(f, "{} = {} < {}", dest, left, right)
            }
            Instruction::GreaterThan(dest, left, right) => {
                write!(f, "{} = {} > {}", dest, left, right)
            }
            Instruction::LessThanEq(dest, left, right) => {
                write!(f, "{} = {} <= {}", dest, left, right)
            }
            Instruction::GreaterThanEq(dest, left, right) => {
                write!(f, "{} = {} >= {}", dest, left, right)
            }
            Instruction::Equal(dest, left, right) => {
                write!(f, "{} = {} == {}", dest, left, right)
            }
            Instruction::NotEqual(dest, left, right) => {
                write!(f, "{} = {} != {}", dest, left, right)
            }
            Instruction::Call(dest, fun, params) => {
                write!(f, "{} = call {}(", dest, fun)?;
                for param in &params[..params.len() - 1] {
                    write!(f, "{}, ", param)?;
                }
                write!(f, "{})", params[params.len() - 1])
            }
            Instruction::Ret(src) => match src {
                None => {
                    write!(f, "return")
                }
                Some(src) => {
                    write!(f, "return {}", src)
                }
            },
            Instruction::Label(label) => {
                write!(f, "{}:", label)
            }
            Instruction::Br(label) => {
                write!(f, "goto {}", label)
            }
            Instruction::BrIfEq(left, right, label) => {
                write!(f, "if {} == {} goto {}", left, right, label)
            }
            Instruction::BrIfNotEq(left, right, label) => {
                write!(f, "if {} != {} goto {}", left, right, label)
            }
            Instruction::BrIfGT(left, right, label) => {
                write!(f, "if {} > {} goto {}", left, right, label)
            }
            Instruction::BrIfLT(left, right, label) => {
                write!(f, "if {} < {} goto {}", left, right, label)
            }
            Instruction::BrIfGE(left, right, label) => {
                write!(f, "if {} >= {} goto {}", left, right, label)
            }
            Instruction::BrIfLE(left, right, label) => {
                write!(f, "if {} <= {} goto {}", left, right, label)
            }
            Instruction::PointerToStringLiteral(dest, string_id) => {
                write!(f, "{} = pointer to string literal {}", dest, string_id)
            }
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub instrs: Vec<Instruction>,
    pub type_info: Box<TypeInfo>,
    // for each parameter, store which var it maps to
    pub param_var_mappings: Vec<Var>,
}

impl Function {
    pub fn new(
        instrs: Vec<Instruction>,
        type_info: Box<TypeInfo>,
        param_var_mappings: Vec<Var>,
    ) -> Self {
        Function {
            instrs,
            type_info,
            param_var_mappings,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(f, "\n  Function type:\n    {}", self.type_info)?;
        write!(f, "\n  Parameters: ")?;
        for i in 0..self.param_var_mappings.len() - 1 {
            write!(f, "{} => {}, ", i, self.param_var_mappings[i])?;
        }
        write!(
            f,
            "{} => {}",
            self.param_var_mappings.len() - 1,
            self.param_var_mappings[self.param_var_mappings.len() - 1]
        )?;
        write!(f, "\n  Body instructions:")?;
        for instr in &self.instrs {
            write!(f, "\n    {}", instr)?;
        }
        write!(f, "\n}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    pub name: Option<String>,
    /// store members' names and types
    pub member_types: HashMap<String, Box<TypeInfo>>,
    pub member_byte_offsets: HashMap<String, u64>,
    pub total_byte_size: u64,
}

impl StructType {
    pub fn named(name: String) -> Self {
        StructType {
            name: Some(name),
            member_types: HashMap::new(),
            member_byte_offsets: HashMap::new(),
            total_byte_size: 0,
        }
    }

    pub fn unnamed() -> Self {
        StructType {
            name: None,
            member_types: HashMap::new(),
            member_byte_offsets: HashMap::new(),
            total_byte_size: 0,
        }
    }

    pub fn push_member(
        &mut self,
        member_name: String,
        member_type: Box<TypeInfo>,
    ) -> Result<(), MiddleEndError> {
        // check if member with same name already exists
        if self.member_types.contains_key(&member_name) {
            return Err(MiddleEndError::DuplicateStructMember);
        }
        let byte_size = member_type.get_byte_size();
        self.member_types
            .insert(member_name.to_owned(), member_type);
        // store byte offset of this member and update total byte size of struct
        self.member_byte_offsets
            .insert(member_name, self.total_byte_size);
        self.total_byte_size += byte_size;
        Ok(())
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!("fmt::Display for StructType")
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    Struct(StructType),
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
            Type::Struct(struct_type) => struct_type.total_byte_size,
            Type::Union(members) => {
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => {
                write!(f, "signed char")
            }
            Type::U8 => {
                write!(f, "unsigned char")
            }
            Type::I16 => {
                write!(f, "signed short")
            }
            Type::U16 => {
                write!(f, "unsigned short")
            }
            Type::I32 => {
                write!(f, "signed int")
            }
            Type::U32 => {
                write!(f, "unsigned int")
            }
            Type::I64 => {
                write!(f, "signed long")
            }
            Type::U64 => {
                write!(f, "unsigned long")
            }
            Type::F32 => {
                write!(f, "float")
            }
            Type::F64 => {
                write!(f, "double")
            }
            Type::Struct(struct_type) => {
                write!(f, "struct {}", struct_type)
            }
            Type::Union(members) => {
                write!(f, "union {{")?;
                for member in &members[..members.len() - 1] {
                    write!(f, "{}, ", member)?;
                }
                write!(f, "{}", members[members.len() - 1])?;
                write!(f, "}}")
            }
            Type::Void => {
                write!(f, "void")
            }
            Type::PointerTo(t) => {
                write!(f, "*({})", t)
            }
            Type::ArrayOf(t, size) => match size {
                Some(size) => write!(f, "({})[{}]", t, size),
                None => write!(f, "({})[]", t),
            },
            Type::Function(ret, params) => {
                write!(f, "({})(", ret)?;
                for param in &params[..params.len() - 1] {
                    write!(f, "{}, ", param)?;
                }
                write!(f, "{})", params[params.len() - 1])
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub type_: Type,
    /// If a struct/union/enum is declared but not defined, this will be false
    pub is_defined: bool,
    // todo mapping of struct/union fields to their type
}

impl TypeInfo {
    pub fn new() -> Self {
        TypeInfo {
            type_: Type::Void,
            is_defined: true,
        }
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

    pub fn is_struct_union_or_enum(&self) -> bool {
        match &self.type_ {
            Type::I8
            | Type::U8
            | Type::I16
            | Type::U16
            | Type::I32
            | Type::U32
            | Type::I64
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Void
            | Type::PointerTo(_)
            | Type::ArrayOf(_, _)
            | Type::Function(_, _) => false,
            Type::Struct(_) | Type::Union(_) => true,
        }
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_)
    }
}

#[derive(Debug)]
pub struct Program {
    pub label_identifiers: HashMap<String, Label>,
    /// the highest label value currently in use
    max_label: Option<Label>,
    pub function_ids: HashMap<String, FunId>,
    pub functions: HashMap<FunId, Function>,
    max_fun_id: Option<FunId>,
    pub global_instrs: Vec<Instruction>,
    max_var: Option<Var>,
    pub string_literals: HashMap<StringLiteralId, String>,
    max_string_literal_id: Option<StringLiteralId>,
    pub declarations: HashMap<String, TypeInfo>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            label_identifiers: HashMap::new(),
            max_label: None,
            function_ids: HashMap::new(),
            functions: HashMap::new(),
            max_fun_id: None,
            global_instrs: Vec::new(),
            max_var: None,
            string_literals: HashMap::new(),
            max_string_literal_id: None,
            declarations: HashMap::new(),
        }
    }

    pub fn new_label(&mut self) -> Label {
        let new_label = match &self.max_label {
            None => Label::initial_id(),
            Some(label) => label.next_id(),
        };
        self.max_label = Some(new_label.to_owned());
        new_label
    }

    pub fn new_identifier_label(&mut self, name: String) -> Label {
        let label = self.new_label();
        self.label_identifiers.insert(name, label.to_owned());
        label
    }

    fn new_fun_id(&mut self, name: String) -> FunId {
        let new_fun_id = match &self.max_fun_id {
            None => FunId::initial_id(),
            Some(fun_id) => fun_id.next_id(),
        };
        self.max_fun_id = Some(new_fun_id.to_owned());
        self.function_ids.insert(name, new_fun_id.to_owned());
        new_fun_id
    }

    pub fn new_fun(&mut self, name: String, fun: Function) -> FunId {
        let fun_id = self.new_fun_id(name);
        self.functions.insert(fun_id.to_owned(), fun);
        fun_id
    }

    pub fn new_var(&mut self) -> Var {
        let new_var = match &self.max_var {
            None => Var::initial_id(),
            Some(var) => var.next_id(),
        };
        self.max_var = Some(new_var.to_owned());
        new_var
    }

    pub fn new_string_literal(&mut self, s: String) -> StringLiteralId {
        let new_string_id = match &self.max_string_literal_id {
            None => StringLiteralId::initial_id(),
            Some(string_id) => string_id.next_id(),
        };
        self.max_string_literal_id = Some(new_string_id.to_owned());
        self.string_literals.insert(new_string_id.to_owned(), s);
        new_string_id
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
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(f, "\nGlobal instructions:")?;
        for instr in &self.global_instrs {
            write!(f, "\n  {}", instr)?;
        }
        for fun_name in self.function_ids.keys() {
            let fun_id = self.function_ids.get(fun_name).unwrap();
            let fun = self.functions.get(fun_id).unwrap();
            write!(f, "\nFunction {} => {}\n{}", fun_name, fun_id, fun)?;
        }
        write!(f, "\nLabel identifiers: {:#?}", self.label_identifiers)?;
        write!(f, "\nString literals: {:#?}", self.string_literals)?;
        write!(f, "\nDeclarations: {:#?}", self.declarations)?;
        write!(f, "\n}}")
    }
}
