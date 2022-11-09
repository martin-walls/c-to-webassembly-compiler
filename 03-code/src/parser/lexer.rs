use log::trace;
use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, Clone)]
pub enum Token {
    Bang,
    Percent,
    Caret,
    Ampersand,
    Asterisk,
    Minus,
    Plus,
    Eq,
    Tilde,
    Bar,
    Dot,
    LessThan,
    GreaterThan,
    Slash,
    Question,

    PlusEq,
    MinusEq,
    AsteriskEq,
    SlashEq,
    PercentEq,
    LeftShiftEq,
    RightShiftEq,
    AmpersandEq,
    CaretEq,
    BarEq,

    Arrow,
    DoublePlus,
    DoubleMinus,
    LeftShift,
    RightShift,
    LessThanEq,
    GreaterThanEq,
    DoubleEq,
    BangEq,
    DoubleAmpersand,
    DoubleBar,

    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    LeftCurly,
    RightCurly,
    Comma,
    Semicolon,
    Colon,

    SingleQuote,
    DoubleQuote,

    DecimalConstant(String),
    BinaryConstant(String),
    OctalConstant(String),
    HexConstant(String),
    FloatingConstant(String),
    StringLiteral(String),
    CharConstant(String),

    Identifier(String),
    TypedefName(String),

    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Int,
    Long,
    Register,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
}

// #[derive(Debug, Clone)]
// pub enum Keyword {
//
// }

type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    chars: Peekable<CharIndices<'input>>,
    typedef_names: &'input Vec<String>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str, typedef_names: &'input Vec<String>) -> Self {
        trace!("lexer input: {}", input);
        Lexer {
            chars: input.char_indices().peekable(),
            typedef_names,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut fsm = Fsm::new();
        let mut start = None;
        let mut end = None;
        loop {
            let (i, c) = match self.chars.peek() {
                Some((i, c)) => (i, c),
                // end of file
                None => {
                    match fsm.state {
                        // EOF
                        State::Start => return None,
                        _ => {
                            return match fsm.token {
                                Some(t) => Some(Ok((start.unwrap(), t, end.unwrap() + 1))),
                                None => Some(Err(LexError::InvalidEOF)),
                            }
                        }
                    }
                }
            };

            if start == None {
                if c.is_whitespace() {
                    self.chars.next();
                    continue;
                }
                start = Some(i.to_owned());
            }
            end = Some(i.to_owned());
            let current_token = fsm.token.to_owned();
            let new_state = fsm.step(c.to_owned(), &self.typedef_names);
            match new_state {
                Some(f) => {
                    self.chars.next();
                    fsm = f;
                }
                // no next state, so take the token we got to
                None => {
                    trace!("{:?}", current_token);
                    return match current_token {
                        Some(t) => Some(Ok((start.unwrap(), t, end.unwrap()))),
                        None => Some(Err(LexError::InvalidToken(start.unwrap(), i.to_owned()))),
                    };
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    InvalidToken(usize, usize),
    InvalidEOF,
}

#[derive(Debug)]
enum State {
    Start,
    Bang,
    BangEq,
    Ampersand,
    AmpersandEq,
    DoubleAmpersand,
    Percent,
    PercentEq,
    Asterisk,
    AsteriskEq,
    Caret,
    CaretEq,
    Plus,
    PlusEq,
    DoublePlus,
    Minus,
    MinusEq,
    DoubleMinus,
    Arrow,
    Eq,
    DoubleEq,
    Tilde,
    Bar,
    BarEq,
    DoubleBar,
    LessThan,
    LessThanEq,
    LeftShift,
    LeftShiftEq,
    GreaterThan,
    GreaterThanEq,
    RightShift,
    RightShiftEq,
    Slash,
    SlashEq,
    Question,
    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    LeftCurly,
    RightCurly,
    Comma,
    Semicolon,
    Colon,
    Identifier,
    Keyword(String),
    TypedefName,
    Decimal,
    Dot,
    Float1,
    Float2,
    Float3,
    Float4,
    Oct1,
    Oct2,
    Bin1,
    Bin2,
    Hex1,
    Hex2,
    String1,
    String2,
    String3,
    String4,
    Char1,
    Char2,
    Char3,
    Char4,
    Char5,
    Char6,
    Char7,
    Char8,
    Char9,
    Char10,
}

#[derive(Debug)]
struct Fsm {
    state: State,
    token: Option<Token>,
    inside_typedef_stmt: bool,
}

impl Fsm {
    fn new() -> Self {
        Fsm {
            state: State::Start,
            token: None,
            inside_typedef_stmt: false,
        }
    }

    fn step(self, input: char, typedef_names: &Vec<String>) -> Option<Self> {
        match self.state {
            State::Start => match input {
                '!' => Some(Fsm {
                    state: State::Bang,
                    token: Some(Token::Bang),
                }),
                '&' => Some(Fsm {
                    state: State::Ampersand,
                    token: Some(Token::Ampersand),
                }),
                '%' => Some(Fsm {
                    state: State::Percent,
                    token: Some(Token::Percent),
                }),
                '*' => Some(Fsm {
                    state: State::Asterisk,
                    token: Some(Token::Asterisk),
                }),
                '^' => Some(Fsm {
                    state: State::Caret,
                    token: Some(Token::Caret),
                }),
                '+' => Some(Fsm {
                    state: State::Plus,
                    token: Some(Token::Plus),
                }),
                '-' => Some(Fsm {
                    state: State::Minus,
                    token: Some(Token::Minus),
                }),
                '=' => Some(Fsm {
                    state: State::Eq,
                    token: Some(Token::Eq),
                }),
                '~' => Some(Fsm {
                    state: State::Tilde,
                    token: Some(Token::Tilde),
                }),
                '|' => Some(Fsm {
                    state: State::Bar,
                    token: Some(Token::Bar),
                }),
                '<' => Some(Fsm {
                    state: State::LessThan,
                    token: Some(Token::LessThan),
                }),
                '>' => Some(Fsm {
                    state: State::GreaterThan,
                    token: Some(Token::GreaterThan),
                }),
                '/' => Some(Fsm {
                    state: State::Slash,
                    token: Some(Token::Slash),
                }),
                '?' => Some(Fsm {
                    state: State::Question,
                    token: Some(Token::Question),
                }),
                '(' => Some(Fsm {
                    state: State::LeftParen,
                    token: Some(Token::LeftParen),
                }),
                ')' => Some(Fsm {
                    state: State::RightParen,
                    token: Some(Token::RightParen),
                }),
                '[' => Some(Fsm {
                    state: State::LeftSquare,
                    token: Some(Token::LeftSquare),
                }),
                ']' => Some(Fsm {
                    state: State::RightSquare,
                    token: Some(Token::RightSquare),
                }),
                '{' => Some(Fsm {
                    state: State::LeftCurly,
                    token: Some(Token::LeftCurly),
                }),
                '}' => Some(Fsm {
                    state: State::RightCurly,
                    token: Some(Token::RightCurly),
                }),
                ',' => Some(Fsm {
                    state: State::Comma,
                    token: Some(Token::Comma),
                }),
                ';' => Some(Fsm {
                    state: State::Semicolon,
                    token: Some(Token::Semicolon),
                }),
                ':' => Some(Fsm {
                    state: State::Colon,
                    token: Some(Token::Colon),
                }),
                '.' => Some(Fsm {
                    state: State::Dot,
                    token: Some(Token::Dot),
                }),
                '"' => Some(Fsm {
                    state: State::String1,
                    token: None,
                }),
                '\'' => Some(Fsm {
                    state: State::Char1,
                    token: None,
                }),
                c if c.is_ascii_alphabetic() => Some(Fsm {
                    state: State::Identifier,
                    token: Some(Token::Identifier(c.to_string())),
                }),
                '_' => Some(Fsm {
                    state: State::Identifier,
                    token: Some(Token::Identifier("_".to_owned())),
                }),
                '0' => Some(Fsm {
                    state: State::Oct1,
                    token: Some(Token::OctalConstant("0".to_owned())),
                }),
                c if c.is_ascii_digit() => Some(Fsm {
                    state: State::Decimal,
                    token: Some(Token::DecimalConstant(c.to_string())),
                }),
                _ => None,
            },
            State::Bang => match input {
                '=' => Some(Fsm {
                    state: State::BangEq,
                    token: Some(Token::BangEq),
                }),
                _ => None,
            },
            State::BangEq => None,
            State::Ampersand => match input {
                '=' => Some(Fsm {
                    state: State::AmpersandEq,
                    token: Some(Token::AmpersandEq),
                }),
                '&' => Some(Fsm {
                    state: State::DoubleAmpersand,
                    token: Some(Token::DoubleAmpersand),
                }),
                _ => None,
            },
            State::AmpersandEq => None,
            State::DoubleAmpersand => None,
            State::Percent => match input {
                '=' => Some(Fsm {
                    state: State::PercentEq,
                    token: Some(Token::PercentEq),
                }),
                _ => None,
            },
            State::PercentEq => None,
            State::Asterisk => match input {
                '=' => Some(Fsm {
                    state: State::AsteriskEq,
                    token: Some(Token::AsteriskEq),
                }),
                _ => None,
            },
            State::AsteriskEq => None,
            State::Caret => match input {
                '=' => Some(Fsm {
                    state: State::CaretEq,
                    token: Some(Token::CaretEq),
                }),
                _ => None,
            },
            State::CaretEq => None,
            State::Plus => match input {
                '=' => Some(Fsm {
                    state: State::PlusEq,
                    token: Some(Token::PlusEq),
                }),
                '+' => Some(Fsm {
                    state: State::DoublePlus,
                    token: Some(Token::DoublePlus),
                }),
                _ => None,
            },
            State::PlusEq => None,
            State::DoublePlus => None,
            State::Minus => match input {
                '=' => Some(Fsm {
                    state: State::MinusEq,
                    token: Some(Token::MinusEq),
                }),
                '-' => Some(Fsm {
                    state: State::DoubleMinus,
                    token: Some(Token::DoubleMinus),
                }),
                '>' => Some(Fsm {
                    state: State::Arrow,
                    token: Some(Token::Arrow),
                }),
                _ => None,
            },
            State::MinusEq => None,
            State::DoubleMinus => None,
            State::Arrow => None,
            State::Eq => match input {
                '=' => Some(Fsm {
                    state: State::DoubleEq,
                    token: Some(Token::DoubleEq),
                }),
                _ => None,
            },
            State::DoubleEq => None,
            State::Tilde => None,
            State::Bar => match input {
                '=' => Some(Fsm {
                    state: State::BarEq,
                    token: Some(Token::BarEq),
                }),
                '|' => Some(Fsm {
                    state: State::DoubleBar,
                    token: Some(Token::DoubleBar),
                }),
                _ => None,
            },
            State::BarEq => None,
            State::DoubleBar => None,
            State::LessThan => match input {
                '=' => Some(Fsm {
                    state: State::LessThanEq,
                    token: Some(Token::LessThanEq),
                }),
                '<' => Some(Fsm {
                    state: State::LeftShift,
                    token: Some(Token::LeftShift),
                }),
                _ => None,
            },
            State::LessThanEq => None,
            State::LeftShift => match input {
                '=' => Some(Fsm {
                    state: State::LeftShiftEq,
                    token: Some(Token::LeftShiftEq),
                }),
                _ => None,
            },
            State::LeftShiftEq => None,
            State::GreaterThan => match input {
                '=' => Some(Fsm {
                    state: State::GreaterThanEq,
                    token: Some(Token::GreaterThanEq),
                }),
                '>' => Some(Fsm {
                    state: State::RightShift,
                    token: Some(Token::RightShift),
                }),
                _ => None,
            },
            State::GreaterThanEq => None,
            State::RightShift => match input {
                '=' => Some(Fsm {
                    state: State::RightShiftEq,
                    token: Some(Token::RightShiftEq),
                }),
                _ => None,
            },
            State::RightShiftEq => None,
            State::Slash => match input {
                '=' => Some(Fsm {
                    state: State::SlashEq,
                    token: Some(Token::SlashEq),
                }),
                _ => None,
            },
            State::SlashEq => None,
            State::Question => None,
            State::LeftParen => None,
            State::RightParen => None,
            State::LeftSquare => None,
            State::RightSquare => None,
            State::LeftCurly => None,
            State::RightCurly => None,
            State::Comma => None,
            State::Semicolon => None,
            State::Colon => None,
            State::Identifier => {
                if !(input.is_ascii_alphabetic() || input == '_') {
                    return None;
                }
                if let Some(Token::Identifier(mut name)) = self.token {
                    name.push(input);
                    return Some(lex_identifier(name, typedef_names));
                }
                // shouldn't ever get here because the Identifier state should
                // always be paired with an Identifier token
                None
            }
            State::Keyword(mut name) => {
                if !(input.is_ascii_alphabetic() || input == '_') {
                    return None;
                }
                name.push(input);
                Some(lex_identifier(name, typedef_names))
            }
            State::TypedefName => {
                if !(input.is_ascii_alphabetic() || input == '_') {
                    return None;
                }
                if let Some(Token::TypedefName(mut name)) = self.token {
                    name.push(input);
                    return Some(lex_identifier(name, typedef_names));
                }
                // shouldn't ever get here because the Identifier state should
                // always be paired with an Identifier token
                None
            }
            State::Decimal => match input {
                '.' => match self.token {
                    Some(Token::DecimalConstant(mut s)) => {
                        s.push('.');
                        Some(Fsm {
                            state: State::Float1,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                'e' | 'E' => match self.token {
                    Some(Token::DecimalConstant(mut s)) => {
                        s.push('e');
                        Some(Fsm {
                            state: State::Float2,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                c if c.is_ascii_digit() => match self.token {
                    Some(Token::DecimalConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Decimal,
                            token: Some(Token::DecimalConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Dot => match input {
                c if c.is_ascii_digit() => {
                    let s = format!(".{}", c);
                    Some(Fsm {
                        state: State::Float1,
                        token: Some(Token::FloatingConstant(s)),
                    })
                }
                _ => None,
            },
            State::Float1 => match input {
                'e' | 'E' => match self.token {
                    Some(Token::FloatingConstant(mut s)) => {
                        s.push('e');
                        Some(Fsm {
                            state: State::Float2,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                c if c.is_ascii_digit() => match self.token {
                    Some(Token::FloatingConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Float1,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Float2 => match input {
                c @ '+' | c @ '-' => match self.token {
                    Some(Token::FloatingConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Float3,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                c if c.is_ascii_digit() => match self.token {
                    Some(Token::FloatingConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Float4,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Float3 | State::Float4 => match input {
                c if c.is_ascii_digit() => match self.token {
                    Some(Token::FloatingConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Float4,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Oct1 => match input {
                '.' => Some(Fsm {
                    state: State::Float1,
                    token: Some(Token::FloatingConstant("0.".to_owned())),
                }),
                'e' | 'E' => Some(Fsm {
                    state: State::Float2,
                    token: Some(Token::FloatingConstant("0e".to_owned())),
                }),
                'b' | 'B' => Some(Fsm {
                    state: State::Bin1,
                    token: Some(Token::BinaryConstant("0b".to_owned())),
                }),
                'x' | 'X' => Some(Fsm {
                    state: State::Hex1,
                    token: Some(Token::HexConstant("0x".to_owned())),
                }),
                c if c.is_digit(8) => {
                    let s = format!("0{}", c);
                    Some(Fsm {
                        state: State::Oct2,
                        token: Some(Token::OctalConstant(s)),
                    })
                }
                _ => None,
            },
            State::Oct2 => match input {
                '.' => match self.token {
                    Some(Token::OctalConstant(mut s)) => {
                        s.push('.');
                        Some(Fsm {
                            state: State::Float1,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                'e' | 'E' => match self.token {
                    Some(Token::OctalConstant(mut s)) => {
                        s.push('e');
                        Some(Fsm {
                            state: State::Float2,
                            token: Some(Token::FloatingConstant(s)),
                        })
                    }
                    _ => None,
                },
                c if c.is_digit(8) => match self.token {
                    Some(Token::OctalConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Oct2,
                            token: Some(Token::OctalConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Bin1 | State::Bin2 => match input {
                c if c.is_digit(2) => match self.token {
                    Some(Token::BinaryConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Bin2,
                            token: Some(Token::BinaryConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Hex1 | State::Hex2 => match input {
                c if c.is_ascii_hexdigit() => match self.token {
                    Some(Token::HexConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Hex2,
                            token: Some(Token::HexConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::String1 => match input {
                '\\' => Some(Fsm {
                    state: State::String2,
                    token: Some(Token::StringLiteral("\\".to_owned())),
                }),
                '"' => Some(Fsm {
                    state: State::String4,
                    token: Some(Token::StringLiteral("".to_owned())),
                }),
                c => Some(Fsm {
                    state: State::String3,
                    token: Some(Token::StringLiteral(c.to_string())),
                }),
            },
            State::String2 => match self.token {
                Some(Token::StringLiteral(mut s)) => {
                    s.push(input);
                    Some(Fsm {
                        state: State::String3,
                        token: Some(Token::StringLiteral(s)),
                    })
                }
                _ => None,
            },
            State::String3 => match input {
                '\\' => match self.token {
                    Some(Token::StringLiteral(s)) => Some(Fsm {
                        state: State::String2,
                        token: Some(Token::StringLiteral(format!("{}\\", s))),
                    }),
                    _ => None,
                },
                '"' => match self.token {
                    Some(Token::StringLiteral(s)) => Some(Fsm {
                        state: State::String4,
                        token: Some(Token::StringLiteral(s.to_owned())),
                    }),
                    _ => None,
                },
                c => match self.token {
                    Some(Token::StringLiteral(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::String3,
                            token: Some(Token::StringLiteral(s)),
                        })
                    }
                    _ => None,
                },
            },
            State::String4 => None,
            State::Char1 => match input {
                '\\' => Some(Fsm {
                    state: State::Char2,
                    token: Some(Token::CharConstant("\\".to_owned())),
                }),
                // empty char constant isn't allowed
                '\'' => None,
                c => Some(Fsm {
                    state: State::Char9,
                    token: Some(Token::CharConstant(c.to_string())),
                }),
            },
            State::Char2 => match input {
                c @ '\'' | c @ '"' | c @ '?' | c @ '\\' | c @ 'n' | c @ 'r' | c @ 't' => {
                    match self.token {
                        Some(Token::CharConstant(mut s)) => {
                            s.push(c);
                            Some(Fsm {
                                state: State::Char3,
                                token: Some(Token::CharConstant(s)),
                            })
                        }
                        _ => None,
                    }
                }
                'x' => match self.token {
                    Some(Token::CharConstant(mut s)) => {
                        s.push('x');
                        Some(Fsm {
                            state: State::Char4,
                            token: Some(Token::CharConstant(s)),
                        })
                    }
                    _ => None,
                },
                c if c.is_digit(8) => match self.token {
                    Some(Token::CharConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Char6,
                            token: Some(Token::CharConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Char3 | State::Char8 | State::Char9 => match input {
                '\'' => match self.token {
                    Some(Token::CharConstant(s)) => Some(Fsm {
                        state: State::Char10,
                        token: Some(Token::CharConstant(s.to_owned())),
                    }),
                    _ => None,
                },
                _ => None,
            },
            State::Char4 => match input {
                c if c.is_ascii_hexdigit() => match self.token {
                    Some(Token::CharConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Char5,
                            token: Some(Token::CharConstant(s)),
                        })
                    }
                    _ => None,
                },
                _ => None,
            },
            State::Char5 => match input {
                c if c.is_ascii_hexdigit() => match self.token {
                    Some(Token::CharConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Char5,
                            token: Some(Token::CharConstant(s)),
                        })
                    }
                    _ => None,
                },
                '\'' => match self.token {
                    Some(Token::CharConstant(s)) => Some(Fsm {
                        state: State::Char10,
                        token: Some(Token::CharConstant(s.to_owned())),
                    }),
                    _ => None,
                },
                _ => None,
            },
            State::Char6 => match input {
                c if c.is_digit(8) => match self.token {
                    Some(Token::CharConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Char7,
                            token: Some(Token::CharConstant(s)),
                        })
                    }
                    _ => None,
                },
                '\'' => match self.token {
                    Some(Token::CharConstant(s)) => Some(Fsm {
                        state: State::Char10,
                        token: Some(Token::CharConstant(s.to_owned())),
                    }),
                    _ => None,
                },
                _ => None,
            },
            State::Char7 => match input {
                c if c.is_digit(8) => match self.token {
                    Some(Token::CharConstant(mut s)) => {
                        s.push(c);
                        Some(Fsm {
                            state: State::Char8,
                            token: Some(Token::CharConstant(s)),
                        })
                    }
                    _ => None,
                },
                '\'' => match self.token {
                    Some(Token::CharConstant(s)) => Some(Fsm {
                        state: State::Char10,
                        token: Some(Token::CharConstant(s.to_owned())),
                    }),
                    _ => None,
                },
                _ => None,
            },
            State::Char10 => None,
        }
    }
}

fn lex_identifier(name: String, typedef_names: &Vec<String>, inside_typedef_stmt: bool) -> Fsm {
    // check if identifier name matches a keyword
    if let Some(keyword) = parse_keyword(&name) {
        return Fsm {
            state: State::Keyword(name),
            token: Some(keyword),
        };
    }
    // check if identifier name matches a typedef name
    if typedef_names.iter().any(|n| n == &name) && !inside_typedef_stmt {
        return Fsm {
            state: State::TypedefName,
            token: Some(Token::TypedefName(name)),
        };
    }
    // if no keyword or typedef name matches, keep lexing as an identifier
    Fsm {
        state: State::Identifier,
        token: Some(Token::Identifier(name)),
    }
}

fn parse_keyword(name: &String) -> Option<Token> {
    match name.as_str() {
        "auto" => Some(Token::Auto),
        "break" => Some(Token::Break),
        "case" => Some(Token::Case),
        "char" => Some(Token::Char),
        "const" => Some(Token::Const),
        "continue" => Some(Token::Continue),
        "default" => Some(Token::Default),
        "do" => Some(Token::Do),
        "double" => Some(Token::Double),
        "else" => Some(Token::Else),
        "enum" => Some(Token::Enum),
        "extern" => Some(Token::Extern),
        "float" => Some(Token::Float),
        "for" => Some(Token::For),
        "goto" => Some(Token::Goto),
        "if" => Some(Token::If),
        "int" => Some(Token::Int),
        "long" => Some(Token::Long),
        "register" => Some(Token::Register),
        "return" => Some(Token::Return),
        "short" => Some(Token::Short),
        "signed" => Some(Token::Signed),
        "sizeof" => Some(Token::Sizeof),
        "static" => Some(Token::Static),
        "struct" => Some(Token::Struct),
        "switch" => Some(Token::Switch),
        "typedef" => Some(Token::Typedef),
        "union" => Some(Token::Union),
        "unsigned" => Some(Token::Unsigned),
        "void" => Some(Token::Void),
        "volatile" => Some(Token::Volatile),
        "while" => Some(Token::While),
        _ => None,
    }
}
