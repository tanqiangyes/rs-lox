use crate::error::LoxResult;
use crate::expr::{
    AssignExpr, BinaryExpr, Expr, GroupingExpr, LiteralExpr, LogicalExpr, UnaryExpr, VariableExpr,
};
use crate::object::Object;
use crate::stmt::*;
use crate::token::*;
use crate::token_type::*;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    has_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &[Token]) -> Parser {
        Parser {
            tokens,
            current: 0,
            has_error: false,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxResult> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let dec = self.declaration();
            match dec {
                Ok(res) => {
                    statements.push(res);
                }
                Err(e) => {
                    e.report("");
                }
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, LoxResult> {
        let result = if self.is_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxResult> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.is_match(&[TokenType::Assign]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(VarStmt { name, initializer }))
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'while'.")?;
        let body = self.statement()?;

        Ok(Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
        }))
    }

    fn statement(&mut self) -> Result<Stmt, LoxResult> {
        if self.is_match(&[TokenType::Break]) {
            let token = self.peek().dup();
            self.consume(TokenType::SemiColon, "Expect ';' after 'break'.")?;
            return Ok(Stmt::Break(BreakStmt { token }));
        }
        if self.is_match(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.is_match(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.is_match(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.is_match(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.is_match(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(BlockStmt {
                statements: self.block()?,
            }));
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.is_match(&[TokenType::SemiColon]) {
            None
        } else if self.is_match(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(TokenType::SemiColon) {
            Expr::Literal(LiteralExpr {
                value: Some(Object::Bool(true)),
            })
        } else {
            self.expression()?
        };

        self.consume(TokenType::SemiColon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(TokenType::RightParen, "Expect ')' after loop condition.")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(BlockStmt {
                statements: vec![
                    body,
                    Stmt::Expression(ExpressionStmt {
                        expression: increment,
                    }),
                ],
            });
        }

        body = Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
        });

        if let Some(init) = initializer {
            body = Stmt::Block(BlockStmt {
                statements: vec![init, body],
            });
        }
        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.is_match(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(IfStmt {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LoxResult> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxResult> {
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(PrintStmt { expression: value }))
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxResult> {
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(ExpressionStmt { expression: value }))
    }

    fn assignment(&mut self) -> Result<Expr, LoxResult> {
        let expr = self.or()?;
        if self.is_match(&[TokenType::Assign]) {
            let equals = self.previous().dup();
            let value = self.assignment()?;

            if let Expr::Variable(expr) = expr {
                return Ok(Expr::Assign(AssignExpr {
                    name: expr.name.dup(),
                    value: Box::new(value),
                }));
            }
            self.error(equals, "Invalid assignment target.");
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.and()?;

        while self.is_match(&[TokenType::Or]) {
            let operator = self.previous().dup();
            let right = self.and()?;
            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.equality()?;

        while self.is_match(&[TokenType::And]) {
            let operator = self.previous().dup();
            let right = self.equality()?;
            expr = Expr::Logical(LogicalExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, LoxResult> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::Equal]) {
            let operator = self.previous().dup();
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.term()?;
        while self.is_match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().dup();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.factor()?;
        while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().dup();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.unary()?;
        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().dup();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxResult> {
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().dup();
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, LoxResult> {
        if self.is_match(&[TokenType::False]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Object::Bool(false)),
            }));
        };
        if self.is_match(&[TokenType::True]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Object::Bool(true)),
            }));
        };
        if self.is_match(&[TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: Some(Object::Nil),
            }));
        };

        if self.is_match(&[TokenType::String, TokenType::Number]) {
            return Ok(Expr::Literal(LiteralExpr {
                value: self.previous().dup().literal,
            }));
        }

        if self.is_match(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(VariableExpr {
                name: self.previous().dup(),
            }));
        }

        if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(GroupingExpr {
                expression: Box::new(expr),
            }));
        }
        let peek = self.peek().dup();
        Err(LoxResult::parse_error(peek, "Expect expression."))
    }

    fn consume(&mut self, ttype: TokenType, message: &str) -> Result<Token, LoxResult> {
        if self.check(ttype) {
            Ok(self.advance().dup())
        } else {
            Err(self.error(self.peek().dup(), message))
        }
    }

    fn error(&mut self, token: Token, message: &str) -> LoxResult {
        self.has_error = true;
        LoxResult::parse_error(token, message)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().is(TokenType::SemiColon) {
                return;
            }

            if matches!(
                self.peek().token_type(),
                TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return
            ) {
                return;
            }
            self.advance();
        }
    }

    fn is_match(&mut self, types: &[TokenType]) -> bool {
        for &t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, ttype: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().is(ttype)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().is(TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    pub fn success(&self) -> bool {
        !self.has_error
    }
}
