use crate::backend::dataflow_analysis::live_variable_analysis::LiveVariableMap;
use crate::middle_end::ids::VarId;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;

pub struct ClashGraph {
    clashes: HashMap<VarId, HashSet<VarId>>,
}

impl ClashGraph {
    fn new() -> Self {
        ClashGraph {
            clashes: HashMap::new(),
        }
    }

    fn add_clash(&mut self, var1: VarId, var2: VarId) {
        if !self.clashes.contains_key(&var1) {
            self.clashes.insert(var1.to_owned(), HashSet::new());
        }
        self.clashes.get_mut(&var1).unwrap().insert(var2.to_owned());

        if !self.clashes.contains_key(&var2) {
            self.clashes.insert(var2.to_owned(), HashSet::new());
        }
        self.clashes.get_mut(&var2).unwrap().insert(var1);
    }
}

impl fmt::Display for ClashGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Clash graph:\n")?;
        for (var, clash_vars) in &self.clashes {
            write!(f, "{}: ", var)?;
            for clash_var in clash_vars {
                write!(f, "{}, ", clash_var)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}

pub fn generate_clash_graph(live_vars: &LiveVariableMap) -> ClashGraph {
    let mut clash_graph = ClashGraph::new();

    for (_instr, simultaneously_live_vars) in live_vars {
        if simultaneously_live_vars.len() < 2 {
            // no clashes
            continue;
        }

        let mut simultaneously_live_vars_vec: Vec<&VarId> =
            simultaneously_live_vars.iter().collect();

        // iteratively pop one of the vars, and store its clash with all the other remaining vars
        while let Some(var) = simultaneously_live_vars_vec.pop() {
            for other_var in &simultaneously_live_vars_vec {
                clash_graph.add_clash(var.to_owned(), (*other_var).to_owned());
            }
        }
    }

    clash_graph
}
