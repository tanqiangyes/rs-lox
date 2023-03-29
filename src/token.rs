use crate::object::Object;
use crate::token_type::*;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub(crate) ttype: TokenType,
    pub(crate) lexeme: String,
    pub(crate) literal: Option<Object>,
    pub(crate) line: usize,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Token {
        Token {
            ttype,
            lexeme,
            literal,
            line,
        }
    }
    pub fn is(&self, ttype: TokenType) -> bool {
        self.ttype == ttype
    }

    pub fn token_type(&self) -> TokenType {
        self.ttype
    }

    pub fn as_string(&self) -> String {
        self.lexeme.clone()
    }

    pub fn dup(&self) -> Token {
        Token {
            ttype: self.ttype,
            lexeme: self.lexeme.to_string(),
            literal: self.literal.clone(),
            line: self.line,
        }
    }

    pub fn eof(line: usize) -> Token {
        Token {
            ttype: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:?} {} {}",
            self.line,
            self.ttype,
            self.lexeme,
            if let Some(literal) = &self.literal {
                literal.to_string()
            } else {
                "None".to_string()
            }
        )
    }
}
