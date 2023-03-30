use crate::token::Token;
use crate::token_type::TokenType;

#[derive(Debug, PartialEq)]
pub struct LoxError {
    token: Option<Token>,
    line: usize,
    message: String,
}

impl LoxError {
    pub fn error(line: usize, message: &str) -> LoxError {
        // err.report("");
        LoxError {
            token: None,
            line,
            message: message.to_string(),
        }
    }

    pub fn parse_error(token: Token, message: &str) -> LoxError {
        // err.report("");
        LoxError {
            token: Some(token.dup()),
            line: token.line,
            message: message.to_string(),
        }
    }

    pub fn runtime_error(token: Token, message: &str) -> LoxError {
        // err.report("");
        LoxError {
            token: Some(token.dup()),
            line: token.line,
            message: message.to_string(),
        }
    }

    pub fn report(&self, loc: &str) {
        if let Some(token) = &self.token {
            if token.is(TokenType::Eof) {
                eprintln!("{} at end {}", token.line, self.message);
            } else {
                eprintln!("{} at '{}' {}", token.line, token.lexeme, self.message);
            }
        } else {
            eprintln!("[line {}] Error{}: {}", self.line, loc, self.message)
        }
    }
}
