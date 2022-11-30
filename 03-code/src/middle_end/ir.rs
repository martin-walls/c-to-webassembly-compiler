use crate::middle_end::ids::{
    FunId, IdGenerator, LabelId, StringLiteralId, StructId, UnionId, ValueType, VarId,
};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir_types::{IrType, StructType, UnionType};
use crate::middle_end::middle_end_error::MiddleEndError;
use log::trace;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Function {
    pub instrs: Vec<Instruction>,
    pub type_info: Box<IrType>,
    // for each parameter, store which var it maps to
    pub param_var_mappings: Vec<VarId>,
    body_is_defined: bool,
}

impl Function {
    pub fn new(
        instrs: Vec<Instruction>,
        type_info: Box<IrType>,
        param_var_mappings: Vec<VarId>,
    ) -> Self {
        Function {
            instrs,
            type_info,
            param_var_mappings,
            body_is_defined: true,
        }
    }

    pub fn declaration(type_info: Box<IrType>) -> Self {
        Function {
            instrs: Vec::new(),
            type_info,
            param_var_mappings: Vec::new(),
            body_is_defined: false,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(f, "\n  Function type:\n    {}", self.type_info)?;
        write!(f, "\n  Parameters: ")?;
        if !self.param_var_mappings.is_empty() {
            for i in 0..self.param_var_mappings.len() - 1 {
                write!(f, "{} => {}, ", i, self.param_var_mappings[i])?;
            }
            write!(
                f,
                "{} => {}",
                self.param_var_mappings.len() - 1,
                self.param_var_mappings[self.param_var_mappings.len() - 1]
            )?;
        }
        write!(f, "\n  Body instructions:")?;
        for instr in &self.instrs {
            write!(f, "\n    {}", instr)?;
        }
        write!(f, "\n}}")
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct TypeInfo {
//     pub type_: IrType,
// }
//
// impl TypeInfo {
//     pub fn new() -> Self {
//         TypeInfo {
//             type_: IrType::Void,
//         }
//     }
//
//     pub fn get_byte_size(&self) -> u64 {
//         self.type_.get_byte_size()
//     }
//
//     pub fn wrap_with_pointer(&mut self) {
//         self.type_ = IrType::PointerTo(Box::new(self.type_.to_owned()));
//     }
//
//     pub fn wrap_with_array(&mut self, size: Option<u64>) {
//         self.type_ = IrType::ArrayOf(Box::new(self.type_.to_owned()), size);
//     }
//
//     pub fn wrap_with_fun(&mut self, params: Vec<Box<TypeInfo>>) {
//         self.type_ = IrType::Function(Box::new(self.type_.to_owned()), params);
//     }
//
//     pub fn is_struct_union_or_enum(&self) -> bool {
//         match &self.type_ {
//             IrType::I8
//             | IrType::U8
//             | IrType::I16
//             | IrType::U16
//             | IrType::I32
//             | IrType::U32
//             | IrType::I64
//             | IrType::U64
//             | IrType::F32
//             | IrType::F64
//             | IrType::Void
//             | IrType::PointerTo(_)
//             | IrType::ArrayOf(_, _)
//             | IrType::Function(_, _) => false,
//             IrType::Struct(_) | IrType::Union(_) => true,
//         }
//     }
// }
//
// impl fmt::Display for TypeInfo {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.type_)
//     }
// }

#[derive(Debug)]
pub struct Program {
    pub label_id_generator: IdGenerator<LabelId>,
    fun_id_generator: IdGenerator<FunId>,
    var_id_generator: IdGenerator<VarId>,
    string_literal_id_generator: IdGenerator<StringLiteralId>,
    struct_id_generator: IdGenerator<StructId>,
    union_id_generator: IdGenerator<UnionId>,
    pub label_ids: HashMap<String, LabelId>,
    pub function_ids: HashMap<String, FunId>,
    pub functions: HashMap<FunId, Function>,
    pub global_instrs: Vec<Instruction>,
    pub string_literals: HashMap<StringLiteralId, String>,
    pub var_types: HashMap<VarId, Box<IrType>>,
    pub structs: HashMap<StructId, StructType>,
    pub unions: HashMap<UnionId, UnionType>,
    pub enum_member_values: HashMap<String, u64>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            label_id_generator: IdGenerator::new(),
            fun_id_generator: IdGenerator::new(),
            var_id_generator: IdGenerator::new(),
            string_literal_id_generator: IdGenerator::new(),
            struct_id_generator: IdGenerator::new(),
            union_id_generator: IdGenerator::new(),
            label_ids: HashMap::new(),
            function_ids: HashMap::new(),
            functions: HashMap::new(),
            global_instrs: Vec::new(),
            string_literals: HashMap::new(),
            var_types: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            enum_member_values: HashMap::new(),
        }
    }

    pub fn new_label(&mut self) -> LabelId {
        self.label_id_generator.new_id()
    }

    pub fn new_identifier_label(&mut self, name: String) -> LabelId {
        let label = self.new_label();
        self.label_ids.insert(name, label.to_owned());
        label
    }

    fn new_fun_id(&mut self, name: String) -> FunId {
        let new_fun_id = self.fun_id_generator.new_id();
        self.function_ids.insert(name, new_fun_id.to_owned());
        new_fun_id
    }

    pub fn new_fun_declaration(
        &mut self,
        name: String,
        fun: Function,
    ) -> Result<FunId, MiddleEndError> {
        match self.function_ids.get(&name) {
            None => {}
            Some(_) => return Err(MiddleEndError::DuplicateFunctionDeclaration(name)),
        }
        let fun_id = self.new_fun_id(name);
        self.functions.insert(fun_id.to_owned(), fun);
        Ok(fun_id)
    }

    pub fn new_fun_body(&mut self, name: String, fun: Function) -> Result<FunId, MiddleEndError> {
        match self.function_ids.get(&name) {
            None => {
                // new function declaration
                let fun_id = self.new_fun_id(name);
                self.functions.insert(fun_id.to_owned(), fun);
                Ok(fun_id)
            }
            Some(fun_id) => {
                // body definition of existing function declaration
                // check whether this definition matches the earlier declaration
                let existing_fun = self.functions.get(fun_id).unwrap();
                if existing_fun.type_info != fun.type_info {
                    return Err(MiddleEndError::DuplicateFunctionDeclaration(name));
                }
                trace!("Adding fun body: {}", name);
                self.functions.insert(fun_id.to_owned(), fun);
                Ok(fun_id.to_owned())
            }
        }
    }

    pub fn get_fun_type(&self, fun_id: &FunId) -> Result<Box<IrType>, MiddleEndError> {
        match self.functions.get(fun_id) {
            None => Err(MiddleEndError::FunctionNotFoundForId(fun_id.to_owned())),
            Some(fun) => Ok(fun.type_info.to_owned()),
        }
    }

    pub fn new_var(&mut self, value_type: ValueType) -> VarId {
        let mut new_var = self.var_id_generator.new_id();
        new_var.set_value_type(value_type);
        new_var
    }

    pub fn new_string_literal(&mut self, s: String) -> StringLiteralId {
        let new_string_id = self.string_literal_id_generator.new_id();
        self.string_literals.insert(new_string_id.to_owned(), s);
        new_string_id
    }

    pub fn add_var_type(
        &mut self,
        var: VarId,
        var_type: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
        trace!("Setting type {} = {}", var, var_type);
        if self.var_types.contains_key(&var) {
            return Err(MiddleEndError::RedeclaredVarType(var));
        }
        self.var_types.insert(var, var_type);
        Ok(())
    }

    pub fn get_var_type(&self, var: &VarId) -> Result<Box<IrType>, MiddleEndError> {
        match self.var_types.get(var) {
            None => Err(MiddleEndError::TypeNotFound),
            Some(t) => Ok(t.to_owned()),
        }
    }

    fn new_struct_id(&mut self) -> StructId {
        self.struct_id_generator.new_id()
    }

    pub fn add_struct_type(&mut self, struct_type: StructType) -> Result<StructId, MiddleEndError> {
        // check if the same struct type has already been stored in program
        for (existing_struct_id, existing_struct_type) in &self.structs {
            if existing_struct_type == &struct_type {
                return Ok(existing_struct_id.to_owned());
            }
        }
        let struct_id = self.new_struct_id();
        self.structs.insert(struct_id.to_owned(), struct_type);
        Ok(struct_id)
    }

    pub fn get_struct_type(&self, struct_id: &StructId) -> Result<StructType, MiddleEndError> {
        match self.structs.get(struct_id) {
            None => Err(MiddleEndError::TypeNotFound),
            Some(t) => Ok(t.to_owned()),
        }
    }

    fn new_union_id(&mut self) -> UnionId {
        self.union_id_generator.new_id()
    }

    pub fn add_union_type(&mut self, union_type: UnionType) -> Result<UnionId, MiddleEndError> {
        // check if the same union type has already been stored in program
        for (existing_union_id, existing_union_type) in &self.unions {
            if existing_union_type == &union_type {
                return Ok(existing_union_id.to_owned());
            }
        }
        let union_id = self.new_union_id();
        self.unions.insert(union_id.to_owned(), union_type);
        Ok(union_id)
    }

    pub fn get_union_type(&self, union_id: &UnionId) -> Result<UnionType, MiddleEndError> {
        match self.unions.get(union_id) {
            None => Err(MiddleEndError::TypeNotFound),
            Some(t) => Ok(t.to_owned()),
        }
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
        write!(f, "\nVar types:")?;
        for (var, type_info) in &self.var_types {
            write!(f, "\n  {} ({}): {}", var, var.get_value_type(), type_info)?;
        }
        write!(f, "\nLabel identifiers:")?;
        for (name, label) in &self.label_ids {
            write!(f, "\n  \"{}\": {}", name, label)?;
        }
        write!(f, "\nString literals:")?;
        for (id, string) in &self.string_literals {
            write!(f, "\n  {}: \"{}\"", id, string)?;
        }
        write!(f, "\nStruct types:")?;
        for (id, type_info) in &self.structs {
            write!(f, "\n  {}: {}", id, type_info)?;
        }
        write!(f, "\nUnion types:")?;
        for (id, type_info) in &self.unions {
            write!(f, "\n  {}: {}", id, type_info)?;
        }
        write!(f, "\n}}")
    }
}
