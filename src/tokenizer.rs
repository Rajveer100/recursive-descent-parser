/// Tokenizer.
///
/// Lazily pulls a token from a stream.

use crate::parser::{Literal, LiteralType, LiteralValue};
use regex::Regex;

const SPEC: [(&str, Option<&str>); 6] = [
    // Skip whitespaces
    (r"^\s+", None),

    // Skip single-line comments
    (r"^\/\/.*", None),

    // Skip multi-line comments
    (r"^\/\*[\s\S]*?\*\/", None),

    // Symbols, Delimiters
    (r"^;", Some(";")),

    // Numbers
    (r"^\d+", Some("NUMBER")),

    // Strings
    (r#"^"[^"]*""#, Some("STRING"))
];

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
                let string = string[self.cursor..].to_string();
                for &(reg, token_type) in SPEC.iter() {
                    if let Some(token_val) = self.get_match(Regex::new(reg).unwrap(),
                                                            string.as_str()) {
                        if token_type.is_none() {
                            return self.get_next_token();
                        }
                        return Some(Literal {
                            literal_type: LiteralType::Type(token_type?.to_string()),
                            value: Box::new(LiteralValue::Value(token_val)),
                        })
                    }
                }
                panic!("Unexpected token: {string}");
            }
        }
    }

    /// Matches a token for given regex.
    fn get_match(&mut self, reg: Regex, string: &str) -> Option<String> {
        if let Some(mat) = reg.captures(string) {
            self.cursor += mat.get(0).unwrap().len();
            return Some(mat.get(0).unwrap().as_str().to_string());
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
}