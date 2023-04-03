use crate::object::Object;
use crate::token::Token;
use crate::token_type::TokenType;

#[derive(Debug, PartialEq)]
pub enum LoxResult {
    ParseError { token: Token, message: String },
    RuntimeError { token: Token, message: String },
    Error { line: usize, message: String },
    SystemError { message: String },
    ReturnValue { value: Object },
    Break,
    Fail,
}

/// we will report the error in right place.
impl LoxResult {
    pub fn error(line: usize, message: &str) -> LoxResult {
        LoxResult::Error {
            line,
            message: message.to_string(),
        }
    }

    pub fn parse_error(token: Token, message: &str) -> LoxResult {
        LoxResult::ParseError {
            token: token.dup(),
            message: message.to_string(),
        }
    }

    pub fn runtime_error(token: Token, message: &str) -> LoxResult {
        LoxResult::RuntimeError {
            token: token.dup(),
            message: message.to_string(),
        }
    }

    pub fn report(&self, loc: &str) {
        match self {
            LoxResult::ParseError { token, message } => {
                if token.is(TokenType::Eof) {
                    eprintln!("[line {}] Error at end: {}", token.line, message);
                } else {
                    eprintln!(
                        "[line {}] Error at '{}': {}",
                        token.line,
                        token.as_string(),
                        message
                    );
                }
            }
            LoxResult::RuntimeError { token, message } => {
                if token.is(TokenType::Eof) {
                    eprintln!("[line {}] Error at end: {}", token.line, message);
                } else {
                    eprintln!("{}\n[line {}]", message, token.line);
                }
            }
            LoxResult::Error { line, message } => {
                eprintln!("[line {}] Error{}: {}", line, loc, message);
            }
            LoxResult::Break | LoxResult::ReturnValue { .. } => {}
            LoxResult::Fail => {
                panic!("should not get here")
            }
            _ => {}
        };
    }
}
