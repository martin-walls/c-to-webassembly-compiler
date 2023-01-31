use crate::backend::wasm_indices::{FuncIdx, WasmIdx};
use crate::middle_end::ids::Id;
use crate::middle_end::ids::{FunId, StringLiteralId, VarId};
use crate::relooper::blocks::{LoopBlockId, MultipleBlockId};
use crate::relooper::relooper::ReloopedFunction;
use std::collections::HashMap;

pub struct ModuleContext {
    pub fun_id_to_func_idx_map: HashMap<FunId, FuncIdx>,
    // (inclusive, exclusive)
    pub imported_func_idx_range: (FuncIdx, FuncIdx),
    pub defined_func_idx_range: (FuncIdx, FuncIdx),
    pub string_literal_id_to_ptr_map: HashMap<StringLiteralId, u32>,
    pub log_stack_ptr_fun_id: Option<FunId>,
}

impl ModuleContext {
    pub fn new() -> Self {
        ModuleContext {
            fun_id_to_func_idx_map: HashMap::new(),
            // func_idx_to_fun_id_map: HashMap::new(),
            imported_func_idx_range: (FuncIdx::initial_idx(), FuncIdx::initial_idx()),
            defined_func_idx_range: (FuncIdx::initial_idx(), FuncIdx::initial_idx()),
            string_literal_id_to_ptr_map: HashMap::new(),
            log_stack_ptr_fun_id: None,
        }
    }

    pub fn new_defined_func_idx(&mut self) -> FuncIdx {
        let new_func_idx = self.defined_func_idx_range.1.to_owned();
        let new_max_func_idx = new_func_idx.next_idx();
        self.defined_func_idx_range.1 = new_max_func_idx.to_owned();
        new_func_idx
    }

    pub fn calculate_func_idxs(
        &mut self,
        imported_functions: &Vec<(FunId, String, ReloopedFunction)>,
        defined_functions: &Vec<(FunId, ReloopedFunction)>,
    ) {
        let mut func_idx = FuncIdx::initial_idx();
        // indexes for imported functions come before indexes for functions defined in the program
        let imported_funcs_start_idx = func_idx.to_owned();
        for (imported_fun_id, _, _) in imported_functions {
            self.fun_id_to_func_idx_map
                .insert(imported_fun_id.to_owned(), func_idx.to_owned());
            func_idx = func_idx.next_idx();
        }
        self.imported_func_idx_range = (imported_funcs_start_idx, func_idx.to_owned());

        let defined_funcs_start_idx = func_idx.to_owned();
        for (defined_fun_id, _) in defined_functions {
            self.fun_id_to_func_idx_map
                .insert(defined_fun_id.to_owned(), func_idx.to_owned());
            func_idx = func_idx.next_idx();
        }
        self.defined_func_idx_range = (defined_funcs_start_idx, func_idx.to_owned());
    }
}

pub enum ControlFlowElement {
    Block(LoopBlockId),
    Loop(LoopBlockId),
    If(MultipleBlockId),
    UnlabelledIf,
}

pub struct FunctionContext {
    pub var_fp_offsets: HashMap<VarId, u32>,
    pub global_var_addrs: HashMap<VarId, u32>,
    pub label_variable: VarId,
    pub control_flow_stack: Vec<ControlFlowElement>,
}

impl FunctionContext {
    pub fn new(
        var_fp_offsets: HashMap<VarId, u32>,
        global_var_addrs: HashMap<VarId, u32>,
        label_variable: VarId,
    ) -> Self {
        FunctionContext {
            var_fp_offsets,
            global_var_addrs,
            label_variable,
            control_flow_stack: Vec::new(),
        }
    }

    pub fn global_context(global_var_addrs: HashMap<VarId, u32>) -> Self {
        FunctionContext {
            var_fp_offsets: HashMap::new(),
            global_var_addrs,
            label_variable: VarId::initial_id(), // dummy var, because global instrs don't have any control flow
            control_flow_stack: Vec::new(),
        }
    }

    pub fn get_depth_of_block(&self, loop_block_id: &LoopBlockId) -> Option<u32> {
        if self.control_flow_stack.is_empty() {
            return None;
        }
        let mut i = self.control_flow_stack.len() - 1;
        // top of stack has a depth of zero
        let mut depth = 0;
        loop {
            match self.control_flow_stack.get(i) {
                None => return None,
                Some(ControlFlowElement::Block(this_loop_block_id)) => {
                    if this_loop_block_id == loop_block_id {
                        return Some(depth);
                    }
                }
                _ => {}
            }
            depth += 1;
            // make sure i doesn't underflow
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }

    pub fn get_depth_of_loop(&self, loop_block_id: &LoopBlockId) -> Option<u32> {
        if self.control_flow_stack.is_empty() {
            return None;
        }
        let mut i = self.control_flow_stack.len() - 1;
        // top of stack has a depth of zero
        let mut depth = 0;
        loop {
            match self.control_flow_stack.get(i) {
                None => return None,
                Some(ControlFlowElement::Loop(this_loop_block_id)) => {
                    if this_loop_block_id == loop_block_id {
                        return Some(depth);
                    }
                }
                _ => {}
            }
            depth += 1;
            // make sure i doesn't underflow
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }

    pub fn get_depth_of_if(&self, multiple_block_id: &MultipleBlockId) -> Option<u32> {
        if self.control_flow_stack.is_empty() {
            return None;
        }
        let mut i = self.control_flow_stack.len() - 1;
        // top of stack has a depth of zero
        let mut depth = 0;
        loop {
            match self.control_flow_stack.get(i) {
                None => return None,
                Some(ControlFlowElement::If(this_multiple_block_id)) => {
                    if this_multiple_block_id == multiple_block_id {
                        return Some(depth);
                    }
                }
                _ => {}
            }
            depth += 1;
            // make sure i doesn't underflow
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }
}
