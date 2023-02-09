use std::collections::{HashMap, HashSet};

use crate::middle_end::ids::InstructionId;
use crate::middle_end::instructions::Instruction;
use crate::relooper::blocks::Block;

#[derive(Debug)]
pub struct Flowgraph {
    pub instrs: HashMap<InstructionId, Instruction>,
    // adjacency lists
    pub successors: HashMap<InstructionId, HashSet<InstructionId>>,
    pub predecessors: HashMap<InstructionId, HashSet<InstructionId>>,
    pub entries: HashSet<InstructionId>,
    pub exits: HashSet<InstructionId>,
}

impl Flowgraph {
    fn new() -> Self {
        Flowgraph {
            instrs: HashMap::new(),
            successors: HashMap::new(),
            predecessors: HashMap::new(),
            entries: HashSet::new(),
            exits: HashSet::new(),
        }
    }

    fn add_instr(&mut self, instr: Instruction) {
        let instr_id = instr.get_instr_id();
        self.instrs.insert(instr_id.to_owned(), instr);
        // init adjacency lists
        self.successors.insert(instr_id.to_owned(), HashSet::new());
        self.predecessors.insert(instr_id, HashSet::new());
    }

    fn add_successor(&mut self, instr_id: InstructionId, successor_id: InstructionId) {
        // make successor_id a successor of instr_id
        self.successors
            .get_mut(&instr_id)
            .unwrap()
            .insert(successor_id.to_owned());
        // make instr_id a predecessor of successor_id
        self.predecessors
            .get_mut(&successor_id)
            .unwrap()
            .insert(instr_id);
    }

    pub fn remove_instr(&mut self, instr_id: &InstructionId) {
        // if instr not in flowgraph, nothing to do
        if !self.instrs.contains_key(instr_id) {
            return;
        }
        let mut successors = self.successors.remove(instr_id).unwrap();
        let mut predecessors = self.predecessors.remove(instr_id).unwrap();
        // just in case
        successors.remove(instr_id);
        predecessors.remove(instr_id);

        self.instrs.remove(instr_id);
        self.successors.remove(instr_id);
        self.predecessors.remove(instr_id);

        for successor_id in self.successors.keys() {
            self.predecessors
                .insert(successor_id.to_owned(), predecessors.to_owned());
        }
        for predecessor_id in self.predecessors.keys() {
            self.successors
                .insert(predecessor_id.to_owned(), successors.to_owned());
        }

        if self.entries.contains(instr_id) {
            self.entries.remove(instr_id);
            self.entries.extend(successors);
        }
        if self.exits.contains(instr_id) {
            self.exits.remove(instr_id);
            self.exits.extend(predecessors);
        }
    }
}

pub fn generate_flowgraph(relooper_block: &Box<Block>) -> Flowgraph {
    let mut flowgraph = Flowgraph::new();

    let (flowgraph_entries, flowgraph_exits, flowgraph_jump_exits) =
        add_block_to_flowgraph_and_get_entries_and_exits(relooper_block, &mut flowgraph);

    flowgraph.entries = flowgraph_entries;
    flowgraph.exits = flowgraph_exits;
    flowgraph.exits.extend(flowgraph_jump_exits);

    flowgraph
}

fn add_block_to_flowgraph_and_get_entries_and_exits(
    block: &Box<Block>,
    flowgraph: &mut Flowgraph,
) -> (
    HashSet<InstructionId>,
    HashSet<InstructionId>,
    HashSet<InstructionId>,
) {
    match &**block {
        Block::Simple { internal, next } => {
            if internal.instrs.is_empty() {
                // shouldn't really ever be the case
                if let Some(next) = next {
                    // just skip this block if it happens to have no instructions
                    return add_block_to_flowgraph_and_get_entries_and_exits(next, flowgraph);
                }
                return (HashSet::new(), HashSet::new(), HashSet::new());
            }

            let (block_entry_instr, mut block_exit_instrs, mut jump_exit_instrs) =
                add_instrs_to_flowgraph(&internal.instrs, flowgraph);

            block_exit_instrs.extend(jump_exit_instrs.to_owned());

            let mut block_entry_instrs = HashSet::new();
            // we've checked that instrs is non-empty, so we can unwrap safely
            block_entry_instrs.insert(block_entry_instr.unwrap());

            if let Some(next) = next {
                let (successors, next_exit_instrs, next_jump_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(next, flowgraph);

                jump_exit_instrs.extend(next_jump_exit_instrs);

                for successor in successors {
                    for end_of_block_instr in &block_exit_instrs {
                        flowgraph
                            .add_successor(end_of_block_instr.to_owned(), successor.to_owned());
                    }
                }

                (block_entry_instrs, next_exit_instrs, jump_exit_instrs)
            } else {
                (block_entry_instrs, block_exit_instrs, jump_exit_instrs)
            }
        }
        Block::Loop { id: _, inner, next } => {
            let (inner_entry_instrs, inner_exit_instrs, mut inner_jump_exit_instrs) =
                add_block_to_flowgraph_and_get_entries_and_exits(inner, flowgraph);

            // make start of loop successor of end of loop
            for start_of_loop_instr in &inner_entry_instrs {
                for end_of_loop_instr in &inner_exit_instrs {
                    flowgraph.add_successor(
                        end_of_loop_instr.to_owned(),
                        start_of_loop_instr.to_owned(),
                    );
                }

                // might be a `continue` jumping back to the start of the loop
                for jump_exit_instr in &inner_jump_exit_instrs {
                    flowgraph
                        .add_successor(jump_exit_instr.to_owned(), start_of_loop_instr.to_owned());
                }
            }

            if let Some(next) = next {
                let (next_entry_instrs, next_exit_instrs, next_jump_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(next, flowgraph);

                for successor in next_entry_instrs {
                    for end_of_loop_instr in &inner_exit_instrs {
                        flowgraph.add_successor(end_of_loop_instr.to_owned(), successor.to_owned());
                    }

                    // could be a `break` jump
                    for jump_exit_instr in &inner_jump_exit_instrs {
                        flowgraph.add_successor(jump_exit_instr.to_owned(), successor.to_owned());
                    }
                }

                inner_jump_exit_instrs.extend(next_jump_exit_instrs);

                (inner_entry_instrs, next_exit_instrs, inner_jump_exit_instrs)
            } else {
                (
                    inner_entry_instrs,
                    inner_exit_instrs,
                    inner_jump_exit_instrs,
                )
            }
        }
        Block::Multiple {
            id: _,
            pre_handled_blocks_instrs,
            handled_blocks,
            next,
        } => {
            // combine entries and exits from all handled blocks
            let mut entry_instrs = HashSet::new();
            let mut all_handled_exit_instrs = HashSet::new();
            let mut all_handled_jump_exit_instrs = HashSet::new();

            let (pre_handled_entry, pre_handled_exits, _jump_exits) =
                add_instrs_to_flowgraph(pre_handled_blocks_instrs, flowgraph);

            if let Some(entry) = pre_handled_entry {
                entry_instrs.insert(entry);
            }

            // let entry_instrs = pre_handled_blocks_instrs
            //     .iter()
            //     .map(|i| i.get_instr_id())
            //     .collect::<HashSet<InstructionId>>();

            for handled_block in handled_blocks {
                let (handled_entry_instrs, handled_exit_instrs, handled_jump_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(handled_block, flowgraph);

                if pre_handled_exits.is_empty() {
                    entry_instrs.extend(handled_entry_instrs);
                } else {
                    for pre_handled in &pre_handled_exits {
                        for handled_entry in &handled_entry_instrs {
                            flowgraph
                                .add_successor(pre_handled.to_owned(), handled_entry.to_owned());
                        }
                    }
                }

                all_handled_exit_instrs.extend(handled_exit_instrs);
                all_handled_jump_exit_instrs.extend(handled_jump_exit_instrs);
            }

            if let Some(next) = next {
                let (next_entry_instrs, next_exit_instrs, next_jump_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(next, flowgraph);

                // could skip handled blocks and go straight to next block
                if pre_handled_exits.is_empty() {
                    entry_instrs.extend(next_entry_instrs.to_owned());
                } else {
                    for pre_handled in &pre_handled_exits {
                        for next_entry in &next_entry_instrs {
                            flowgraph.add_successor(pre_handled.to_owned(), next_entry.to_owned());
                        }
                    }
                }

                for successor in next_entry_instrs {
                    for end_of_handled_instr in &all_handled_exit_instrs {
                        flowgraph
                            .add_successor(end_of_handled_instr.to_owned(), successor.to_owned());
                    }

                    // could be an `endHandled` jump
                    for jump_exit_instr in &all_handled_jump_exit_instrs {
                        flowgraph.add_successor(jump_exit_instr.to_owned(), successor.to_owned())
                    }
                }

                all_handled_jump_exit_instrs.extend(next_jump_exit_instrs);

                (entry_instrs, next_exit_instrs, all_handled_jump_exit_instrs)
            } else {
                (
                    entry_instrs,
                    all_handled_exit_instrs,
                    all_handled_jump_exit_instrs,
                )
            }
        }
    }
}

fn add_instrs_to_flowgraph(
    instrs: &Vec<Instruction>,
    flowgraph: &mut Flowgraph,
) -> (
    Option<InstructionId>, // can be None eg. if this is an empty else block
    HashSet<InstructionId>,
    HashSet<InstructionId>,
) {
    let mut entry_instr = None;
    let mut exit_instrs = HashSet::new();
    let mut jump_exit_instrs = HashSet::new();

    let mut prev_instrs: HashSet<InstructionId> = HashSet::new();
    for i in 0..instrs.len() {
        let instr = instrs.get(i).unwrap();
        let instr_id = instr.get_instr_id();
        flowgraph.add_instr(instr.to_owned());

        match instr {
            Instruction::Break(..)
            | Instruction::Continue(..)
            | Instruction::EndHandledBlock(..)
            | Instruction::Ret(..)
            | Instruction::TailCall(..) => {
                // successive from previous instr
                for prev_instr_id in &prev_instrs {
                    flowgraph.add_successor(prev_instr_id.to_owned(), instr_id.to_owned());
                }
                // these instrs jump out of this block
                jump_exit_instrs.insert(instr_id.to_owned());

                // next instr isn't a successor
                prev_instrs.clear();

                if i == 0 {
                    entry_instr = Some(instr_id.to_owned());
                }
                if i == instrs.len() - 1 {
                    exit_instrs.insert(instr_id.to_owned());
                }
            }
            Instruction::Br(..) | Instruction::BrIfEq(..) | Instruction::BrIfNotEq(..) => {
                unreachable!("Relooper algorithm removes all unstructured branch instrs")
            }
            Instruction::IfEqElse(_, _, _, instrs1, instrs2)
            | Instruction::IfNotEqElse(_, _, _, instrs1, instrs2) => {
                // successive from previous instr
                for prev_instr_id in &prev_instrs {
                    flowgraph.add_successor(prev_instr_id.to_owned(), instr_id.to_owned());
                }

                let (instrs1_entry, instrs1_exits, instrs1_jump_exits) =
                    add_instrs_to_flowgraph(instrs1, flowgraph);
                let (instrs2_entry, instrs2_exits, instrs2_jump_exits) =
                    add_instrs_to_flowgraph(instrs2, flowgraph);

                if let Some(entry) = instrs1_entry {
                    flowgraph.add_successor(instr_id.to_owned(), entry);
                }
                if let Some(entry) = instrs2_entry {
                    flowgraph.add_successor(instr_id.to_owned(), entry);
                }

                prev_instrs.clear();
                prev_instrs.extend(instrs1_exits.to_owned());
                prev_instrs.extend(instrs2_exits.to_owned());

                jump_exit_instrs.extend(instrs1_jump_exits);
                jump_exit_instrs.extend(instrs2_jump_exits);

                if i == 0 {
                    entry_instr = Some(instr_id.to_owned());
                }
                if i == instrs.len() - 1 {
                    exit_instrs.extend(instrs1_exits);
                    exit_instrs.extend(instrs2_exits);
                }
            }
            _ => {
                // all other instrs are successive
                for prev_instr_id in &prev_instrs {
                    flowgraph.add_successor(prev_instr_id.to_owned(), instr_id.to_owned());
                }
                prev_instrs.clear();
                prev_instrs.insert(instr_id.to_owned());

                if i == 0 {
                    entry_instr = Some(instr_id.to_owned());
                }
                if i == instrs.len() - 1 {
                    exit_instrs.insert(instr_id.to_owned());
                }
            }
        }
    }

    (entry_instr, exit_instrs, jump_exit_instrs)
}
