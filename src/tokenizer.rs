/// Tokenizer.
///
/// Lazily pulls a token from a stream.

use crate::parser::{Literal, LiteralType};

pub struct Tokenizer {
    pub string: LiteralType,
    cursor: usize,
}

impl Tokenizer {
    /// Intializes a string.
    pub fn new(string: String) -> Self {
        Self {
            string: LiteralType::Type(string),
            cursor: 0
        }
    }

    /// Obtains next token.
    pub fn get_next_token(&mut self) -> Option<Literal> {
        if !self.has_more_tokens() {
            return None;
        }

        match self.string.clone() {
            LiteralType::Type(string) => {
                let string = string.chars().collect::<Vec<char>>();

                // Numbers
                if string[self.cursor].is_digit(10) {
                    let mut num = String::new();
                    while self.cursor < string.len() && string[self.cursor].is_digit(10) {
                        num.push(string[self.cursor]);
                        self.cursor += 1;
                    }

                    return Some(Literal {
                        literal_type: LiteralType::Type(String::from("NUMBER")),
                        value: num,
                    })
                }

                // Strings
                if string[self.cursor] == '"' {
                    let mut s = String::new();
                    self.cursor += 1; // skip first '"'
                    while string[self.cursor] != '"' && !self.is_eof() {
                        s.push(string[self.cursor]);
                        self.cursor += 1;
                    }
                    self.cursor += 1; // skip last '"'

                    return Some(Literal {
                        literal_type: LiteralType::Type(String::from("STRING")),
                        value: s,
                    })
                }
            }
        }

        None
    }

    /// Check if there exists more tokens.
    fn has_more_tokens(&self) -> bool {
        match self.string {
            LiteralType::Type(ref string) => {
                self.cursor < string.len()
            }
        }
    }

    // Check if tokenizer reached EOF.
    fn is_eof(&self) -> bool {
        match self.string {
            LiteralType::Type(ref string) => {
                self.cursor == string.len()
            }
        }
    }
}