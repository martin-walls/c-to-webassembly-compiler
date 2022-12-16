use crate::backend::wasm_indices::{FuncIdx, WasmIdx};
use crate::middle_end::ids::{FunId, VarId};
use crate::relooper::blocks::{LoopBlockId, MultipleBlockId};
use crate::relooper::relooper::ReloopedFunction;
use std::collections::HashMap;

pub struct ModuleContext {
    pub fun_id_to_func_idx_map: HashMap<FunId, FuncIdx>,
    pub func_idx_to_fun_id_map: HashMap<FuncIdx, FunId>,
}

impl ModuleContext {
    pub fn new() -> Self {
        ModuleContext {
            fun_id_to_func_idx_map: HashMap::new(), // todo we need to calculate these before we convert the instrs
            func_idx_to_fun_id_map: HashMap::new(),
        }
    }

    pub fn calculate_func_idxs(
        &mut self,
        imported_functions: &Vec<(FunId, String, ReloopedFunction)>,
        defined_functions: &Vec<(FunId, ReloopedFunction)>,
    ) {
        let mut func_idx = FuncIdx::initial_idx();
        // indexes for imported functions come before indexes for functions defined in the program
        for (imported_fun_id, _, _) in imported_functions {
            self.fun_id_to_func_idx_map
                .insert(imported_fun_id.to_owned(), func_idx.to_owned());
            self.func_idx_to_fun_id_map
                .insert(func_idx.to_owned(), imported_fun_id.to_owned());
            func_idx = func_idx.next_idx();
        }

        for (defined_fun_id, _) in defined_functions {
            self.fun_id_to_func_idx_map
                .insert(defined_fun_id.to_owned(), func_idx.to_owned());
            self.func_idx_to_fun_id_map
                .insert(func_idx.to_owned(), defined_fun_id.to_owned());
            func_idx = func_idx.next_idx();
        }
    }
}

pub enum ControlFlowElement {
    Block(LoopBlockId),
    Loop(LoopBlockId),
    If(MultipleBlockId),
    UnlabelledIf, // todo this one might not be necessary
}

pub struct FunctionContext {
    pub var_fp_offsets: HashMap<VarId, u32>,
    pub label_variable: VarId,
    pub control_flow_stack: Vec<ControlFlowElement>,
}

impl FunctionContext {
    pub fn new(var_fp_offsets: HashMap<VarId, u32>, label_variable: VarId) -> Self {
        FunctionContext {
            var_fp_offsets,
            label_variable,
            control_flow_stack: Vec::new(),
        }
    }
}
