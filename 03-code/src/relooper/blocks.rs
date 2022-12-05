use crate::fmt_indented::{FmtIndented, IndentLevel};
use crate::middle_end::ids::LabelId;
use crate::middle_end::instructions::Instruction;
use std::fmt;
use std::fmt::Formatter;

/// A 'label' block. This is a list of instructions starting with a label
/// and ending with one or more branch instructions.
/// We call it a label to distinguish it from the output blocks we're generating.
#[derive(Debug, Clone)]
pub struct Label {
    pub label: LabelId,
    pub instrs: Vec<Instruction>,
}

impl Label {
    pub fn new(label: LabelId) -> Self {
        Label {
            label,
            instrs: Vec::new(),
        }
    }

    pub fn possible_branch_targets(&self) -> Vec<LabelId> {
        let mut targets = Vec::new();
        for instr in &self.instrs {
            match instr {
                Instruction::Br(label_id)
                | Instruction::BrIfEq(_, _, label_id)
                | Instruction::BrIfNotEq(_, _, label_id)
                | Instruction::BrIfGT(_, _, label_id)
                | Instruction::BrIfLT(_, _, label_id)
                | Instruction::BrIfGE(_, _, label_id)
                | Instruction::BrIfLE(_, _, label_id) => {
                    // set semantics - only want one copy of each label to branch to,
                    // even if there are multiple branches
                    if !targets.contains(label_id) {
                        targets.push(label_id.to_owned());
                    }
                }
                _ => {}
            }
        }
        targets
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Label: {}", self.label)?;
        for instr in &self.instrs {
            write!(f, "\n  {}", instr)?;
        }
        write!(f, "")
    }
}

/// An output block from the relooper algorithm
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

impl FmtIndented for Block {
    fn fmt_indented(&self, f: &mut Formatter<'_>, indent_level: &mut IndentLevel) -> fmt::Result {
        match self {
            Block::Simple { internal, next } => {
                indent_level.write(f)?;
                writeln!(f, "SIMPLE {{")?;
                indent_level.increment_marked();
                indent_level.write(f)?;
                writeln!(f, "internal: {}", internal.label)?;
                match next {
                    Some(next) => {
                        indent_level.write(f)?;
                        writeln!(f, "next:")?;
                        indent_level.increment();
                        next.fmt_indented(f, indent_level)?;
                        indent_level.decrement();
                    }
                    None => {
                        indent_level.write(f)?;
                        writeln!(f, "next: NULL")?;
                    }
                }
                indent_level.decrement();
                indent_level.write(f)?;
                writeln!(f, "}}")
            }
            Block::Loop { inner, next } => {
                indent_level.write(f)?;
                writeln!(f, "LOOP {{")?;
                indent_level.increment_marked();
                indent_level.write(f)?;
                writeln!(f, "inner:")?;
                indent_level.increment();
                inner.fmt_indented(f, indent_level)?;
                indent_level.decrement();
                match next {
                    Some(next) => {
                        indent_level.write(f)?;
                        writeln!(f, "next:",)?;
                        indent_level.increment();
                        next.fmt_indented(f, indent_level)?;
                        indent_level.decrement();
                    }
                    None => {
                        indent_level.write(f)?;
                        writeln!(f, "next: NULL")?;
                    }
                }
                indent_level.decrement();
                indent_level.write(f)?;
                writeln!(f, "}}")
            }
            Block::Multiple {
                handled_blocks,
                next,
            } => {
                indent_level.write(f)?;
                writeln!(f, "MULTIPLE {{")?;
                indent_level.increment_marked();
                indent_level.write(f)?;
                writeln!(f, "handled: ")?;
                indent_level.increment();
                for handled in &handled_blocks[..handled_blocks.len() - 1] {
                    handled.fmt_indented(f, indent_level)?;
                }
                handled_blocks[handled_blocks.len() - 1].fmt_indented(f, indent_level)?;
                indent_level.decrement();
                match next {
                    Some(next) => {
                        indent_level.write(f)?;
                        writeln!(f, "next:")?;
                        indent_level.increment();
                        next.fmt_indented(f, indent_level)?;
                        indent_level.decrement();
                    }
                    None => {
                        indent_level.write(f)?;
                        writeln!(f, "next: NULL")?;
                    }
                }
                indent_level.decrement();
                indent_level.write(f)?;
                writeln!(f, "}}")
            }
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, &mut IndentLevel::zero())
    }
}
