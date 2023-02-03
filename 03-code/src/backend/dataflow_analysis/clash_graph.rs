use crate::backend::dataflow_analysis::live_variable_analysis::LiveVariableMap;
use crate::middle_end::ids::VarId;
use crate::middle_end::ir::ProgramMetadata;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
pub struct ClashGraph {
    pub clashes: HashMap<VarId, HashSet<VarId>>,
}

impl ClashGraph {
    fn new() -> Self {
        ClashGraph {
            clashes: HashMap::new(),
        }
    }

    fn add_var(&mut self, var: VarId) {
        if !self.clashes.contains_key(&var) {
            self.clashes.insert(var, HashSet::new());
        }
    }

    pub fn remove_var(&mut self, var: &VarId) {
        self.clashes.remove(var);
        for (var, clashes) in &mut self.clashes {
            clashes.remove(var);
        }
    }

    fn add_clash(&mut self, var1: VarId, var2: VarId) {
        self.add_var(var1.to_owned());
        self.clashes.get_mut(&var1).unwrap().insert(var2.to_owned());

        self.add_var(var2.to_owned());
        self.clashes.get_mut(&var2).unwrap().insert(var1);
    }

    pub fn count_clashes(&self, var: &VarId) -> usize {
        match self.clashes.get(var) {
            Some(clashes) => clashes.len(),
            None => {
                // if var isn't in clash graph, it has no clashes to other vars
                0 as usize
            }
        }
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
            for var in simultaneously_live_vars {
                clash_graph.add_var(var.to_owned());
            }
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
