use std::{iter::Peekable, str::Chars};

#[cfg(test)]
#[path = "interpret_string_tests.rs"]
mod interpret_string_tests;

pub fn interpret_string(s: &str) -> Result<String, Error> {
    InterpretString::new(s).collect()
}

struct InterpretString<'a> {
    s: Peekable<Chars<'a>>,
}

impl<'a> InterpretString<'a> {
    pub fn new(s: &'a str) -> Self {
        InterpretString {
            s: s.chars().peekable(),
        }
    }
}

impl Iterator for InterpretString<'_> {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.s.next() {
            Some(c) => Some(match c {
                '\\' => match self.s.next() {
                    Some('\'') => Ok('\''),
                    Some('\"') => Ok('"'),
                    Some('?') => Ok('?'),
                    Some('\\') => Ok('\\'),
                    Some('n') => Ok('\n'),
                    Some('r') => Ok('\r'),
                    Some('t') => Ok('\t'),
                    Some('x') => {
                        // hex char code
                        // check whether the next digit is a valid hex digit,
                        // if not return error
                        match self.s.peek() {
                            Some(c) if c.is_ascii_hexdigit() => {
                                // store the hex digits in a buffer, so we can
                                // convert them after
                                let mut buffer = String::new();
                                // keep consuming until the next char either isn't
                                // a hex char or we get to the end of the string
                                while let Some(c) = self.s.next() {
                                    buffer.push(c);
                                    match self.s.peek() {
                                        Some(c) if c.is_ascii_hexdigit() => continue,
                                        Some(_) | None => break,
                                    }
                                }
                                // convert the hex code to an int, and then to
                                // the corresponding char
                                let hex_code = u32::from_str_radix(&buffer, 16).unwrap();
                                match char::from_u32(hex_code) {
                                    Some(c) => Ok(c),
                                    None => Err(Error::InvalidCharCode(buffer)),
                                }
                            }
                            Some(c) => Err(Error::InvalidEscapeChar(*c)),
                            None => Err(Error::EscapeCharAtEndOfString),
                        }
                    }
                    Some(c) if c.is_digit(8) => {
                        // octal char code
                        // store the octal digits in a buffer to convert them after
                        let mut buffer = String::new();
                        // the char we just consumed is the first octal digit
                        buffer.push(c);
                        // consume up to two more octal digits
                        loop {
                            // octal char sequences are limited to 3 characters,
                            // as per the C spec
                            if buffer.len() == 3 {
                                break;
                            }
                            match self.s.peek() {
                                Some(c) if c.is_digit(8) => {
                                    buffer.push(self.s.next().unwrap());
                                }
                                Some(_) | None => break,
                            }
                        }
                        // convert the octal code to an int, and then to the
                        // corresponding char
                        let oct_code = u32::from_str_radix(&buffer, 8).unwrap();
                        match char::from_u32(oct_code) {
                            Some(c) => Ok(c),
                            None => Err(Error::InvalidCharCode(buffer)),
                        }
                    }
                    Some(c) => Err(Error::InvalidEscapeChar(c)),
                    None => Err(Error::EscapeCharAtEndOfString),
                },
                c => Ok(c),
            }),
            None => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidEscapeChar(char),
    InvalidCharCode(String),
    EscapeCharAtEndOfString,
}
