use std::collections::HashMap;

use log::debug;

use crate::backend::dataflow_analysis::flowgraph::{generate_flowgraph, Flowgraph};
use crate::backend::dataflow_analysis::instruction_def_ref::def_set;
use crate::backend::dataflow_analysis::live_variable_analysis::live_variable_analysis;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::ProgramMetadata;
use crate::relooper::blocks::Block;

pub fn remove_dead_vars(
    block: &mut Box<Block>,
    prog_metadata: &mut Box<ProgramMetadata>,
) -> Flowgraph {
    let mut flowgraph = generate_flowgraph(block);

    let mut changes = true;
    while changes {
        changes = false;

        let live_vars = live_variable_analysis(&flowgraph);

        let mut remove_instrs = Vec::new();
        let mut replace_instrs = HashMap::new();

        for (instr_id, instr) in &flowgraph.instrs {
            let defs = def_set(instr);

            if defs.is_empty() {
                continue;
            }

            let mut at_least_one_def_is_live = false;
            for def_var in defs {
                // if none of the defined vars are live at any of the successors of this instr,
                // the instr is dead

                for successor in flowgraph.successors.get(instr_id).unwrap() {
                    let is_live_at_successor = live_vars.get(successor).unwrap().contains(&def_var);
                    if is_live_at_successor {
                        at_least_one_def_is_live = true;
                        break;
                    }
                }

                // we can stop once we've found one
                if at_least_one_def_is_live {
                    break;
                }
            }

            if at_least_one_def_is_live {
                continue;
            }

            // this instr's result is dead

            // if no side effects, remove instr
            if !instr.has_side_effect() {
                remove_instrs.push(instr_id.to_owned());
            } else {
                // do the instr, but don't assign to dest
                match instr {
                    Instruction::Call(id, _dest, fun_id, params) => {
                        let new_instr = Instruction::Call(
                            id.to_owned(),
                            prog_metadata.init_null_dest_var(),
                            fun_id.to_owned(),
                            params.to_owned(),
                        );
                        replace_instrs.insert(instr_id.to_owned(), new_instr);
                        debug!("replacing call instr");
                    }
                    _ => unreachable!(),
                }
            }
        }

        for instr_id in remove_instrs {
            flowgraph.remove_instr(&instr_id);
            block.remove_instr(&instr_id);
        }
        for (instr_id, new_instr) in replace_instrs {
            block.replace_instr(&instr_id, new_instr.to_owned());
            // overwrite the existing instr
            flowgraph.instrs.insert(instr_id, new_instr);
        }
    }

    flowgraph
}
