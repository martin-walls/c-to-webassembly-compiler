use crate::middle_end::ids::{FunId, LabelId, VarId};
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir_types::IrType;
use crate::middle_end::middle_end_error::MiddleEndError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
    loop_stack: Vec<LoopOrSwitchContext>,
    scope_stack: Vec<Scope>,
    pub in_function_name_expr: bool,
    function_names: HashMap<String, FunId>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            loop_stack: Vec::new(),
            scope_stack: vec![Scope::new()], // start with a global scope
            in_function_name_expr: false,
            function_names: HashMap::new(),
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
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return false,
                Some(LoopOrSwitchContext::Switch(_)) => return true,
                Some(LoopOrSwitchContext::Loop(_)) => {}
            }
            i -= 1;
        }
    }

    pub fn get_switch_variable(&self) -> Option<VarId> {
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get(i) {
                None => return None,
                Some(LoopOrSwitchContext::Switch(switch_context)) => {
                    return Some(switch_context.switch_var.to_owned());
                }
                _ => {}
            }
            i -= 1;
        }
    }

    pub fn add_default_switch_case(
        &mut self,
        body: Vec<Instruction>,
    ) -> Result<(), MiddleEndError> {
        let mut i = self.loop_stack.len() - 1;
        loop {
            match self.loop_stack.get_mut(i) {
                None => return Err(MiddleEndError::DefaultOutsideSwitchContext),
                Some(loop_or_switch) => match loop_or_switch {
                    LoopOrSwitchContext::Loop(_) => {}
                    LoopOrSwitchContext::Switch(switch_context) => {
                        return switch_context.add_default_case(body);
                    }
                },
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
        match self.scope_stack.last_mut() {
            None => Err(MiddleEndError::ScopeError),
            Some(scope) => scope.add_var(name, var, type_info),
        }
    }

    pub fn resolve_identifier_to_var(
        &self,
        identifier_name: &str,
    ) -> Result<VarId, MiddleEndError> {
        if self.scope_stack.is_empty() {
            return Err(MiddleEndError::ScopeError);
        }
        let mut i = self.scope_stack.len() - 1;
        loop {
            match self.scope_stack.get(i) {
                None => {
                    return Err(MiddleEndError::UndeclaredIdentifier(
                        identifier_name.to_owned(),
                    ))
                }
                Some(scope) => match scope.resolve_identifier_to_var(identifier_name) {
                    None => {}
                    Some(var) => return Ok(var),
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

    pub fn add_function_declaration(
        &mut self,
        name: String,
        fun_id: FunId,
    ) -> Result<(), MiddleEndError> {
        // check for duplicate declarations
        match self.resolve_identifier_to_fun(&name) {
            Ok(_) => return Err(MiddleEndError::DuplicateFunctionDeclaration(name)),
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
}

impl Scope {
    fn new() -> Self {
        Scope {
            variable_names: HashMap::new(),
            variable_types: HashMap::new(),
            typedef_types: HashMap::new(),
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
    pub default_case: Option<Vec<Instruction>>, //todo redo switch case logic
}

impl SwitchContext {
    pub fn new(end_label: LabelId, switch_var: VarId) -> Self {
        SwitchContext {
            end_label,
            switch_var,
            default_case: None,
        }
    }

    pub fn add_default_case(&mut self, body: Vec<Instruction>) -> Result<(), MiddleEndError> {
        match self.default_case {
            None => {
                self.default_case = Some(body);
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
