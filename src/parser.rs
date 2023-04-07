use crate::error::LoxResult;
use crate::expr::{
    AssignExpr, BinaryExpr, CallExpr, Expr, GetExpr, GroupingExpr, LiteralExpr, LogicalExpr,
    SetExpr, SuperExpr, ThisExpr, UnaryExpr, VariableExpr,
};
use crate::object::Object;
use crate::stmt::*;
use crate::token::*;
use crate::token_type::*;
use std::rc::Rc;

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

    pub fn parse(&mut self) -> Result<Vec<Rc<Stmt>>, LoxResult> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let dec = self.declaration();
            match dec {
                Ok(res) => {
                    statements.push(Rc::new(res));
                }
                Err(e) => {
                    e.report();
                }
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, LoxResult> {
        let result = if self.is_match(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.is_match(&[TokenType::Fun]) {
            self.function("function")
        } else if self.is_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn class_declaration(&mut self) -> Result<Stmt, LoxResult> {
        let name = self.consume(TokenType::Identifier, "Expect a class name.")?;

        let superclass = if self.is_match(&[TokenType::Less]) {
            self.consume(TokenType::Identifier, "Expect super class name.")?;
            Some(Rc::new(Expr::Variable(Rc::new(VariableExpr {
                name: self.previous().dup(),
            }))))
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(Rc::new(self.function("method")?));
        }
        self.consume(TokenType::RightBrace, "Expect '}' before class body.")?;
        Ok(Stmt::Class(Rc::new(ClassStmt {
            name,
            superclass,
            methods: Rc::new(methods),
        })))
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxResult> {
        let keyword = self.previous().dup();
        let value = if self.check(TokenType::SemiColon) {
            None
        } else {
            Some(Rc::new(self.expression()?))
        };
        self.consume(TokenType::SemiColon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(Rc::new(ReturnStmt { keyword, value })))
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxResult> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.is_match(&[TokenType::Assign]) {
            Some(Rc::new(self.expression()?))
        } else {
            None
        };
        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(Rc::new(VarStmt { name, initializer })))
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = Rc::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expect ')' after 'while'.")?;
        let body = self.statement()?;

        Ok(Stmt::While(Rc::new(WhileStmt {
            condition,
            body: Rc::new(body),
        })))
    }

    fn statement(&mut self) -> Result<Stmt, LoxResult> {
        if self.is_match(&[TokenType::Break]) {
            let token = self.previous().dup();
            self.consume(TokenType::SemiColon, "Expect ';' after 'break'.")?;
            return Ok(Stmt::Break(Rc::new(BreakStmt { token })));
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

        if self.is_match(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.is_match(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.is_match(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(Rc::new(BlockStmt {
                statements: Rc::new(self.block()?),
            })));
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
            Rc::new(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Bool(true)),
            })))
        } else {
            Rc::new(self.expression()?)
        };

        self.consume(TokenType::SemiColon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(Rc::new(self.expression()?))
        };

        self.consume(TokenType::RightParen, "Expect ')' after loop condition.")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(Rc::new(BlockStmt {
                statements: Rc::new(vec![
                    Rc::new(body),
                    Rc::new(Stmt::Expression(Rc::new(ExpressionStmt {
                        expression: increment,
                    }))),
                ]),
            }));
        }

        body = Stmt::While(Rc::new(WhileStmt {
            condition,
            body: Rc::new(body),
        }));

        if let Some(init) = initializer {
            body = Stmt::Block(Rc::new(BlockStmt {
                statements: Rc::new(vec![Rc::new(init), Rc::new(body)]),
            }));
        }
        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = Rc::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.")?;
        let then_branch = Rc::new(self.statement()?);
        let else_branch = if self.is_match(&[TokenType::Else]) {
            Some(Rc::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(Rc::new(IfStmt {
            condition,
            then_branch,
            else_branch,
        })))
    }

    fn block(&mut self) -> Result<Vec<Rc<Stmt>>, LoxResult> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(Rc::new(self.declaration()?));
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxResult> {
        let value = Rc::new(self.expression()?);
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Rc::new(PrintStmt { expression: value })))
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxResult> {
        let value = Rc::new(self.expression()?);
        self.consume(TokenType::SemiColon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(Rc::new(ExpressionStmt {
            expression: value,
        })))
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, LoxResult> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {kind} name."))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {kind} name."),
        )?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            params.push(self.consume(TokenType::Identifier, "Expect paramter name.")?);
            while self.is_match(&[TokenType::Comma]) {
                if params.len() >= 255 && !self.has_error {
                    self.error(self.peek().dup(), "Can`t have more than 255 parameters.");
                    self.has_error = true;
                }

                params.push(self.consume(TokenType::Identifier, "Expect paramter name.")?);
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' after {kind} body."),
        )?;
        let body = Rc::new(self.block()?);
        Ok(Stmt::Function(Rc::new(FunctionStmt {
            name,
            params: Rc::new(params),
            body,
        })))
    }

    fn assignment(&mut self) -> Result<Expr, LoxResult> {
        let expr = self.or()?;
        if self.is_match(&[TokenType::Assign]) {
            let equals = self.previous().dup();
            let value = self.assignment()?;

            if let Expr::Variable(expr) = expr {
                return Ok(Expr::Assign(Rc::new(AssignExpr {
                    name: expr.name.dup(),
                    value: Rc::new(value),
                })));
            } else if let Expr::Get(get) = expr {
                return Ok(Expr::Set(Rc::new(SetExpr {
                    object: get.object.clone(),
                    name: get.name.dup(),
                    value: Rc::new(value),
                })));
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
            expr = Expr::Logical(Rc::new(LogicalExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.equality()?;

        while self.is_match(&[TokenType::And]) {
            let operator = self.previous().dup();
            let right = self.equality()?;
            expr = Expr::Logical(Rc::new(LogicalExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
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
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
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
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.factor()?;
        while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().dup();
            let right = self.factor()?;
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.unary()?;
        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().dup();
            let right = self.unary()?;
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxResult> {
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().dup();
            let right = self.unary()?;
            return Ok(Expr::Unary(Rc::new(UnaryExpr {
                operator,
                right: Rc::new(right),
            })));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.primary()?;
        loop {
            if self.is_match(&[TokenType::LeftParen]) {
                expr = self.finish_call(Rc::new(expr))?;
            } else if self.is_match(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get(Rc::new(GetExpr {
                    object: Rc::new(expr),
                    name,
                }));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Rc<Expr>) -> Result<Expr, LoxResult> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            arguments.push(Rc::new(self.expression()?));
            while self.is_match(&[TokenType::Comma]) {
                if arguments.len() >= 255 {
                    return Err(
                        self.error(self.peek().dup(), "Can`t have more than 255 arguments.")
                    );
                }
                arguments.push(Rc::new(self.expression()?));
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(Expr::Call(Rc::new(CallExpr {
            callee: Rc::clone(&callee),
            paren,
            arguments,
        })))
    }

    fn primary(&mut self) -> Result<Expr, LoxResult> {
        if self.is_match(&[TokenType::False]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Bool(false)),
            })));
        }
        if self.is_match(&[TokenType::True]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Bool(true)),
            })));
        }
        if self.is_match(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Nil),
            })));
        }

        if self.is_match(&[TokenType::String, TokenType::Number]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: self.previous().dup().literal,
            })));
        }

        if self.is_match(&[TokenType::Super]) {
            let keyword = self.previous().dup();
            self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenType::Dot, "Expect superclass method name.")?;
            return Ok(Expr::Super(Rc::new(SuperExpr { keyword, method })));
        }

        if self.is_match(&[TokenType::This]) {
            return Ok(Expr::This(Rc::new(ThisExpr {
                keyword: self.previous().dup(),
            })));
        }

        if self.is_match(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(Rc::new(VariableExpr {
                name: self.previous().dup(),
            })));
        }

        if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Rc::new(GroupingExpr {
                expression: Rc::new(expr),
            })));
        }
        Err(LoxResult::parse_error(
            self.peek().dup(),
            "Expect expression.",
        ))
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
