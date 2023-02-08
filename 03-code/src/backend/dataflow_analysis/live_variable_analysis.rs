use std::collections::{HashMap, HashSet};

use crate::backend::dataflow_analysis::flowgraph::Flowgraph;
use crate::backend::dataflow_analysis::instruction_def_ref::{def_set, ref_set};
use crate::middle_end::ids::{InstructionId, VarId};

pub type LiveVariableMap = HashMap<InstructionId, HashSet<VarId>>;

pub fn live_variable_analysis(flowgraph: &Flowgraph) -> LiveVariableMap {
    // for every instr, which vars are live at that point
    let mut live: LiveVariableMap = LiveVariableMap::new();

    let mut changes = true;
    while changes {
        changes = false;

        for (instr_id, instr) in &flowgraph.instrs {
            // U_{s in succ} live(s)
            let mut out_live: HashSet<VarId> = HashSet::new();
            for successor in flowgraph.successors.get(instr_id).unwrap() {
                out_live.extend(live.get(successor).unwrap_or(&HashSet::new()).to_owned());
            }

            // \ def(n)
            for def_var in def_set(instr) {
                out_live.remove(&def_var);
            }

            // U ref(n)
            for ref_var in ref_set(instr) {
                out_live.insert(ref_var);
            }

            let prev_live = live.insert(instr_id.to_owned(), out_live.to_owned());

            match prev_live {
                None => {
                    changes = true;
                }
                Some(prev_live_vars) => {
                    if prev_live_vars != out_live {
                        changes = true
                    }
                }
            }
        }
    }

    live
}
