use std::str::Chars;

pub fn interpret_string(s: &str) -> Result<String, Error> {
    InterpretString{s: s.chars()}.collect()
}

struct InterpretString<'a> {
    s: Chars<'a>,
}

impl Iterator for InterpretString<'_> {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.s.next() {
            Some(match c {
                '\\' => match self.s.next() {
                    Some('\'') => Ok('\''),
                    Some('\"') => Ok('"'),
                    Some('?') => Ok('?'),
                    Some('\\') => Ok('\\'),
                    Some('n') => Ok('\n'),
                    Some('r') => Ok('\r'),
                    Some('t') => Ok('\t'),
                    Some(c) => Err(Error::InvalidEscapeChar(c)),
                    None => Err(Error::EscapeCharAtEndOfString),
                },
                c => Ok(c),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidEscapeChar(char),
    EscapeCharAtEndOfString,
}