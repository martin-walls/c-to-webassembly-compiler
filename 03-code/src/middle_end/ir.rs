use crate::middle_end::ids::{FunId, Id, LabelId, StringLiteralId, StructId, UnionId, VarId};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir_types::{IrType, StructType};
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::parser::ast::UnionType;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct Function {
    pub instrs: Vec<Instruction>,
    pub type_info: Box<IrType>,
    // for each parameter, store which var it maps to
    pub param_var_mappings: Vec<VarId>,
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
    pub label_identifiers: HashMap<String, LabelId>,
    /// the highest label value currently in use
    max_label: Option<LabelId>,
    pub function_ids: HashMap<String, FunId>,
    pub functions: HashMap<FunId, Function>,
    max_fun_id: Option<FunId>,
    pub global_instrs: Vec<Instruction>,
    max_var: Option<VarId>,
    pub string_literals: HashMap<StringLiteralId, String>,
    max_string_literal_id: Option<StringLiteralId>,
    pub var_types: HashMap<VarId, Box<IrType>>,
    pub structs: HashMap<StructId, StructType>,
    max_struct_id: Option<StructId>,
    pub unions: HashMap<UnionId, UnionType>,
    max_union_id: Option<UnionId>,
    pub enum_member_values: HashMap<String, u64>,
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
            // declarations: HashMap::new(), todo remove
            var_types: HashMap::new(),
            structs: HashMap::new(),
            max_struct_id: None,
            unions: HashMap::new(),
            max_union_id: None,
            enum_member_values: HashMap::new(),
        }
    }

    pub fn new_label(&mut self) -> LabelId {
        let new_label = match &self.max_label {
            None => LabelId::initial_id(),
            Some(label) => label.next_id(),
        };
        self.max_label = Some(new_label.to_owned());
        new_label
    }

    pub fn new_identifier_label(&mut self, name: String) -> LabelId {
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

    pub fn get_fun_type(&self, fun_id: &FunId) -> Result<Box<IrType>, MiddleEndError> {
        match self.functions.get(fun_id) {
            None => Err(MiddleEndError::FunctionNotFoundForId(fun_id.to_owned())),
            Some(fun) => Ok(fun.type_info.to_owned()),
        }
    }

    pub fn new_var(&mut self) -> VarId {
        let new_var = match &self.max_var {
            None => VarId::initial_id(),
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

    pub fn add_var_type(
        &mut self,
        var: VarId,
        var_type: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
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
        let new_struct_id = match &self.max_struct_id {
            None => StructId::initial_id(),
            Some(struct_id) => struct_id.next_id(),
        };
        self.max_struct_id = Some(new_struct_id.to_owned());
        new_struct_id
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
            write!(f, "\n  {}: {}", var, type_info)?;
        }
        write!(f, "\nLabel identifiers:")?;
        for (name, label) in &self.label_identifiers {
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
        write!(f, "\n}}")
    }
}
