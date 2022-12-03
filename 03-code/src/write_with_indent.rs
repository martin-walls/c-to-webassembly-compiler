use std::cmp::max;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

const INDENT_SIZE: u32 = 2;

pub struct IndentLevel(u32, HashSet<u32>);

impl IndentLevel {
    pub fn zero() -> Self {
        IndentLevel(0, HashSet::new())
    }

    pub fn increment(&mut self) {
        self.0 = self.0 + 1;
    }

    pub fn increment_marked(&mut self) {
        self.1.insert(self.0);
        self.increment();
    }

    pub fn decrement(&mut self) {
        self.0 = max(self.0 - 1, 0);
        self.1.remove(&self.0);
    }

    fn is_marked(&self, level: u32) -> bool {
        self.1.contains(&level)
    }

    pub fn write(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..self.0 {
            if self.is_marked(i) {
                write!(f, "┊ ")?;
            } else {
                write!(f, "  ")?;
            }
        }
        Ok(())
    }
}
