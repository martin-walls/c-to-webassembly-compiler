use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

use log::{debug, trace};

use crate::backend::target_code_generation::MAIN_FUNCTION_SOURCE_NAME;
use crate::middle_end::ids::{
    FunId, IdGenerator, InstructionId, LabelId, StringLiteralId, StructId, UnionId, ValueType,
    VarId,
};
use crate::middle_end::instructions::{Dest, Instruction};
use crate::middle_end::ir_types::{IrType, StructType, UnionType};
use crate::middle_end::middle_end_error::MiddleEndError;

#[derive(Debug)]
pub struct Function {
    pub instrs: Vec<Instruction>,
    pub type_info: Box<IrType>,
    // for each parameter, store which var it maps to
    pub param_var_mappings: Vec<VarId>,
    pub body_is_defined: bool,
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

#[derive(Debug)]
pub struct ProgramInstructions {
    pub functions: HashMap<FunId, Function>,
    pub global_instrs: Vec<Instruction>,
}

impl ProgramInstructions {
    pub fn new() -> Self {
        ProgramInstructions {
            functions: HashMap::new(),
            global_instrs: Vec::new(),
        }
    }

    pub fn insert_function(&mut self, fun_id: FunId, fun: Function) {
        self.functions.insert(fun_id, fun);
    }

    fn get_fun_type(&self, fun_id: &FunId) -> Result<Box<IrType>, MiddleEndError> {
        match self.functions.get(fun_id) {
            None => Err(MiddleEndError::FunctionNotFoundForId(fun_id.to_owned())),
            Some(fun) => Ok(fun.type_info.to_owned()),
        }
    }
}

impl fmt::Display for ProgramInstructions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(f, "\nGlobal instructions:")?;
        for instr in &self.global_instrs {
            write!(f, "\n  {}", instr)?;
        }
        write!(f, "\nFunction bodies:")?;
        for (fun_id, fun) in &self.functions {
            write!(f, "\nFunction {}:\n{}", fun_id, fun)?;
        }
        write!(f, "\n}}")
    }
}

#[derive(Debug)]
pub struct ProgramMetadata {
    instr_id_generator: IdGenerator<InstructionId>,
    pub label_id_generator: IdGenerator<LabelId>,
    fun_id_generator: IdGenerator<FunId>,
    var_id_generator: IdGenerator<VarId>,
    string_literal_id_generator: IdGenerator<StringLiteralId>,
    struct_id_generator: IdGenerator<StructId>,
    union_id_generator: IdGenerator<UnionId>,
    pub label_ids: HashMap<String, LabelId>,
    pub function_ids: HashMap<String, FunId>,
    pub function_types: HashMap<FunId, Box<IrType>>,
    pub function_param_var_mappings: HashMap<FunId, Vec<VarId>>,
    pub string_literals: HashMap<StringLiteralId, String>,
    pub var_types: HashMap<VarId, Box<IrType>>,
    pub structs: HashMap<StructId, StructType>,
    pub unions: HashMap<UnionId, UnionType>,
    pub enum_member_values: HashMap<String, u64>,
    pub null_dest_var: Option<Dest>,
}

impl ProgramMetadata {
    pub fn new() -> Self {
        ProgramMetadata {
            instr_id_generator: IdGenerator::new(),
            label_id_generator: IdGenerator::new(),
            fun_id_generator: IdGenerator::new(),
            var_id_generator: IdGenerator::new(),
            string_literal_id_generator: IdGenerator::new(),
            struct_id_generator: IdGenerator::new(),
            union_id_generator: IdGenerator::new(),
            label_ids: HashMap::new(),
            function_ids: HashMap::new(),
            function_types: HashMap::new(),
            function_param_var_mappings: HashMap::new(),
            string_literals: HashMap::new(),
            var_types: HashMap::new(),
            structs: HashMap::new(),
            unions: HashMap::new(),
            enum_member_values: HashMap::new(),
            null_dest_var: None,
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

    pub fn new_fun_declaration(&mut self, name: String) -> Result<FunId, MiddleEndError> {
        match self.function_ids.get(&name) {
            None => {}
            Some(_) => return Err(MiddleEndError::DuplicateFunctionDeclaration(name)),
        }
        let fun_id = self.fun_id_generator.new_id();
        self.function_ids.insert(name, fun_id.to_owned());
        Ok(fun_id)
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

    pub fn add_struct_type(&mut self, struct_type: StructType) -> Result<StructId, MiddleEndError> {
        // check if the same struct type has already been stored in program
        for (existing_struct_id, existing_struct_type) in &self.structs {
            if existing_struct_type == &struct_type {
                return Ok(existing_struct_id.to_owned());
            }
        }
        let struct_id = self.struct_id_generator.new_id();
        self.structs.insert(struct_id.to_owned(), struct_type);
        Ok(struct_id)
    }

    pub fn get_struct_type(&self, struct_id: &StructId) -> Result<StructType, MiddleEndError> {
        match self.structs.get(struct_id) {
            None => Err(MiddleEndError::TypeNotFound),
            Some(t) => Ok(t.to_owned()),
        }
    }

    pub fn add_union_type(&mut self, union_type: UnionType) -> Result<UnionId, MiddleEndError> {
        // check if the same union type has already been stored in program
        for (existing_union_id, existing_union_type) in &self.unions {
            if existing_union_type == &union_type {
                return Ok(existing_union_id.to_owned());
            }
        }
        let union_id = self.union_id_generator.new_id();
        self.unions.insert(union_id.to_owned(), union_type);
        Ok(union_id)
    }

    pub fn get_union_type(&self, union_id: &UnionId) -> Result<UnionType, MiddleEndError> {
        match self.unions.get(union_id) {
            None => Err(MiddleEndError::TypeNotFound),
            Some(t) => Ok(t.to_owned()),
        }
    }

    pub fn get_fun_type(&self, fun_id: &FunId) -> Result<Box<IrType>, MiddleEndError> {
        match self.function_types.get(fun_id) {
            None => Err(MiddleEndError::FunctionNotFoundForId(fun_id.to_owned())),
            Some(fun_type) => Ok(fun_type.to_owned()),
        }
    }

    pub fn get_main_fun_id(&self) -> Result<FunId, MiddleEndError> {
        match self.function_ids.get(MAIN_FUNCTION_SOURCE_NAME) {
            None => Err(MiddleEndError::NoMainFunctionDefined),
            Some(fun_id) => Ok(fun_id.to_owned()),
        }
    }

    pub fn init_null_dest_var(&mut self) -> Dest {
        match &self.null_dest_var {
            Some(dest) => {
                debug!("null var already initialised: {}", dest);
                dest.to_owned()
            }
            None => {
                let null_var = self.new_var(ValueType::None);
                self.null_dest_var = Some(null_var.to_owned());
                debug!("new null var: {}", null_var);
                null_var
            }
        }
    }

    pub fn is_var_the_null_dest(&self, dest: &Dest) -> bool {
        if let Some(null) = &self.null_dest_var {
            return null == dest;
        }
        // if there's no null dest var set, then false
        false
    }

    pub fn new_instr_id(&mut self) -> InstructionId {
        self.instr_id_generator.new_id()
    }
}

impl fmt::Display for ProgramMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(f, "\nFunction names:")?;
        for fun_name in self.function_ids.keys() {
            let fun_id = self.function_ids.get(fun_name).unwrap();
            write!(f, "\nFunction {} => {}", fun_name, fun_id)?;
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

pub struct Program {
    pub program_instructions: Box<ProgramInstructions>,
    pub program_metadata: Box<ProgramMetadata>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            program_instructions: Box::new(ProgramInstructions::new()),
            program_metadata: Box::new(ProgramMetadata::new()),
        }
    }

    pub fn new_label(&mut self) -> LabelId {
        self.program_metadata.new_label()
    }

    pub fn new_identifier_label(&mut self, name: String) -> LabelId {
        self.program_metadata.new_identifier_label(name)
    }

    pub fn resolve_identifier_to_label(&self, name: &str) -> Option<&LabelId> {
        self.program_metadata.label_ids.get(name)
    }

    pub fn new_fun_declaration(
        &mut self,
        name: String,
        fun: Function,
    ) -> Result<FunId, MiddleEndError> {
        let fun_id = self.program_metadata.new_fun_declaration(name)?;
        self.program_metadata
            .function_types
            .insert(fun_id.to_owned(), fun.type_info.to_owned());
        self.program_metadata
            .function_param_var_mappings
            .insert(fun_id.to_owned(), fun.param_var_mappings.to_vec());
        self.program_instructions
            .insert_function(fun_id.to_owned(), fun);
        Ok(fun_id)
    }

    pub fn new_fun_body(&mut self, name: String, fun: Function) -> Result<FunId, MiddleEndError> {
        match self.program_metadata.function_ids.get(&name) {
            None => self.new_fun_declaration(name, fun),
            Some(existing_fun_id) => {
                // body definition of existing function declaration
                // check whether this definition matches the earlier declaration
                let existing_fun = self
                    .program_instructions
                    .functions
                    .get(existing_fun_id)
                    .unwrap();
                if existing_fun.type_info != fun.type_info {
                    return Err(MiddleEndError::DuplicateFunctionDeclaration(name));
                }
                self.program_metadata
                    .function_types
                    .insert(existing_fun_id.to_owned(), fun.type_info.to_owned());
                self.program_metadata
                    .function_param_var_mappings
                    .insert(existing_fun_id.to_owned(), fun.param_var_mappings.to_vec());
                self.program_instructions
                    .insert_function(existing_fun_id.to_owned(), fun);
                Ok(existing_fun_id.to_owned())
            }
        }
    }

    pub fn get_fun_type(&self, fun_id: &FunId) -> Result<Box<IrType>, MiddleEndError> {
        self.program_instructions.get_fun_type(fun_id)
    }

    pub fn new_var(&mut self, value_type: ValueType) -> VarId {
        self.program_metadata.new_var(value_type)
    }

    pub fn new_string_literal(&mut self, s: String) -> StringLiteralId {
        self.program_metadata.new_string_literal(s)
    }

    pub fn add_var_type(
        &mut self,
        var: VarId,
        var_type: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
        self.program_metadata.add_var_type(var, var_type)
    }

    pub fn get_var_type(&self, var: &VarId) -> Result<Box<IrType>, MiddleEndError> {
        self.program_metadata.get_var_type(var)
    }

    pub fn add_struct_type(&mut self, struct_type: StructType) -> Result<StructId, MiddleEndError> {
        self.program_metadata.add_struct_type(struct_type)
    }

    pub fn get_struct_type(&self, struct_id: &StructId) -> Result<StructType, MiddleEndError> {
        self.program_metadata.get_struct_type(struct_id)
    }

    pub fn add_union_type(&mut self, union_type: UnionType) -> Result<UnionId, MiddleEndError> {
        self.program_metadata.add_union_type(union_type)
    }

    pub fn get_union_type(&self, union_id: &UnionId) -> Result<UnionType, MiddleEndError> {
        self.program_metadata.get_union_type(union_id)
    }

    pub fn new_instr_id(&mut self) -> InstructionId {
        self.program_metadata.new_instr_id()
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Instructions:\n{}", self.program_instructions)?;
        write!(f, "\nMetadata:\n{}", self.program_metadata)
    }
}
