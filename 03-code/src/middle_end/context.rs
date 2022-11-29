use crate::middle_end::ids::{FunId, LabelId, StructId, UnionId, VarId};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir_types::{EnumConstant, IrType};
use crate::middle_end::middle_end_error::MiddleEndError;
use log::trace;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
    loop_stack: Vec<LoopOrSwitchContext>,
    scope_stack: Vec<Scope>,
    pub in_function_name_expr: bool,
    function_names: HashMap<String, FunId>,
    pub directly_on_lhs_of_assignment: bool,
}

pub enum IdentifierResolveResult {
    Var(VarId),
    EnumConst(EnumConstant),
}

impl Context {
    pub fn new() -> Self {
        Context {
            loop_stack: Vec::new(),
            scope_stack: vec![Scope::new()], // start with a global scope
            in_function_name_expr: false,
            function_names: HashMap::new(),
            directly_on_lhs_of_assignment: false,
        }
    }

    pub fn push_loop(&mut self, loop_context: LoopContext) {
        self.loop_stack
            .push(LoopOrSwitchContext::Loop(loop_context));
    }

    pub fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    pub fn push_switch(&mut self, switch_context: SwitchContext) {
        self.loop_stack
            .push(LoopOrSwitchContext::Switch(switch_context));
    }

    pub fn pop_switch(&mut self) -> Result<SwitchContext, MiddleEndError> {
        match self.loop_stack.pop() {
            None | Some(LoopOrSwitchContext::Loop(_)) => Err(MiddleEndError::LoopNestingError),
            Some(LoopOrSwitchContext::Switch(switch_context)) => Ok(switch_context),
        }
    }

    pub fn get_break_label(&self) -> Option<&LabelId> {
        match self.loop_stack.last() {
            None => None,
            Some(LoopOrSwitchContext::Loop(loop_context)) => Some(&loop_context.end_label),
            Some(LoopOrSwitchContext::Switch(switch_context)) => Some(&switch_context.end_label),
        }
    }

    pub fn get_continue_label(&self) -> Option<&LabelId> {
        if self.loop_stack.is_empty() {
            return None;
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return None,
                Some(LoopOrSwitchContext::Loop(loop_context)) => {
                    return Some(&loop_context.continue_label);
                }
                Some(LoopOrSwitchContext::Switch(_)) => {}
            }
            // if context was a switch context, keep looking backwards for the top loop context
            i -= 1;
        }
    }

    pub fn is_in_switch_context(&self) -> bool {
        if self.loop_stack.is_empty() {
            return false;
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return false,
                Some(LoopOrSwitchContext::Switch(_)) => return true,
                Some(LoopOrSwitchContext::Loop(_)) => {}
            }
            if i == 0 {
                return false;
            }
            i -= 1;
        }
    }

    pub fn get_switch_variable(&self) -> Option<VarId> {
        if self.loop_stack.is_empty() {
            return None;
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return None,
                Some(LoopOrSwitchContext::Switch(switch_context)) => {
                    return Some(switch_context.switch_var.to_owned());
                }
                _ => {}
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }

    pub fn new_switch_case_block(
        &mut self,
        case_body_label: LabelId,
        mut case_condition_instrs: Vec<Instruction>,
    ) -> Result<(), MiddleEndError> {
        if self.loop_stack.is_empty() {
            return Err(MiddleEndError::CaseOutsideSwitchContext);
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get_mut(i) {
                None => return Err(MiddleEndError::CaseOutsideSwitchContext),
                Some(loop_or_switch_context) => match loop_or_switch_context {
                    LoopOrSwitchContext::Loop(_) => {}
                    LoopOrSwitchContext::Switch(switch_context) => {
                        // every case block must have a label so it can be jumped to,
                        // so enforce that by adding it here
                        let case_instrs = vec![Instruction::Label(case_body_label)];
                        switch_context.case_blocks.push(case_instrs);
                        switch_context
                            .case_condition_instrs
                            .append(&mut case_condition_instrs);
                        return Ok(());
                    }
                },
            }
            if i == 0 {
                return Err(MiddleEndError::CaseOutsideSwitchContext);
            }
            i -= 1;
        }
    }

    pub fn push_instrs_to_switch_case_block(
        &mut self,
        instrs: Vec<Instruction>,
    ) -> Result<(), MiddleEndError> {
        if self.loop_stack.is_empty() {
            return Err(MiddleEndError::CaseOutsideSwitchContext);
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get_mut(i) {
                None => return Err(MiddleEndError::CaseOutsideSwitchContext),
                Some(loop_or_switch_context) => match loop_or_switch_context {
                    LoopOrSwitchContext::Loop(_) => {}
                    LoopOrSwitchContext::Switch(switch_context) => {
                        return switch_context.push_instrs_to_last_case_block(instrs);
                    }
                },
            }
            if i == 0 {
                return Err(MiddleEndError::CaseOutsideSwitchContext);
            }
            i -= 1;
        }
    }

    pub fn add_default_switch_block_label(&mut self, label: LabelId) -> Result<(), MiddleEndError> {
        if self.loop_stack.is_empty() {
            return Err(MiddleEndError::DefaultOutsideSwitchContext);
        }
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get_mut(i) {
                None => return Err(MiddleEndError::DefaultOutsideSwitchContext),
                Some(loop_or_switch_context) => match loop_or_switch_context {
                    LoopOrSwitchContext::Loop(_) => {}
                    LoopOrSwitchContext::Switch(switch_context) => {
                        return switch_context.add_default_block_label(label);
                    }
                },
            }
            if i == 0 {
                return Err(MiddleEndError::DefaultOutsideSwitchContext);
            }
            i -= 1;
        }
    }

    pub fn push_scope(&mut self) {
        self.scope_stack.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn add_variable_to_scope(
        &mut self,
        name: String,
        var: VarId,
        type_info: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
        trace!("adding variable \"{}\" to scope", name);
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_var(name, var, type_info),
        }
    }

    // pub fn resolve_identifier_to_var(
    //     &self,
    //     identifier_name: &str,
    // ) -> Result<VarId, MiddleEndError> {
    //     if self.scope_stack.is_empty() {
    //         return Err(MiddleEndError::ScopeError);
    //     }
    //     let mut i = self.scope_stack.len() - 1;
    //     loop {
    //         match self.scope_stack.get(i) {
    //             None => {
    //                 return Err(MiddleEndError::UndeclaredIdentifier(
    //                     identifier_name.to_owned(),
    //                 ))
    //             }
    //             Some(scope) => match scope.resolve_identifier_to_var(identifier_name) {
    //                 None => {}
    //                 Some(var) => return Ok(var),
    //             },
    //         }
    //         if i == 0 {
    //             return Err(MiddleEndError::UndeclaredIdentifier(
    //                 identifier_name.to_owned(),
    //             ));
    //         }
    //         i -= 1;
    //     }
    // }

    pub fn resolve_identifier_to_var_or_const(
        &self,
        identifier_name: &str,
    ) -> Result<IdentifierResolveResult, MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::ScopeError);
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => return Err(MiddleEndError::ScopeError),
                Some(scope) => match scope.resolve_identifier_to_var(identifier_name) {
                    None => match scope.resolve_identifier_to_enum_constant(identifier_name) {
                        None => {}
                        Some(c) => return Ok(IdentifierResolveResult::EnumConst(c)),
                    },
                    Some(var) => return Ok(IdentifierResolveResult::Var(var)),
                },
            }
            if i == 0 {
                return Err(MiddleEndError::UndeclaredIdentifier(
                    identifier_name.to_owned(),
                ));
            }
            i -= 1;
        }
    }

    pub fn add_enum_constant(
        &mut self,
        name: String,
        value: EnumConstant,
    ) -> Result<(), MiddleEndError> {
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_enum_constant(name, value),
        }
    }

    pub fn add_enum_tag(&mut self, name: String) -> Result<(), MiddleEndError> {
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_enum_tag(name),
        }
    }

    pub fn resolve_identifier_to_enum_tag(
        &self,
        identifier_name: &str,
    ) -> Result<(), MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::ScopeError);
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => return Err(MiddleEndError::ScopeError),
                Some(scope) => match scope.resolve_identifier_to_enum_tag(identifier_name) {
                    true => return Ok(()),
                    false => {}
                },
            }
            if i == 0 {
                return Err(MiddleEndError::UndeclaredEnumTag(
                    identifier_name.to_owned(),
                ));
            }
            i -= 1;
        }
    }

    pub fn add_struct_tag(
        &mut self,
        name: String,
        struct_id: StructId,
    ) -> Result<(), MiddleEndError> {
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_struct_tag(name, struct_id),
        }
    }

    pub fn resolve_struct_tag_to_struct_id(
        &self,
        identifier_name: &str,
    ) -> Result<StructId, MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::ScopeError);
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => return Err(MiddleEndError::ScopeError),
                Some(scope) => match scope.resolve_struct_tag_to_struct_id(identifier_name) {
                    Ok(id) => return Ok(id),
                    Err(_) => {}
                },
            }
            if i == 0 {
                return Err(MiddleEndError::UndeclaredStructTag(
                    identifier_name.to_owned(),
                ));
            }
            i -= 1;
        }
    }

    pub fn add_union_tag(&mut self, name: String, union_id: UnionId) -> Result<(), MiddleEndError> {
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_union_tag(name, union_id),
        }
    }

    pub fn resolve_union_tag_to_union_id(
        &self,
        identifier_name: &str,
    ) -> Result<UnionId, MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::ScopeError);
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => return Err(MiddleEndError::ScopeError),
                Some(scope) => match scope.resolve_union_tag_to_union_id(identifier_name) {
                    Ok(id) => return Ok(id),
                    Err(_) => {}
                },
            }
            if i == 0 {
                return Err(MiddleEndError::UndeclaredUnionTag(
                    identifier_name.to_owned(),
                ));
            }
            i -= 1;
        }
    }

    pub fn add_function_declaration(
        &mut self,
        name: String,
        fun_id: FunId,
    ) -> Result<(), MiddleEndError> {
        // check for duplicate declarations
        match self.resolve_identifier_to_fun(&name) {
            Ok(existing_fun_id) => {
                // if mapping already exists, don't do anything
                if existing_fun_id == fun_id {
                    return Ok(());
                }
                return Err(MiddleEndError::DuplicateFunctionDeclaration(name));
            }
            Err(_) => {}
        }
        self.function_names.insert(name, fun_id);
        Ok(())
    }

    pub fn resolve_identifier_to_fun(
        &self,
        identifier_name: &str,
    ) -> Result<FunId, MiddleEndError> {
        match self.function_names.get(identifier_name) {
            Some(fun_id) => Ok(fun_id.to_owned()),
            None => Err(MiddleEndError::UndeclaredIdentifier(
                identifier_name.to_owned(),
            )),
        }
    }

    pub fn add_typedef(
        &mut self,
        typedef_name: String,
        type_info: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
        // it's an error to redeclare the same typedef name with a different type
        match self.resolve_typedef(&typedef_name) {
            Ok(t) => {
                if t != type_info {
                    return Err(MiddleEndError::DuplicateTypeDeclaration(typedef_name));
                }
            }
            Err(_) => {}
        }
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_typedef(typedef_name, type_info),
        }
    }

    pub fn resolve_typedef(&self, typedef_name: &str) -> Result<Box<IrType>, MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::ScopeError);
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => return Err(MiddleEndError::UndeclaredType(typedef_name.to_owned())),
                Some(scope) => match scope.resolve_identifier_to_type(typedef_name) {
                    None => {}
                    Some(t) => return Ok(t),
                },
            }
            if i == 0 {
                return Err(MiddleEndError::UndeclaredType(typedef_name.to_owned()));
            }
            i -= 1;
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    /// map identifiers to variables in the IR
    variable_names: HashMap<String, VarId>,
    /// map variables to their type information
    variable_types: HashMap<VarId, Box<IrType>>,
    /// map typedef names to their types
    typedef_types: HashMap<String, Box<IrType>>,
    /// map enum constants to their integer values
    enum_constants: HashMap<String, EnumConstant>,
    /// List of the names of enum types that are declared
    enum_tags: Vec<String>,
    /// List of the names of struct types that are declared
    struct_tags: HashMap<String, StructId>,
    /// List of the names of union types that are declared
    union_tags: HashMap<String, UnionId>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            variable_names: HashMap::new(),
            variable_types: HashMap::new(),
            typedef_types: HashMap::new(),
            enum_constants: HashMap::new(),
            enum_tags: Vec::new(),
            struct_tags: HashMap::new(),
            union_tags: HashMap::new(),
        }
    }

    fn add_var(
        &mut self,
        identifier_name: String,
        var: VarId,
        type_info: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
        self.variable_names.insert(identifier_name, var.to_owned());
        self.variable_types.insert(var, type_info);
        Ok(())
    }

    fn resolve_identifier_to_var(&self, identifier_name: &str) -> Option<VarId> {
        match self.variable_names.get(identifier_name) {
            None => None,
            Some(var) => Some(var.to_owned()),
        }
    }

    fn add_typedef(
        &mut self,
        typedef_name: String,
        type_info: Box<IrType>,
    ) -> Result<(), MiddleEndError> {
        self.typedef_types.insert(typedef_name, type_info);
        Ok(())
    }

    fn resolve_identifier_to_type(&self, typedef_name: &str) -> Option<Box<IrType>> {
        match self.typedef_types.get(typedef_name) {
            None => None,
            Some(t) => Some(t.to_owned()),
        }
    }

    fn add_enum_constant(
        &mut self,
        name: String,
        value: EnumConstant,
    ) -> Result<(), MiddleEndError> {
        if self.enum_constants.contains_key(&name) {
            return Err(MiddleEndError::DuplicateEnumConstantDeclaration(name));
        }
        trace!("Setting enum constant {} = {}", name, value);
        self.enum_constants.insert(name, value);
        Ok(())
    }

    fn add_enum_tag(&mut self, name: String) -> Result<(), MiddleEndError> {
        if self.enum_tags.contains(&name) {
            return Err(MiddleEndError::DuplicateTypeDeclaration(name));
        }
        self.enum_tags.push(name);
        Ok(())
    }

    fn resolve_identifier_to_enum_constant(&self, identifier_name: &str) -> Option<EnumConstant> {
        match self.enum_constants.get(identifier_name) {
            None => None,
            Some(c) => Some(c.to_owned()),
        }
    }

    fn resolve_identifier_to_enum_tag(&self, identifier_name: &str) -> bool {
        self.enum_tags.contains(&identifier_name.to_owned())
    }

    fn add_struct_tag(&mut self, name: String, struct_id: StructId) -> Result<(), MiddleEndError> {
        if self.struct_tags.contains_key(&name) {
            return Err(MiddleEndError::DuplicateTypeDeclaration(name));
        }
        self.struct_tags.insert(name, struct_id);
        Ok(())
    }

    fn resolve_struct_tag_to_struct_id(
        &self,
        identifier_name: &str,
    ) -> Result<StructId, MiddleEndError> {
        match self.struct_tags.get(identifier_name) {
            None => Err(MiddleEndError::UndeclaredStructTag(
                identifier_name.to_owned(),
            )),
            Some(id) => Ok(id.to_owned()),
        }
    }

    fn add_union_tag(&mut self, name: String, union_id: UnionId) -> Result<(), MiddleEndError> {
        if self.union_tags.contains_key(&name) {
            return Err(MiddleEndError::DuplicateTypeDeclaration(name));
        }
        self.union_tags.insert(name, union_id);
        Ok(())
    }

    fn resolve_union_tag_to_union_id(
        &self,
        identifier_name: &str,
    ) -> Result<UnionId, MiddleEndError> {
        match self.union_tags.get(identifier_name) {
            None => Err(MiddleEndError::UndeclaredUnionTag(
                identifier_name.to_owned(),
            )),
            Some(id) => Ok(id.to_owned()),
        }
    }
}

#[derive(Debug)]
pub struct LoopContext {
    start_label: LabelId,
    end_label: LabelId,
    continue_label: LabelId,
}

impl LoopContext {
    pub fn while_loop(start_label: LabelId, end_label: LabelId) -> Self {
        LoopContext {
            start_label: start_label.to_owned(),
            end_label,
            continue_label: start_label,
        }
    }

    pub fn do_while_loop(
        start_label: LabelId,
        end_label: LabelId,
        continue_label: LabelId,
    ) -> Self {
        LoopContext {
            start_label,
            end_label,
            continue_label,
        }
    }

    pub fn for_loop(start_label: LabelId, end_label: LabelId, continue_label: LabelId) -> Self {
        LoopContext {
            start_label,
            end_label,
            continue_label,
        }
    }
}

#[derive(Debug)]
pub struct SwitchContext {
    pub end_label: LabelId,
    pub switch_var: VarId,
    pub case_condition_instrs: Vec<Instruction>,
    pub case_blocks: Vec<Vec<Instruction>>,
    pub default_block_label: Option<LabelId>,
}

impl SwitchContext {
    pub fn new(end_label: LabelId, switch_var: VarId) -> Self {
        SwitchContext {
            end_label,
            switch_var,
            case_condition_instrs: Vec::new(),
            case_blocks: Vec::new(),
            default_block_label: None,
        }
    }

    fn push_instrs_to_last_case_block(
        &mut self,
        mut instrs: Vec<Instruction>,
    ) -> Result<(), MiddleEndError> {
        match self.case_blocks.last_mut() {
            None => Err(MiddleEndError::NoCaseBlockToPushInstructionTo),
            Some(block_instrs) => {
                block_instrs.append(&mut instrs);
                Ok(())
            }
        }
    }

    fn add_default_block_label(&mut self, label: LabelId) -> Result<(), MiddleEndError> {
        match self.default_block_label {
            None => {
                self.default_block_label = Some(label);
                Ok(())
            }
            Some(_) => Err(MiddleEndError::MultipleDefaultCasesInSwitch),
        }
    }
}

#[derive(Debug)]
enum LoopOrSwitchContext {
    Loop(LoopContext),
    Switch(SwitchContext),
}
