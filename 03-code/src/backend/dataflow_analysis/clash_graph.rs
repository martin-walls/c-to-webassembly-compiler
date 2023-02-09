use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;

use crate::backend::dataflow_analysis::flowgraph::{generate_flowgraph, Flowgraph};
use crate::backend::dataflow_analysis::live_variable_analysis::live_variable_analysis;
use crate::middle_end::ids::VarId;
use crate::middle_end::instructions::Instruction;
use crate::relooper::blocks::Block;

#[derive(Clone)]
pub struct ClashGraph {
    pub clashes: HashMap<VarId, HashSet<VarId>>,
    universal_clashes: HashSet<VarId>,
}

impl ClashGraph {
    fn new() -> Self {
        ClashGraph {
            clashes: HashMap::new(),
            universal_clashes: HashSet::new(),
        }
    }

    fn add_var(&mut self, var: VarId) {
        if !self.clashes.contains_key(&var) {
            self.clashes.insert(var, HashSet::new());
        }
    }

    pub fn remove_var(&mut self, var: &VarId) {
        self.clashes.remove(var);
        self.universal_clashes.remove(var);
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

    fn add_universal_clash(&mut self, var: VarId) {
        self.universal_clashes.insert(var);
    }

    pub fn count_clashes(&self, var: &VarId) -> usize {
        if self.universal_clashes.contains(var) {
            return usize::MAX;
        }
        match self.clashes.get(var) {
            Some(clashes) => clashes.len(),
            None => {
                // if var isn't in clash graph, it has no clashes to other vars
                0 as usize
            }
        }
    }

    pub fn do_vars_clash(&self, var1: &VarId, var2: &VarId) -> bool {
        if self.universal_clashes.contains(var1) || self.universal_clashes.contains(var2) {
            return true;
        }
        // the clash graph is symmetric, so we only need to check in one direction
        match self.clashes.get(var1) {
            None => {
                // if var isn't in clash graph, it has no clashes to other vars
                true
            }
            Some(clashes) => clashes.contains(var2),
        }
    }
}

impl fmt::Display for ClashGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Clash graph:")?;
        for (var, clash_vars) in &self.clashes {
            write!(f, "{}: ", var)?;
            for clash_var in clash_vars {
                write!(f, "{}, ", clash_var)?;
            }
            writeln!(f)?;
        }
        write!(f, "")
    }
}

pub fn generate_clash_graph(block: &Box<Block>) -> ClashGraph {
    let flowgraph = generate_flowgraph(block);
    let live_vars = live_variable_analysis(&flowgraph);

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

    // any var that we take address of should clash with everything else,
    // to ensure safety of analysis
    insert_universal_clashes_for_address_taken_vars(&mut clash_graph, &flowgraph);

    clash_graph
}

fn insert_universal_clashes_for_address_taken_vars(
    clash_graph: &mut ClashGraph,
    flowgraph: &Flowgraph,
) {
    for (_id, instr) in &flowgraph.instrs {
        if let Instruction::AddressOf(_, _, src) = instr {
            clash_graph.add_universal_clash(src.unwrap_var().unwrap());
        }
    }
}
