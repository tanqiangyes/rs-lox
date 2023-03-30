use crate::error::LoxError;
use crate::object::Object;
use crate::token::*;
use crate::token_type::*;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
        let mut had_error: Option<LoxError> = None;
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => {}
                Err(e) => {
                    e.report("");
                    had_error = Some(e);
                }
            }
        }
        self.tokens.push(Token::eof(self.line));
        if let Some(e) = had_error {
            Err(e)
        } else {
            Ok(&self.tokens)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::SemiColon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let tok = if self.is_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(tok);
            }
            '=' => {
                let tok = if self.is_match('=') {
                    TokenType::Equal
                } else {
                    TokenType::Assign
                };
                self.add_token(tok);
            }
            '>' => {
                let tok = if self.is_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(tok);
            }
            '<' => {
                let tok = if self.is_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(tok);
            }
            '/' => {
                if self.is_match('/') {
                    let start = self.current - 1;
                    let start_line = self.line;
                    while let Some(ch) = self.peek() {
                        if ch != '\n' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.print_comment(start, start_line);
                } else if self.is_match('*') {
                    let start = self.current - 1;
                    let start_line = self.line;
                    self.advance();
                    self.scan_comment(start, start_line)?;
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string()?;
            }
            '0'..='9' => {
                self.number();
            }
            _ => {
                if c.is_alphanumeric() || c == '_' {
                    self.identifier();
                } else {
                    return Err(LoxError::error(self.line, "Unknown token type"));
                }
            }
        }
        Ok(())
    }
    // comment like /*  */
    fn scan_comment(&mut self, start: usize, start_line: usize) -> Result<(), LoxError> {
        while !self.is_match('*') && !self.is_at_end() {
            self.advance();
            if self.peek() == Some('\n') {
                self.line += 1;
            }
        }

        if !self.is_at_end() {
            return if self.is_match('*') && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                self.print_comment(start, start_line);
                Ok(())
            } else {
                self.advance();
                self.scan_comment(start, start_line)
            };
        }
        Err(LoxError::error(self.line, "UnClosed comment."))
    }

    fn print_comment(&self, start: usize, _start_line: usize) {
        println!(
            "Comment: {}",
            self.source[start..self.current].iter().collect::<String>()
        );
    }

    fn identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let val: String = self.source[self.start..self.current].iter().collect();
        if let Some(ttype) = Scanner::keywords(val.as_str()) {
            self.add_token(ttype)
        } else {
            self.add_token(TokenType::Identifier)
        }
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if Some('.') == self.peek() && Scanner::is_digit(self.peek_next()) {
            self.advance();
            while Scanner::is_digit(self.peek()) {
                self.advance();
            }
        }
        let val: String = self.source[self.start..self.current].iter().collect();
        let num: f64 = val.parse().unwrap();
        self.add_token_object(TokenType::Number, Some(Object::Num(num)));
        // Ok(())
    }

    fn is_digit(ch: Option<char>) -> bool {
        if let Some(ch) = ch {
            ch.is_ascii_digit()
        } else {
            false
        }
    }

    fn is_alpha_numeric(ch: Option<char>) -> bool {
        if let Some(ch) = ch {
            ch.is_ascii_alphanumeric()
        } else {
            false
        }
    }

    fn string(&mut self) -> Result<(), LoxError> {
        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            }
            if ch == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(LoxError::error(self.line, "Unterminated string."));
        }

        self.advance();

        let val: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token_object(TokenType::String, Some(Object::Str(val)));
        Ok(())
    }

    fn advance(&mut self) -> char {
        let result = *self.source.get(self.current).unwrap();
        self.current += 1;
        result
    }

    fn add_token(&mut self, ttype: TokenType) {
        self.add_token_object(ttype, None);
    }

    fn add_token_object(&mut self, ttype: TokenType, literal: Option<Object>) {
        let lexeme = self.source[self.start..self.current].iter().collect();
        self.tokens
            .push(Token::new(ttype, lexeme, literal, self.line))
    }

    fn is_match(&mut self, expected: char) -> bool {
        if let Some(ch) = self.source.get(self.current) {
            if *ch != expected {
                return false;
            }
            return true;
        }
        false
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).copied()
    }

    fn keywords(check: &str) -> Option<TokenType> {
        match check {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}
