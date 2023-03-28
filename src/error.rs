use crate::token::Token;
use crate::token_type::TokenType;

#[derive(Debug)]
pub struct LoxError {
    token: Option<Token>,
    line: usize,
    message: String,
}

impl LoxError {
    pub fn error(line: usize, message: String) -> LoxError {
        let err = LoxError {
            token: None,
            line,
            message,
        };
        err.report("".to_string());
        err
    }

    pub fn parse_error(token: Token, message: String) -> LoxError {
        let err = LoxError {
            token: Some(token.dup()),
            line: token.line.clone(),
            message,
        };
        err.report("".to_string());
        err
    }

    pub fn report(&self, loc: String) {
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
