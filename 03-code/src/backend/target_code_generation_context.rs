use crate::backend::wasm_indices::FuncIdx;
use crate::middle_end::ids::{FunId, VarId};
use crate::relooper::blocks::{LoopBlockId, MultipleBlockId};
use std::collections::HashMap;

pub struct ModuleContext {
    pub func_idx_mappings: HashMap<FunId, FuncIdx>, // todo we need to calculate these before we convert the instrs
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
