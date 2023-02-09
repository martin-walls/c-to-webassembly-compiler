use std::collections::{HashMap, HashSet};

use log::{error, info};

use crate::middle_end::ids::{FunId, Id, IdGenerator, LabelId, ValueType, VarId};
use crate::middle_end::instructions::{Constant, Instruction, Src};
use crate::middle_end::ir::{Program, ProgramMetadata};
use crate::middle_end::ir_types::IrType;
use crate::relooper::blocks::{Block, Label, LoopBlockId, MultipleBlockId};
use crate::relooper::soupify::soupify;

pub type Labels = HashMap<LabelId, Label>;
type Entries = Vec<LabelId>;
type ReachabilityMap = HashMap<LabelId, Vec<LabelId>>;

struct RelooperContext<'a> {
    loop_block_id_generator: &'a mut IdGenerator<LoopBlockId>,
    multiple_block_id_generator: &'a mut IdGenerator<MultipleBlockId>,
    label_variable: &'a VarId,
}

impl<'a> RelooperContext<'a> {
    pub fn new(
        loop_block_id_generator: &'a mut IdGenerator<LoopBlockId>,
        multiple_block_id_generator: &'a mut IdGenerator<MultipleBlockId>,
        label_variable: &'a VarId,
    ) -> Self {
        RelooperContext {
            loop_block_id_generator,
            multiple_block_id_generator,
            label_variable,
        }
    }
}

#[derive(Debug)]
pub struct ReloopedFunction {
    pub block: Option<Box<Block>>,
    pub label_variable: Option<VarId>,
    // only None if block is None
    pub type_info: Box<IrType>,
    pub param_var_mappings: Vec<VarId>,
    pub body_is_defined: bool,
}

pub struct ProgramBlocks {
    pub functions: HashMap<FunId, ReloopedFunction>,
    pub global_instrs: Option<Box<Block>>,
}

impl ProgramBlocks {
    pub fn new() -> Self {
        ProgramBlocks {
            functions: HashMap::new(),
            global_instrs: None,
        }
    }
}

pub struct ReloopedProgram {
    pub program_blocks: Box<ProgramBlocks>,
    pub program_metadata: Box<ProgramMetadata>,
}

pub fn reloop(mut prog: Box<Program>) -> ReloopedProgram {
    let mut program_blocks = ProgramBlocks::new();

    let mut loop_block_id_generator = IdGenerator::<LoopBlockId>::new();
    let mut multiple_block_id_generator = IdGenerator::<MultipleBlockId>::new();
    for (fun_id, function) in prog.program_instructions.functions {
        // function with no body (ie. one that we'll link to in JS runtime)
        if !function.body_is_defined || function.instrs.is_empty() {
            program_blocks.functions.insert(
                fun_id,
                ReloopedFunction {
                    block: None,
                    label_variable: None,
                    type_info: function.type_info,
                    param_var_mappings: function.param_var_mappings,
                    body_is_defined: function.body_is_defined,
                },
            );
            continue;
        }
        let label_var = init_label_variable(&mut prog.program_metadata);
        let (labels, entry) = soupify(function.instrs, &mut prog.program_metadata);

        let mut context = RelooperContext::new(
            &mut loop_block_id_generator,
            &mut multiple_block_id_generator,
            &label_var,
        );
        let block = create_block_from_labels(
            labels,
            vec![entry],
            &mut context,
            &mut prog.program_metadata,
        );
        match block {
            Some(block) => {
                info!("Created block for function {}:\n{}", fun_id, block);
                assert_no_branch_instrs_left(&block);
                program_blocks.functions.insert(
                    fun_id,
                    ReloopedFunction {
                        block: Some(block),
                        label_variable: Some(label_var),
                        type_info: function.type_info,
                        param_var_mappings: function.param_var_mappings,
                        body_is_defined: function.body_is_defined,
                    },
                );
            }
            None => error!(
                "No block created for function {}, even though it had instructions",
                fun_id
            ),
        }
    }
    if !prog.program_instructions.global_instrs.is_empty() {
        let label_var = init_label_variable(&mut prog.program_metadata);
        let (labels, entry) = soupify(
            prog.program_instructions.global_instrs,
            &mut prog.program_metadata,
        );

        let mut context = RelooperContext::new(
            &mut loop_block_id_generator,
            &mut multiple_block_id_generator,
            &label_var,
        );
        let block = create_block_from_labels(
            labels,
            vec![entry],
            &mut context,
            &mut prog.program_metadata,
        );
        match block {
            Some(block) => {
                info!("Created block for global instructions:\n{}", block);
                assert_no_branch_instrs_left(&block);
                program_blocks.global_instrs = Some(block);
            }
            None => error!("No block created for global instructions, even though non-empty"),
        }
    }

    ReloopedProgram {
        program_blocks: Box::new(program_blocks),
        program_metadata: prog.program_metadata,
    }
}

fn init_label_variable(prog_metadata: &mut ProgramMetadata) -> VarId {
    let label_var = prog_metadata.new_var(ValueType::LValue);
    // make label variable an unsigned long
    prog_metadata
        .add_var_type(label_var.to_owned(), Box::new(IrType::U64))
        .unwrap();
    label_var
}

fn assert_no_branch_instrs_left(block: &Box<Block>) {
    match &**block {
        Block::Simple { internal, next } => {
            for instr in &internal.instrs {
                let is_branch_instr = matches!(
                    instr,
                    Instruction::Br(..) | Instruction::BrIfEq(..) | Instruction::BrIfNotEq(..)
                );
                assert!(
                    !is_branch_instr,
                    "No branch instructions should be left in output of relooper"
                )
            }
            if let Some(next) = next {
                assert_no_branch_instrs_left(next);
            }
        }
        Block::Loop { inner, next, .. } => {
            assert_no_branch_instrs_left(inner);
            if let Some(next) = next {
                assert_no_branch_instrs_left(next);
            }
        }
        Block::Multiple {
            handled_blocks,
            next,
            ..
        } => {
            for handled_block in handled_blocks {
                assert_no_branch_instrs_left(handled_block);
            }
            if let Some(next) = next {
                assert_no_branch_instrs_left(next);
            }
        }
    }
}

fn create_block_from_labels(
    mut labels: Labels,
    entries: Entries,
    context: &mut RelooperContext,
    prog_metadata: &mut Box<ProgramMetadata>,
) -> Option<Box<Block>> {
    let reachability = calculate_reachability(&labels);
    let reachability_from_entries = combine_reachability_from_entries(&reachability, &entries);

    if entries.is_empty() {
        return None;
    }

    // if we have a single entry that we can't return to, create a simple block
    if entries.len() == 1 {
        let single_entry = entries.first().unwrap();
        // check that the single entry isn't contained in the set of possible
        // destination labels from this entry
        if !reachability_from_entries.contains(single_entry) {
            let next_entries: Entries = labels.get(single_entry).unwrap().possible_branch_targets();
            let mut this_label = labels.remove(single_entry).unwrap();
            replace_branch_instrs(&mut this_label, context, prog_metadata);
            let next_block = create_block_from_labels(labels, next_entries, context, prog_metadata);
            return Some(Box::new(Block::Simple {
                internal: this_label,
                next: next_block,
            }));
        }
    }

    // check if we can return to all of the entries, if so, create a loop block
    let mut can_return_to_all_entries = true;
    for entry in &entries {
        let reachable = reachability_from_entries.contains(entry);
        if !reachable {
            can_return_to_all_entries = false;
            break;
        }
    }
    if can_return_to_all_entries {
        return Some(create_loop_block(
            labels,
            entries,
            reachability,
            context,
            prog_metadata,
        ));
    }

    // if we have more than one entry, try to create a multiple block
    if entries.len() > 1 {
        match try_create_multiple_block(&labels, &entries, &reachability, context, prog_metadata) {
            None => {}
            Some(block) => return Some(block),
        }
    }

    // if creating a multiple block fails, create a loop block
    Some(create_loop_block(
        labels,
        entries,
        reachability,
        context,
        prog_metadata,
    ))
}

fn calculate_reachability(labels: &Labels) -> ReachabilityMap {
    let mut possible_branch_targets = HashMap::new();
    for label in labels.values() {
        possible_branch_targets.insert(label.label.to_owned(), label.possible_branch_targets());
    }

    // compute the transitive closure of possible_branch_targets
    let mut reachability: ReachabilityMap = possible_branch_targets.clone();
    loop {
        let mut made_changes = false;
        for (_source_label, reachable_labels) in reachability.iter_mut() {
            let mut i = 0;
            loop {
                if i >= reachable_labels.len() {
                    break;
                }
                // for all the labels we currently know we can reach from _source_label,
                // if any of their branch targets aren't in the set of reachable nodes,
                // add them
                let reachable_label = reachable_labels.get(i).unwrap();
                for dest_label in possible_branch_targets
                    .get(reachable_label)
                    .unwrap_or(&Vec::<LabelId>::new())
                {
                    if !reachable_labels.contains(dest_label) {
                        reachable_labels.push(dest_label.to_owned());
                        made_changes = true;
                    }
                }
                i += 1;
            }
        }
        // keep looping until there's no more labels to add
        if !made_changes {
            break;
        }
    }
    reachability
}

fn combine_reachability_from_entries(
    reachability: &ReachabilityMap,
    entries: &Vec<LabelId>,
) -> Vec<LabelId> {
    // use a hashset to combine the reachable labels, cos we don't want duplicates
    let mut combined_reachability = HashSet::new();

    for entry in entries {
        // add the reachability for each entry
        for label in reachability.get(entry).unwrap() {
            combined_reachability.insert(label.to_owned());
        }
    }

    Vec::from_iter(combined_reachability)
}

fn replace_branch_instrs(
    label: &mut Label,
    context: &RelooperContext,
    prog_metadata: &mut Box<ProgramMetadata>,
) {
    // the only branch instructions in a label are at the end.
    //  (or a return, or a break/continue/endHandled instruction that's already been converted)
    // either the label ends in a single unconditional branch,
    // or it ends in a conditional branch followed by an unconditional branch.

    if label.instrs.is_empty() {
        return;
    }

    // we can safely unwrap, because a label must have instructions.
    let unconditional_branch_label_id = match label.instrs.last().unwrap() {
        Instruction::Br(_, label_id) => Some(label_id),
        // any other instruction we'll ignore.
        // most likely this is a return or a break/continue/endHandled
        // that's already been processed, or the other possible case
        // is that this label is the end of the global instructions, in which
        // case it doesn't need to end with a branch.
        _ => None,
    };

    let conditional_branch_instr_index = match unconditional_branch_label_id {
        Some(_) => {
            // handle the possibility that the unconditional branch is the only instruction in the label
            if label.instrs.len() > 1 {
                Some(label.instrs.len() - 2)
            } else {
                None
            }
        }
        None => {
            // handle the possibility that the (setLabel + break/continue/endHandled) are the only
            // two instructions in the label
            if label.instrs.len() > 2 {
                Some(label.instrs.len() - 3)
            } else {
                None
            }
        }
    };

    let conditional_branch_instr = match conditional_branch_instr_index {
        Some(index) => match label.instrs.get(index).unwrap() {
            i @ Instruction::BrIfEq(..) | i @ Instruction::BrIfNotEq(..) => Some(i),
            _ => None,
        },
        None => None,
    };

    match (conditional_branch_instr, unconditional_branch_label_id) {
        (None, None) => {
            // no need to do anything. either
            //   - only an unconditional branch is present, and it's already
            //     converted to a break/continue/endHandled
            //   - both the conditional and unconditional branches have already been
            //     converted to a break/continue/endHandled
        }
        (None, Some(unconditional_branch_label_id)) => {
            // the case where only an unconditional branch is present, the
            // conditional branch either isn't present or has already been handled

            // replace the branch with an instruction setting the label variable
            let new_instr = Instruction::SimpleAssignment(
                prog_metadata.new_instr_id(),
                context.label_variable.to_owned(),
                Src::Constant(Constant::Int(unconditional_branch_label_id.as_u64() as i128)),
            );
            // remove the unconditional branch
            label.instrs.remove(label.instrs.len() - 1);
            // add the new instruction to replace it
            label.instrs.push(new_instr);
        }
        (Some(conditional_branch_instr), None) => {
            // the case where we have a conditional branch instruction, but the unconditional
            // branch instruction has already been converted to a break/continue/endHandled
            let else_instrs = vec![
                label.instrs.get(label.instrs.len() - 2).unwrap().to_owned(),
                label.instrs.last().unwrap().to_owned(),
            ];
            let new_instr = match conditional_branch_instr {
                Instruction::BrIfEq(_, src1, src2, label_id) => Instruction::IfEqElse(
                    prog_metadata.new_instr_id(),
                    src1.to_owned(),
                    src2.to_owned(),
                    vec![Instruction::SimpleAssignment(
                        prog_metadata.new_instr_id(),
                        context.label_variable.to_owned(),
                        Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                    )],
                    else_instrs,
                ),
                Instruction::BrIfNotEq(_, src1, src2, label_id) => Instruction::IfNotEqElse(
                    prog_metadata.new_instr_id(),
                    src1.to_owned(),
                    src2.to_owned(),
                    vec![Instruction::SimpleAssignment(
                        prog_metadata.new_instr_id(),
                        context.label_variable.to_owned(),
                        Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                    )],
                    else_instrs,
                ),
                _ => unreachable!(),
            };
            // remove the break/continue/endHandled (these two instructions are moved into the else clause)
            label.instrs.pop();
            // remove the setLabel belonging to the break/continue/endHandled
            label.instrs.pop();
            // remove the conditional branch
            label.instrs.pop();
            // add the if/else instruction to replace it
            label.instrs.push(new_instr);
        }
        (Some(conditional_branch_instr), Some(unconditional_branch_label_id)) => {
            // the case where we have both a conditional branch instruction and an unconditional
            // branch statement present
            let else_instrs = vec![Instruction::SimpleAssignment(
                prog_metadata.new_instr_id(),
                context.label_variable.to_owned(),
                Src::Constant(Constant::Int(unconditional_branch_label_id.as_u64() as i128)),
            )];
            let new_instr = match conditional_branch_instr {
                Instruction::BrIfEq(_, src1, src2, label_id) => Instruction::IfEqElse(
                    prog_metadata.new_instr_id(),
                    src1.to_owned(),
                    src2.to_owned(),
                    vec![Instruction::SimpleAssignment(
                        prog_metadata.new_instr_id(),
                        context.label_variable.to_owned(),
                        Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                    )],
                    else_instrs,
                ),
                Instruction::BrIfNotEq(_, src1, src2, label_id) => Instruction::IfNotEqElse(
                    prog_metadata.new_instr_id(),
                    src1.to_owned(),
                    src2.to_owned(),
                    vec![Instruction::SimpleAssignment(
                        prog_metadata.new_instr_id(),
                        context.label_variable.to_owned(),
                        Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                    )],
                    else_instrs,
                ),
                _ => unreachable!(),
            };
            // remove the unconditional branch
            label.instrs.pop();
            // remove the conditional branch
            label.instrs.pop();
            // add the if/else instruction to replace it
            label.instrs.push(new_instr);
        }
    }
}

fn create_loop_block(
    labels: Labels,
    entries: Entries,
    reachability: ReachabilityMap,
    context: &mut RelooperContext,
    prog_metadata: &mut Box<ProgramMetadata>,
) -> Box<Block> {
    let mut inner_labels: Labels = HashMap::new();
    let mut next_labels: Labels = HashMap::new();
    // find the labels that can return to one of the entries, and those that can't
    for (label_id, label) in labels {
        let mut can_return = false;
        for entry in &entries {
            if reachability.get(&label_id).unwrap().contains(entry) {
                can_return = true;
                break;
            }
        }
        if can_return {
            inner_labels.insert(label_id, label);
        } else {
            next_labels.insert(label_id, label);
        }
    }

    // find the entries for the next block
    let mut next_entries = HashSet::new();
    for label in inner_labels.values() {
        // "the next block's entry labels are all the labels in the next block that can
        //  be reached by the inner block" (Relooper paper, p9)
        //   > does this mean direct branches or reached along some execution path?
        //     surely direct branches otherwise all the remaining labels would
        //     become entries...
        for branch_target in label.possible_branch_targets() {
            // branch targets from the inner block that are labels in the
            // next block are entry labels for the next block
            if next_labels.get(&branch_target).is_some() {
                next_entries.insert(branch_target);
            }
        }
    }
    let next_entries: Entries = Vec::from_iter(next_entries);

    let loop_block_id = context.loop_block_id_generator.new_id();

    // turn branch instructions to start of loop and out of loop into continue and break instructions
    replace_branch_instrs_inside_loop(
        &mut inner_labels,
        &entries,
        &next_entries,
        &loop_block_id,
        context,
        prog_metadata,
    );

    // entries for the inner block are the same as entries for this block
    // we can unwrap inner_block cos we know we can return to entries, so there must be
    // some labels in inner
    let inner_block =
        create_block_from_labels(inner_labels, entries, context, prog_metadata).unwrap();
    let next_block = create_block_from_labels(next_labels, next_entries, context, prog_metadata);

    Box::new(Block::Loop {
        id: loop_block_id,
        inner: inner_block,
        next: next_block,
    })
}

fn replace_branch_instrs_inside_loop(
    inner_labels: &mut Labels,
    loop_entries: &Entries,
    next_entries: &Entries,
    loop_block_id: &LoopBlockId,
    context: &RelooperContext,
    prog_metadata: &mut Box<ProgramMetadata>,
) {
    for inner_label in inner_labels.values_mut() {
        for i in 0..inner_label.instrs.len() {
            let instr = inner_label.instrs.get(i).unwrap();
            let mut new_instrs: Vec<Instruction> = Vec::new();
            match instr {
                Instruction::Br(_, label_id) => {
                    if loop_entries.contains(label_id) {
                        // set the label variable
                        new_instrs.push(Instruction::SimpleAssignment(
                            prog_metadata.new_instr_id(),
                            context.label_variable.to_owned(),
                            Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                        ));
                        // turn branch back to the start of the loop into a continue
                        new_instrs.push(Instruction::Continue(
                            prog_metadata.new_instr_id(),
                            loop_block_id.to_owned(),
                        ));
                    } else if next_entries.contains(label_id) {
                        // set the label variable
                        new_instrs.push(Instruction::SimpleAssignment(
                            prog_metadata.new_instr_id(),
                            context.label_variable.to_owned(),
                            Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                        ));
                        // turn branch out of the loop into a break
                        new_instrs.push(Instruction::Break(
                            prog_metadata.new_instr_id(),
                            loop_block_id.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfEq(_, src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        new_instrs.push(Instruction::IfEqElse(
                            prog_metadata.new_instr_id(),
                            src1.to_owned(),
                            src2.to_owned(),
                            vec![
                                // set the label variable
                                Instruction::SimpleAssignment(
                                    prog_metadata.new_instr_id(),
                                    context.label_variable.to_owned(),
                                    Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                                ),
                                // turn branch back to the start of the loop into a continue
                                Instruction::Continue(
                                    prog_metadata.new_instr_id(),
                                    loop_block_id.to_owned(),
                                ),
                            ],
                            vec![],
                        ));
                    } else if next_entries.contains(label_id) {
                        new_instrs.push(Instruction::IfEqElse(
                            prog_metadata.new_instr_id(),
                            src1.to_owned(),
                            src2.to_owned(),
                            vec![
                                // set the label variable
                                Instruction::SimpleAssignment(
                                    prog_metadata.new_instr_id(),
                                    context.label_variable.to_owned(),
                                    Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                                ),
                                // turn branch out of the loop into a break
                                Instruction::Break(
                                    prog_metadata.new_instr_id(),
                                    loop_block_id.to_owned(),
                                ),
                            ],
                            vec![],
                        ));
                    }
                }
                Instruction::BrIfNotEq(_, src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        new_instrs.push(Instruction::IfNotEqElse(
                            prog_metadata.new_instr_id(),
                            src1.to_owned(),
                            src2.to_owned(),
                            vec![
                                // set the label variable
                                Instruction::SimpleAssignment(
                                    prog_metadata.new_instr_id(),
                                    context.label_variable.to_owned(),
                                    Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                                ),
                                // turn branch back to the start of the loop into a continue
                                Instruction::Continue(
                                    prog_metadata.new_instr_id(),
                                    loop_block_id.to_owned(),
                                ),
                            ],
                            vec![],
                        ));
                    } else if next_entries.contains(label_id) {
                        new_instrs.push(Instruction::IfNotEqElse(
                            prog_metadata.new_instr_id(),
                            src1.to_owned(),
                            src2.to_owned(),
                            vec![
                                // set the label variable
                                Instruction::SimpleAssignment(
                                    prog_metadata.new_instr_id(),
                                    context.label_variable.to_owned(),
                                    Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                                ),
                                // turn branch out of the loop into a break
                                Instruction::Break(
                                    prog_metadata.new_instr_id(),
                                    loop_block_id.to_owned(),
                                ),
                            ],
                            vec![],
                        ));
                    }
                }
                _ => {}
            }
            if !new_instrs.is_empty() {
                inner_label.instrs.remove(i);
                for (j, instr) in new_instrs.into_iter().enumerate() {
                    inner_label.instrs.insert(i + j, instr);
                }
            }
        }
    }
}

fn try_create_multiple_block(
    labels: &Labels,
    entries: &Entries,
    reachability: &ReachabilityMap,
    context: &mut RelooperContext,
    prog_metadata: &mut Box<ProgramMetadata>,
) -> Option<Box<Block>> {
    // "for each entry, find all the labels it reaches that can't be reached by any other entry"
    let mut uniquely_reachable_labels: HashMap<LabelId, Vec<LabelId>> = HashMap::new();
    for entry in entries {
        // let reachable_labels = labels.get(entry).unwrap().possible_branch_targets();
        let mut reachable_labels = reachability.get(entry).unwrap().clone();
        if !reachable_labels.contains(entry) {
            reachable_labels.push(entry.to_owned());
        }
        // check which of the labels can't be reached by any other entry
        for label in &reachable_labels {
            let mut uniquely_reachable = true;
            for other_entry in entries {
                if other_entry == entry {
                    continue;
                }
                if other_entry == label || reachability.get(other_entry).unwrap().contains(label) {
                    uniquely_reachable = false;
                    break;
                }
            }
            if uniquely_reachable {
                match uniquely_reachable_labels.get_mut(entry) {
                    Some(labels) => labels.push(label.to_owned()),
                    None => {
                        uniquely_reachable_labels.insert(entry.to_owned(), vec![label.to_owned()]);
                    }
                }
            }
        }
    }
    if !uniquely_reachable_labels.is_empty() {
        // map of entry to labels for each handled block
        let mut handled_labels: HashMap<LabelId, Labels> = HashMap::new();
        // let mut handled_entries = HashSet::new();
        let mut next_labels: Labels = HashMap::new();
        // split labels into handled and next labels
        for (label_id, label) in labels {
            let mut handled_by_entry: Option<&LabelId> = None;
            for (entry, entry_unique_labels) in &uniquely_reachable_labels {
                if entry_unique_labels.contains(label_id) {
                    // handled_entries.insert(entry);
                    handled_by_entry = Some(entry);
                    break;
                }
            }
            if let Some(entry) = handled_by_entry {
                match handled_labels.get_mut(entry) {
                    Some(labels) => {
                        labels.insert(label_id.to_owned(), label.to_owned());
                    }
                    None => {
                        let mut labels: Labels = HashMap::new();
                        labels.insert(label_id.to_owned(), label.to_owned());
                        handled_labels.insert(entry.to_owned(), labels);
                    }
                }
            } else {
                next_labels.insert(label_id.to_owned(), label.to_owned());
            }
        }

        let mut next_entries = entries.to_owned();
        // keep all the non-handled entries
        next_entries.retain(|e| handled_labels.get(e).is_none());

        let multiple_block_id = context.multiple_block_id_generator.new_id();

        let mut handled_blocks = Vec::new();
        for (handled_label_entry, mut handled_labels) in handled_labels {
            // add any new entries that are branched to from inside the handled blocks
            for handled_label in handled_labels.values() {
                for branch_target in handled_label.possible_branch_targets() {
                    // if this is a branch to the next block
                    if next_labels.get(&branch_target).is_some() {
                        if !next_entries.contains(&branch_target) {
                            next_entries.push(branch_target);
                        }
                    }
                }
            }

            replace_branch_instrs_inside_handled_block(
                &mut handled_labels,
                &next_entries,
                &multiple_block_id,
                context,
                prog_metadata,
            );

            let handled_block = create_block_from_labels(
                handled_labels,
                vec![handled_label_entry],
                context,
                prog_metadata,
            )
            .unwrap();
            handled_blocks.push(handled_block);
        }

        let next_block =
            create_block_from_labels(next_labels, next_entries, context, prog_metadata);

        let pre_handled_blocks_instrs = vec![Instruction::ReferenceVariable(
            prog_metadata.new_instr_id(),
            context.label_variable.to_owned(),
        )];

        return Some(Box::new(Block::Multiple {
            id: multiple_block_id,
            pre_handled_blocks_instrs,
            handled_blocks,
            next: next_block,
        }));
    }
    None
}

fn replace_branch_instrs_inside_handled_block(
    handled_labels: &mut Labels,
    next_entries: &Entries,
    multiple_block_id: &MultipleBlockId,
    context: &RelooperContext,
    prog_metadata: &mut Box<ProgramMetadata>,
) {
    for handled_label in handled_labels.values_mut() {
        for i in 0..handled_label.instrs.len() {
            let instr = handled_label.instrs.get(i).unwrap();
            let mut new_instrs: Vec<Instruction> = Vec::new();
            match instr {
                Instruction::Br(_, label_id) => {
                    if next_entries.contains(label_id) {
                        new_instrs.push(Instruction::SimpleAssignment(
                            prog_metadata.new_instr_id(),
                            context.label_variable.to_owned(),
                            Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                        ));
                        // turn branch to next block into end handled block instruction
                        new_instrs.push(Instruction::EndHandledBlock(
                            prog_metadata.new_instr_id(),
                            multiple_block_id.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfEq(_, src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        new_instrs.push(Instruction::IfEqElse(
                            prog_metadata.new_instr_id(),
                            src1.to_owned(),
                            src2.to_owned(),
                            vec![
                                Instruction::SimpleAssignment(
                                    prog_metadata.new_instr_id(),
                                    context.label_variable.to_owned(),
                                    Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                                ),
                                // turn branch to next block into end handled block instruction
                                Instruction::EndHandledBlock(
                                    prog_metadata.new_instr_id(),
                                    multiple_block_id.to_owned(),
                                ),
                            ],
                            vec![],
                        ));
                    }
                }
                Instruction::BrIfNotEq(_, src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        new_instrs.push(Instruction::IfNotEqElse(
                            prog_metadata.new_instr_id(),
                            src1.to_owned(),
                            src2.to_owned(),
                            vec![
                                Instruction::SimpleAssignment(
                                    prog_metadata.new_instr_id(),
                                    context.label_variable.to_owned(),
                                    Src::Constant(Constant::Int(label_id.as_u64() as i128)),
                                ),
                                // turn branch to next block into end handled block instruction
                                Instruction::EndHandledBlock(
                                    prog_metadata.new_instr_id(),
                                    multiple_block_id.to_owned(),
                                ),
                            ],
                            vec![],
                        ));
                    }
                }
                _ => {}
            }
            if !new_instrs.is_empty() {
                handled_label.instrs.remove(i);
                for (j, instr) in new_instrs.into_iter().enumerate() {
                    handled_label.instrs.insert(i + j, instr);
                }
            }
        }
    }
}
