use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

use log::debug;

use crate::middle_end::ids::FunId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::Program;
use crate::middle_end::middle_end_error::MiddleEndError;

pub fn remove_unused_functions(prog: &mut Program) -> Result<(), MiddleEndError> {
    let call_graph = generate_call_graph(prog)?;

    // do call graph analysis to find which functions are never called
    let mut unused_fun_ids = HashSet::new();
    for fun_id in prog.program_instructions.functions.keys() {
        unused_fun_ids.insert(fun_id.to_owned());
    }

    for entry_fun_id in call_graph.entries {
        walk_graph(&call_graph.graph, entry_fun_id, |fun_id| {
            unused_fun_ids.remove(fun_id);
        });
    }

    // remove the unused functions
    debug!("removing unused fun ids: {:?}", unused_fun_ids);

    for unused_fun_id in &unused_fun_ids {
        prog.program_instructions.functions.remove(unused_fun_id);
        prog.program_metadata.function_types.remove(unused_fun_id);
        prog.program_metadata
            .function_param_var_mappings
            .remove(unused_fun_id);
        prog.program_metadata
            .function_ids
            .retain(|_name, fun_id| fun_id != unused_fun_id);
    }

    Ok(())
}

struct CallGraph {
    /// Adjacency list representation of the call graph
    graph: HashMap<FunId, HashSet<FunId>>,
    /// Set of entry function(s)
    entries: HashSet<FunId>,
}

impl CallGraph {
    fn new() -> Self {
        CallGraph {
            graph: HashMap::new(),
            entries: HashSet::new(),
        }
    }
}

fn generate_call_graph(prog: &Program) -> Result<CallGraph, MiddleEndError> {
    let mut call_graph = CallGraph::new();

    for (fun_id, function) in &prog.program_instructions.functions {
        let mut callee_fun_ids = HashSet::new();

        for instr in &function.instrs {
            match instr {
                Instruction::Call(_, _, callee_fun_id, _)
                | Instruction::TailCall(_, callee_fun_id, _) => {
                    callee_fun_ids.insert(callee_fun_id.to_owned());
                }
                _ => {}
            }
        }

        call_graph.graph.insert(fun_id.to_owned(), callee_fun_ids);
    }

    // add any functions called globally to entries
    for instr in &prog.program_instructions.global_instrs {
        match instr {
            Instruction::Call(_, _, callee_fun_id, _)
            | Instruction::TailCall(_, callee_fun_id, _) => {
                call_graph.entries.insert(callee_fun_id.to_owned());
            }
            _ => {}
        }
    }

    // set main() as an entry
    let main_fun_id = prog.program_metadata.get_main_fun_id()?;
    call_graph.entries.insert(main_fun_id);

    Ok(call_graph)
}

fn walk_graph<T: Eq + Hash + Display, F>(
    adjacency_list: &HashMap<T, HashSet<T>>,
    entry: T,
    mut f: F,
) where
    F: FnMut(&T),
{
    // stack of nodes we've seen but not yet explored
    let mut to_explore = vec![&entry];
    // set of all the nodes we've seen (ie. that have been added to to_explore at some point)
    let mut seen = HashSet::new();
    seen.insert(&entry);

    while !to_explore.is_empty() {
        // pop top node from stack
        let node = to_explore.pop().unwrap();
        // get children of node, and add those we haven't yet seen to to_explore
        let children = adjacency_list.get(node).unwrap();
        for child in children {
            if !seen.contains(&child) {
                to_explore.push(child);
                // mark node as seen
                seen.insert(child);
            }
        }

        // call f for this node
        f(node);
    }
}
