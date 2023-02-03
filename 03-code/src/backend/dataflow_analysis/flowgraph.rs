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

            let (block_entry_instr, mut block_exit_instrs, jump_exit_instrs) =
                add_instrs_to_flowgraph(&internal.instrs, flowgraph, instr_id_generator);

            block_exit_instrs.extend(jump_exit_instrs);

            let mut block_entry_instrs = HashSet::new();
            block_entry_instrs.insert(block_entry_instr);

            if let Some(next) = next {
                let (successors, next_exit_instrs) =
                    add_block_to_flowgraph_and_get_entries_and_exits(
                        next,
                        flowgraph,
                        instr_id_generator,
                    );

                for successor in successors {
                    for end_of_block_instr in &block_exit_instrs {
                        flowgraph
                            .add_successor(end_of_block_instr.to_owned(), successor.to_owned());
                    }
                }

                (block_entry_instrs, next_exit_instrs)
            } else {
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

                // could skip handled blocks and go straight to next block
                entry_instrs.extend(next_entry_instrs.to_owned());

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

fn add_instrs_to_flowgraph(
    instrs: &Vec<Instruction>,
    flowgraph: &mut Flowgraph,
    instr_id_generator: &mut IdGenerator<InstructionId>,
) -> (
    InstructionId,
    HashSet<InstructionId>,
    HashSet<InstructionId>,
) {
    // dummy value that'll get overwritten
    let mut entry_instr = InstructionId(0);
    let mut exit_instrs = HashSet::new();
    let mut block_exit_instrs = HashSet::new();

    let mut prev_instrs: HashSet<InstructionId> = HashSet::new();
    for i in 0..instrs.len() {
        let instr_id = instr_id_generator.new_id();
        let instr = instrs.get(i).unwrap();
        flowgraph.add_instr(instr_id.to_owned(), instr.to_owned());

        match instr {
            Instruction::Break(_)
            | Instruction::Continue(_)
            | Instruction::EndHandledBlock(_)
            | Instruction::Ret(_)
            | Instruction::TailCall(_, _) => {
                // successive from previous instr
                for prev_instr_id in &prev_instrs {
                    flowgraph.add_successor(prev_instr_id.to_owned(), instr_id.to_owned());
                }
                // these instrs jump out of this block
                block_exit_instrs.insert(instr_id.to_owned());

                // next instr isn't a successor
                prev_instrs.clear();

                if i == 0 {
                    entry_instr = instr_id.to_owned();
                }
                if i == instrs.len() - 1 {
                    exit_instrs.insert(instr_id.to_owned());
                }
            }
            Instruction::Br(_) | Instruction::BrIfEq(_, _, _) | Instruction::BrIfNotEq(_, _, _) => {
                unreachable!("Relooper algorithm removes all unstructured branch instrs")
            }
            Instruction::IfEqElse(_, _, instrs1, instrs2)
            | Instruction::IfNotEqElse(_, _, instrs1, instrs2) => {
                // successive from previous instr
                for prev_instr_id in &prev_instrs {
                    flowgraph.add_successor(prev_instr_id.to_owned(), instr_id.to_owned());
                }

                let (instrs1_entry, instrs1_exits, instrs1_block_exits) =
                    add_instrs_to_flowgraph(&instrs1, flowgraph, instr_id_generator);
                let (instrs2_entry, instrs2_exits, instrs2_block_exits) =
                    add_instrs_to_flowgraph(&instrs2, flowgraph, instr_id_generator);

                flowgraph.add_successor(instr_id.to_owned(), instrs1_entry);
                flowgraph.add_successor(instr_id.to_owned(), instrs2_entry);

                prev_instrs.clear();
                prev_instrs.extend(instrs1_exits.to_owned());
                prev_instrs.extend(instrs2_exits.to_owned());

                block_exit_instrs.extend(instrs1_block_exits);
                block_exit_instrs.extend(instrs2_block_exits);

                if i == 0 {
                    entry_instr = instr_id.to_owned();
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
                    entry_instr = instr_id.to_owned();
                }
                if i == instrs.len() - 1 {
                    exit_instrs.insert(instr_id.to_owned());
                }
            }
        }
    }

    (entry_instr, exit_instrs, block_exit_instrs)
}
