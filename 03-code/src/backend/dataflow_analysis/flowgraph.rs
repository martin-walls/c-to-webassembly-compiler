use crate::middle_end::ids::{Id, IdGenerator};
use crate::middle_end::instructions::Instruction;
use crate::relooper::blocks::Block;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstructionId(u64);

impl Id for InstructionId {
    fn initial_id() -> Self {
        InstructionId(0)
    }

    fn next_id(&self) -> Self {
        InstructionId(self.0 + 1)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

pub struct Flowgraph {
    instrs: HashMap<InstructionId, Instruction>,
    // adjacency lists
    successors: HashMap<InstructionId, HashSet<InstructionId>>,
    predecessors: HashMap<InstructionId, HashSet<InstructionId>>,
    entries: HashSet<InstructionId>,
    exits: HashSet<InstructionId>,
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

    fn add_instr(&mut self, instr_id: InstructionId, instr: Instruction) {
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
}

pub fn generate_flowgraph(relooper_block: &Box<Block>) -> Flowgraph {
    let mut instr_id_generator = IdGenerator::<InstructionId>::new();

    let mut flowgraph = Flowgraph::new();

    let (flowgraph_entries, flowgraph_exits) = add_block_to_flowgraph_and_get_entries_and_exits(
        relooper_block,
        &mut flowgraph,
        &mut instr_id_generator,
    );

    flowgraph.entries = flowgraph_entries;
    flowgraph.exits = flowgraph_exits;

    flowgraph
}

fn add_block_to_flowgraph_and_get_entries_and_exits(
    block: &Box<Block>,
    flowgraph: &mut Flowgraph,
    instr_id_generator: &mut IdGenerator<InstructionId>,
) -> (HashSet<InstructionId>, HashSet<InstructionId>) {
    match &**block {
        Block::Simple { internal, next } => {
            if internal.instrs.is_empty() {
                // shouldn't really ever be the case
                if let Some(next) = next {
                    // just skip this block if it happens to have no instructions
                    return add_block_to_flowgraph_and_get_entries_and_exits(
                        next,
                        flowgraph,
                        instr_id_generator,
                    );
                }
                return (HashSet::new(), HashSet::new());
            }

            let mut block_entry_instrs = HashSet::new();
            // dummy initialiser value, to make the compiler happy
            let mut last_instr_id = InstructionId(0);
            let mut prev_instr_id = None;
            for i in 0..internal.instrs.len() {
                let instr_id = instr_id_generator.new_id();
                let instr = internal.instrs.get(i).unwrap();
                flowgraph.add_instr(instr_id.to_owned(), instr.to_owned());

                // instrs are successive
                if let Some(prev_instr_id) = prev_instr_id {
                    flowgraph.add_successor(prev_instr_id, instr_id.to_owned());
                }

                if i == 0 {
                    block_entry_instrs.insert(instr_id.to_owned());
                } else if i == internal.instrs.len() - 1 {
                    last_instr_id = instr_id.to_owned();
                }

                prev_instr_id = Some(instr_id);
            }

            // TODO handle break/continue/endhandled instrs

            if let Some(next) = next {
                let (successors, block_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(
                        next,
                        flowgraph,
                        instr_id_generator,
                    );

                for successor_id in successors {
                    flowgraph.add_successor(last_instr_id.to_owned(), successor_id);
                }

                (block_entry_instrs, block_exit_instrs)
            } else {
                let mut block_exit_instrs = HashSet::new();
                block_exit_instrs.insert(last_instr_id);
                (block_entry_instrs, block_exit_instrs)
            }
        }
        Block::Loop { id: _, inner, next } => {
            let (inner_entry_instrs, inner_exit_instrs) =
                add_block_to_flowgraph_and_get_entries_and_exits(
                    inner,
                    flowgraph,
                    instr_id_generator,
                );

            // make start of loop successor of end of loop
            for start_of_loop_instr in &inner_entry_instrs {
                for end_of_loop_instr in &inner_exit_instrs {
                    flowgraph.add_successor(
                        end_of_loop_instr.to_owned(),
                        start_of_loop_instr.to_owned(),
                    );
                }
            }

            if let Some(next) = next {
                let (next_entry_instrs, next_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(
                        next,
                        flowgraph,
                        instr_id_generator,
                    );

                for successor in next_entry_instrs {
                    for end_of_loop_instr in &inner_exit_instrs {
                        flowgraph.add_successor(end_of_loop_instr.to_owned(), successor.to_owned());
                    }
                }

                (inner_entry_instrs, next_exit_instrs)
            } else {
                (inner_entry_instrs, inner_exit_instrs)
            }
        }
        Block::Multiple {
            id: _,
            handled_blocks,
            next,
        } => {
            // combine entries and exits from all handled blocks
            let mut entry_instrs = HashSet::new();
            let mut all_handled_exit_instrs = HashSet::new();

            for handled_block in handled_blocks {
                let (handled_entry_instrs, handled_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(
                        handled_block,
                        flowgraph,
                        instr_id_generator,
                    );

                entry_instrs.extend(handled_entry_instrs);
                all_handled_exit_instrs.extend(handled_exit_instrs);
            }

            if let Some(next) = next {
                let (next_entry_instrs, next_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(
                        next,
                        flowgraph,
                        instr_id_generator,
                    );

                for successor in next_entry_instrs {
                    for end_of_handled_instr in &all_handled_exit_instrs {
                        flowgraph
                            .add_successor(end_of_handled_instr.to_owned(), successor.to_owned());
                    }
                }

                (entry_instrs, next_exit_instrs)
            } else {
                (entry_instrs, all_handled_exit_instrs)
            }
        }
    }
}
