use crate::display::{write_indent, IndentLevel};
use crate::middle_end::ids::LabelId;
use crate::middle_end::instructions::Instruction;
use crate::middle_end::ir::Program;
use crate::relooper::soupify::{soupify, Label};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;

pub fn reloop(mut prog: Box<Program>) {
    for (_fun_id, function) in prog.program_instructions.functions {
        // function with no body (ie. one that we'll link to in JS runtime)
        if function.instrs.is_empty() {
            continue;
        }
        let (labels, entry) = soupify(
            function.instrs,
            &mut prog.program_metadata.label_id_generator,
        );
        println!("\nlabels:");
        for (_, label) in &labels {
            println!("{}", label);
        }
        println!();
        let block = create_block_from_labels(labels, vec![entry]);
        match block {
            Some(block) => println!("created block\n{}", block),
            None => println!("No block created"),
        }
    }
    let labels = soupify(
        prog.program_instructions.global_instrs,
        &mut prog.program_metadata.label_id_generator,
    );
}

#[derive(Debug)]
pub enum Block {
    Simple {
        internal: Label,
        next: Option<Box<Block>>,
    },
    Loop {
        inner: Box<Block>,
        next: Option<Box<Block>>,
    },
    Multiple {
        handled_blocks: Vec<Box<Block>>,
        next: Option<Box<Block>>,
    },
}

impl Block {
    fn print(&self, f: &mut Formatter<'_>, indent_level: &mut IndentLevel) -> fmt::Result {
        match self {
            Block::Simple { internal, next } => {
                write_indent(f, indent_level)?;
                writeln!(f, "Simple {{")?;
                indent_level.increment();
                write_indent(f, indent_level)?;
                writeln!(f, "internal: {}", internal.label)?;
                match next {
                    Some(next) => {
                        write_indent(f, indent_level)?;
                        writeln!(f, "next:")?;
                        indent_level.increment();
                        next.print(f, indent_level)?;
                        indent_level.decrement();
                    }
                    None => {
                        write_indent(f, indent_level)?;
                        writeln!(f, "next: NULL")?;
                    }
                }
                indent_level.decrement();
                write_indent(f, indent_level)?;
                writeln!(f, "}}")
            }
            Block::Loop { inner, next } => {
                write_indent(f, indent_level)?;
                writeln!(f, "Loop {{")?;
                indent_level.increment();
                write_indent(f, indent_level)?;
                writeln!(f, "inner:")?;
                indent_level.increment();
                inner.print(f, indent_level)?;
                indent_level.decrement();
                match next {
                    Some(next) => {
                        write_indent(f, indent_level)?;
                        writeln!(f, "next:",)?;
                        indent_level.increment();
                        next.print(f, indent_level)?;
                        indent_level.decrement();
                    }
                    None => {
                        write_indent(f, indent_level)?;
                        writeln!(f, "next: NULL")?;
                    }
                }
                indent_level.decrement();
                write_indent(f, indent_level)?;
                writeln!(f, "}}")
            }
            Block::Multiple {
                handled_blocks,
                next,
            } => {
                write_indent(f, indent_level)?;
                writeln!(f, "Multiple {{")?;
                indent_level.increment();
                write_indent(f, indent_level)?;
                writeln!(f, "handled: ")?;
                indent_level.increment();
                for handled in &handled_blocks[..handled_blocks.len() - 1] {
                    handled.print(f, indent_level)?;
                }
                handled_blocks[handled_blocks.len() - 1].print(f, indent_level)?;
                indent_level.decrement();
                match next {
                    Some(next) => {
                        write_indent(f, indent_level)?;
                        writeln!(f, "next:")?;
                        indent_level.increment();
                        next.print(f, indent_level)?;
                        indent_level.decrement();
                    }
                    None => {
                        write_indent(f, indent_level)?;
                        writeln!(f, "next: NULL")?;
                    }
                }
                indent_level.decrement();
                write_indent(f, indent_level)?;
                writeln!(f, "}}")
            }
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print(f, &mut IndentLevel::zero())
    }
}

pub type Labels = HashMap<LabelId, Label>;
type Entries = Vec<LabelId>;
type ReachabilityMap = HashMap<LabelId, Vec<LabelId>>;

fn create_block_from_labels(mut labels: Labels, entries: Entries) -> Option<Box<Block>> {
    let reachability = calculate_reachability(&labels);
    let reachability_from_entries = combine_reachability_from_entries(&reachability, &entries);
    // print!("reachable from entries: ");
    // for label in &reachability_from_entries {
    //     print!("{}  ", label);
    // }
    // println!();

    if entries.is_empty() {
        return None;
    }

    // if we have a single entry that we can't return to, create a simple block
    if entries.len() == 1 {
        let single_entry = entries.first().unwrap();
        // check that the single entry isn't contained in the set of possible
        // destination labels from this entry
        if !reachability_from_entries.contains(single_entry) {
            println!("\nCreate simple block: {}", single_entry);
            let next_entries: Entries = labels.get(single_entry).unwrap().possible_branch_targets();
            print!("  next entries: ");
            for entry in &next_entries {
                print!("{}  ", entry);
            }
            println!();
            let this_label = labels.remove(single_entry).unwrap();
            let next_block = create_block_from_labels(labels, next_entries);
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
        return Some(create_loop_block(labels, entries, reachability));
    }

    // if we have more than one entry, try to create a multiple block
    if entries.len() > 1 {
        match try_create_multiple_block(&labels, &entries, &reachability) {
            None => {}
            Some(block) => return Some(block),
        }
    }

    // if creating a multiple block fails, create a loop block
    Some(create_loop_block(labels, entries, reachability))
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
        // println!("  {}", entry);
        for label in reachability.get(&entry).unwrap() {
            combined_reachability.insert(label.to_owned());
        }
    }

    Vec::from_iter(combined_reachability)
}

fn create_loop_block(
    labels: Labels,
    entries: Entries,
    reachability: ReachabilityMap,
) -> Box<Block> {
    println!("\ncreate loop block");
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
    for (_label_id, label) in &inner_labels {
        // "the next block's entry labels are all the labels in the next block that can
        //  be reached by the inner block" (Relooper paper, p9)
        //   > does this mean direct branches or reached along some execution path?
        //     surely direct branches otherwise all the remaining labels would
        //     become entries...
        for branch_target in label.possible_branch_targets() {
            // branch targets from the inner block that are labels in the
            // next block are entry labels for the next block
            if let Some(_) = next_labels.get(&branch_target) {
                next_entries.insert(branch_target);
            }
        }
    }
    let next_entries: Entries = Vec::from_iter(next_entries);

    // turn branch instructions to start of loop and out of loop into continue and break instructions
    replace_branch_instrs_inside_loop(&mut inner_labels, &entries, &next_entries);

    print!("  next labels: ");
    for (label_id, _) in &next_labels {
        print!("{}  ", label_id);
    }
    println!();
    print!("  next entries: ");
    for entry in &next_entries {
        print!("{}  ", entry);
    }
    println!();

    // entries for the inner block are the same as entries for this block
    // we can unwrap inner_block cos we know we can return to entries, so there must be
    // some labels in inner
    let inner_block = create_block_from_labels(inner_labels, entries).unwrap();
    let next_block = create_block_from_labels(next_labels, next_entries);

    Box::new(Block::Loop {
        inner: inner_block,
        next: next_block,
    })
}

fn replace_branch_instrs_inside_loop(
    inner_labels: &mut Labels,
    loop_entries: &Entries,
    next_entries: &Entries,
) {
    for (_inner_label_id, inner_label) in inner_labels {
        for i in 0..inner_label.instrs.len() {
            let instr = inner_label.instrs.get(i).unwrap();
            let mut new_instr: Option<Instruction> = None;
            match instr {
                Instruction::Br(label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr = Some(Instruction::Continue);
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr = Some(Instruction::Break);
                    }
                }
                Instruction::BrIfEq(src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr =
                            Some(Instruction::ContinueIfEq(src1.to_owned(), src2.to_owned()));
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr = Some(Instruction::BreakIfEq(src1.to_owned(), src2.to_owned()));
                    }
                }
                Instruction::BrIfNotEq(src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr = Some(Instruction::ContinueIfNotEq(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr =
                            Some(Instruction::BreakIfNotEq(src1.to_owned(), src2.to_owned()));
                    }
                }
                Instruction::BrIfLT(src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr =
                            Some(Instruction::ContinueIfLT(src1.to_owned(), src2.to_owned()));
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr = Some(Instruction::BreakIfLT(src1.to_owned(), src2.to_owned()));
                    }
                }
                Instruction::BrIfGT(src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr =
                            Some(Instruction::ContinueIfGT(src1.to_owned(), src2.to_owned()));
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr = Some(Instruction::BreakIfGT(src1.to_owned(), src2.to_owned()));
                    }
                }
                Instruction::BrIfLE(src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr =
                            Some(Instruction::ContinueIfLE(src1.to_owned(), src2.to_owned()));
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr = Some(Instruction::BreakIfLE(src1.to_owned(), src2.to_owned()));
                    }
                }
                Instruction::BrIfGE(src1, src2, label_id) => {
                    if loop_entries.contains(label_id) {
                        // turn branch back to the start of the loop into a continue
                        new_instr =
                            Some(Instruction::ContinueIfGE(src1.to_owned(), src2.to_owned()));
                    } else if next_entries.contains(label_id) {
                        // turn branch out of the loop into a break
                        new_instr = Some(Instruction::BreakIfGE(src1.to_owned(), src2.to_owned()));
                    }
                }
                _ => {}
            }
            if let Some(new_instr) = new_instr {
                let removed = inner_label.instrs.remove(i);
                println!("  replaced instr {}", removed);
                println!("  with instr     {}", new_instr);
                inner_label.instrs.insert(i, new_instr);
            }
        }
    }
}

fn try_create_multiple_block(
    labels: &Labels,
    entries: &Entries,
    reachability: &ReachabilityMap,
) -> Option<Box<Block>> {
    println!("\ntry create multiple block");
    print!("from labels ");
    for (label, _) in labels {
        print!("{} ", label);
    }
    println!();
    print!("with entries ");
    for entry in entries {
        print!("{} ", entry);
    }
    println!();
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
                if other_entry == label || reachability.get(other_entry).unwrap().contains(&label) {
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
    for (src_label, dest_labels) in &uniquely_reachable_labels {
        print!("  {} uniquely reaches ", src_label);
        for label in dest_labels {
            print!("{}  ", label);
        }
        println!();
    }
    if uniquely_reachable_labels.len() >= 1 {
        // map of entry to labels for each handled block
        let mut handled_labels: HashMap<LabelId, Labels> = HashMap::new();
        // let mut handled_entries = HashSet::new();
        let mut next_labels: Labels = HashMap::new();
        // split labels into handled and next labels
        for (label_id, label) in labels {
            let mut handled_by_entry: Option<&LabelId> = None;
            for (entry, entry_unique_labels) in &uniquely_reachable_labels {
                if entry_unique_labels.contains(&label_id) {
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

        // check which of the handled labels are entries
        // for (handled_label_id, _) in &handled_labels {}

        let mut next_entries = entries.to_owned();
        // keep all the non-handled entries
        next_entries.retain(|e| match handled_labels.get(e) {
            None => true,
            Some(_) => false,
        });

        let mut handled_blocks = Vec::new();
        for (handled_label_entry, mut handled_labels) in handled_labels {
            // add any new entries that are branched to from inside the handled blocks
            for (_, handled_label) in &handled_labels {
                for branch_target in handled_label.possible_branch_targets() {
                    // if this is a branch to the next block
                    if let Some(_) = next_labels.get(&branch_target) {
                        if !next_entries.contains(&branch_target) {
                            next_entries.push(branch_target);
                        }
                    }
                }
            }

            print!(
                "  Creating handled block: entry {}, labels ",
                handled_label_entry
            );
            for (label_id, _) in &handled_labels {
                print!("{}  ", label_id);
            }
            println!();

            replace_branch_instrs_inside_handled_block(&mut handled_labels, &next_entries);

            let handled_block =
                create_block_from_labels(handled_labels, vec![handled_label_entry]).unwrap();
            handled_blocks.push(handled_block);
        }

        let next_block = create_block_from_labels(next_labels, next_entries);

        return Some(Box::new(Block::Multiple {
            handled_blocks,
            next: next_block,
        }));
    }
    None
}

fn replace_branch_instrs_inside_handled_block(handled_labels: &mut Labels, next_entries: &Entries) {
    for (_handled_label_id, handled_label) in handled_labels {
        for i in 0..handled_label.instrs.len() {
            let instr = handled_label.instrs.get(i).unwrap();
            let mut new_instr: Option<Instruction> = None;
            match instr {
                Instruction::Br(label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlock);
                    }
                }
                Instruction::BrIfEq(src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlockIfEq(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfNotEq(src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlockIfNotEq(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfLT(src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlockIfLT(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfGT(src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlockIfGT(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfLE(src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlockIfLE(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    }
                }
                Instruction::BrIfGE(src1, src2, label_id) => {
                    if next_entries.contains(label_id) {
                        // turn branch to next block into end handled block instruction
                        new_instr = Some(Instruction::EndHandledBlockIfGE(
                            src1.to_owned(),
                            src2.to_owned(),
                        ));
                    }
                }
                _ => {}
            }
            if let Some(new_instr) = new_instr {
                let removed = handled_label.instrs.remove(i);
                println!("  replaced instr {}", removed);
                println!("  with instr     {}", new_instr);
                handled_label.instrs.insert(i, new_instr);
            }
        }
    }
}
