use std::cmp::max;
use std::fmt;
use std::fmt::Formatter;

const INDENT_SIZE: u32 = 2;

pub struct IndentLevel(u32);

impl IndentLevel {
    pub fn zero() -> Self {
        IndentLevel(0)
    }

    pub fn increment(&mut self) {
        self.0 = self.0 + 1;
    }

    pub fn decrement(&mut self) {
        self.0 = max(self.0 - 1, 0);
    }

    pub fn width(&self) -> u32 {
        self.0
    }
}

pub fn write_indent(f: &mut Formatter<'_>, indent_level: &IndentLevel) -> fmt::Result {
    for _ in 0..indent_level.width() {
        write!(f, "│ ")?;
    }
    Ok(())
}
