use crate::error::LoxResult;
use crate::expr::{
    AssignExpr, BinaryExpr, CallExpr, Expr, ExprVisitor, GetExpr, GroupingExpr, LiteralExpr,
    LogicalExpr, SetExpr, SuperExpr, ThisExpr, UnaryExpr, VariableExpr,
};
use crate::interpreter::Interpreter;
use crate::stmt::{
    BlockStmt, BreakStmt, ClassStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt,
    Stmt, StmtVisitor, VarStmt, WhileStmt,
};
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
    has_error: RefCell<bool>,
    current_fun_type: RefCell<FunctionType>,
    current_class_type: RefCell<ClassType>,
    in_while: RefCell<bool>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum ClassType {
    None,
    Class,
    SubClass,
}

impl<'a> StmtVisitor<()> for Resolver<'a> {
    fn visit_block_stmt(&self, _wrapper: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_class_stmt(&self, _wrapper: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        let enclosing_class = self.current_class_type.replace(ClassType::Class);

        self.declare(&stmt.name);
        self.define(&stmt.name);

        if let Some(superclass) = stmt.superclass.clone() {
            if let Expr::Variable(sup) = superclass.clone().as_ref() {
                if stmt.name.as_string().eq(&sup.name.as_string()) {
                    self.error(stmt.name.dup(), "A class cannot inherit from itself");
                } else {
                    self.current_class_type.replace(ClassType::SubClass);
                    self.resolve_expr(superclass.clone())?;
                    self.begin_scope();
                    self.scopes
                        .borrow()
                        .last()
                        .unwrap()
                        .borrow_mut()
                        .insert("super".to_string(), true);
                }
            } else {
                self.error(stmt.name.dup(), "Get superclass name failed.");
            }
        }

        self.begin_scope();
        self.scopes
            .borrow()
            .last()
            .unwrap()
            .borrow_mut()
            .insert("this".to_string(), true);

        for method in stmt.methods.deref() {
            if let Stmt::Function(method) = method.deref() {
                let declaration = if method.name.as_string() == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(method, declaration)?;
            } else {
                return Err(LoxResult::runtime_error(
                    stmt.name.dup(),
                    "Class method did not resolve into a function statement.",
                ));
            }
        }

        self.end_scope();
        if stmt.superclass.is_some() {
            self.end_scope();
        }
        self.current_class_type.replace(enclosing_class);
        Ok(())
    }

    fn visit_break_stmt(&self, _wrapper: Rc<Stmt>, stmt: &BreakStmt) -> Result<(), LoxResult> {
        if !*self.in_while.borrow() {
            self.error(
                stmt.token.dup(),
                "Break statement outside of a for/while loop",
            )
        }
        Ok(())
    }

    fn visit_expression_stmt(
        &self,
        _wrapper: Rc<Stmt>,
        stmt: &ExpressionStmt,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_function_stmt(
        &self,
        _wrapper: Rc<Stmt>,
        stmt: &FunctionStmt,
    ) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if_stmt(&self, _wrapper: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.then_branch.clone())?;
        if let Some(else_branch) = stmt.else_branch.clone() {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&self, _wrapper: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_return_stmt(&self, _wrapper: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if *self.current_fun_type.borrow() == FunctionType::None {
            self.error(stmt.keyword.dup(), "Can't return from top-level code.")
        }
        if let Some(value) = stmt.value.clone() {
            if *self.current_fun_type.borrow() == FunctionType::Initializer {
                self.error(
                    stmt.keyword.dup(),
                    "Can't return a value from an initializer.",
                )
            }
            self.resolve_expr(value)?;
        }
        Ok(())
    }

    fn visit_var_stmt(&self, _wrapper: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        if let Some(init) = stmt.initializer.clone() {
            self.resolve_expr(init)?;
        }
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_while_stmt(&self, _wrapper: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        let previous_nesting = self.in_while.replace(true);
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.body.clone())?;
        self.in_while.replace(previous_nesting);
        Ok(())
    }
}

impl<'a> ExprVisitor<()> for Resolver<'a> {
    fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_local(wrapper, &expr.name);
        Ok(())
    }

    fn visit_binary_expr(&self, _wrapper: Rc<Expr>, expr: &BinaryExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_call_expr(&self, _wrapper: Rc<Expr>, expr: &CallExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.callee.clone())?;
        for argument in expr.arguments.iter() {
            self.resolve_expr(argument.clone())?;
        }
        Ok(())
    }

    fn visit_get_expr(&self, _wrapper: Rc<Expr>, expr: &GetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.object.clone())?;
        Ok(())
    }

    fn visit_grouping_expr(
        &self,
        _wrapper: Rc<Expr>,
        expr: &GroupingExpr,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.expression.clone())?;
        Ok(())
    }

    fn visit_literal_expr(&self, _wrapper: Rc<Expr>, _expr: &LiteralExpr) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_logical_expr(&self, _wrapper: Rc<Expr>, expr: &LogicalExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_set_expr(&self, _wrapper: Rc<Expr>, expr: &SetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_expr(expr.object.clone())?;
        Ok(())
    }

    fn visit_super_expr(&self, wrapper: Rc<Expr>, expr: &SuperExpr) -> Result<(), LoxResult> {
        if self.current_class_type.borrow().clone() == ClassType::None {
            self.error(expr.keyword.dup(), "Can`t use super outside of a class.");
        } else if self.current_class_type.borrow().clone() != ClassType::SubClass {
            self.error(
                expr.keyword.dup(),
                "Can`t use 'super' in a class whit no superclass.",
            )
        }
        self.resolve_local(wrapper, &expr.keyword);
        Ok(())
    }

    fn visit_this_expr(&self, wrapper: Rc<Expr>, expr: &ThisExpr) -> Result<(), LoxResult> {
        if *self.current_class_type.borrow() == ClassType::None {
            self.error(expr.keyword.dup(), "Can't use 'this' outside a class.");
        }
        self.resolve_local(wrapper, &expr.keyword);
        Ok(())
    }

    fn visit_unary_expr(&self, _wrapper: Rc<Expr>, expr: &UnaryExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_variable_expr(&self, wrapper: Rc<Expr>, expr: &VariableExpr) -> Result<(), LoxResult> {
        if !self.scopes.borrow().is_empty()
            && self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.as_string())
                == Some(&false)
        {
            Err(LoxResult::runtime_error(
                expr.name.dup(),
                "Can`t read local variable in its own initializer.",
            ))
        } else {
            self.resolve_local(wrapper, &expr.name);
            Ok(())
        }
    }
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
            has_error: RefCell::new(false),
            current_fun_type: RefCell::new(FunctionType::None),
            current_class_type: RefCell::new(ClassType::None),
            in_while: RefCell::new(false),
        }
    }

    pub fn resolve(&self, statements: &Rc<Vec<Rc<Stmt>>>) -> Result<(), LoxResult> {
        for statement in statements.iter() {
            self.resolve_stmt(statement.clone())?;
        }
        Ok(())
    }

    fn resolve_stmt(&self, statement: Rc<Stmt>) -> Result<(), LoxResult> {
        statement.accept(statement.clone(), self)
    }

    fn resolve_expr(&self, expr: Rc<Expr>) -> Result<(), LoxResult> {
        expr.accept(expr.clone(), self)
    }

    fn resolve_function(
        &self,
        function: &FunctionStmt,
        func_type: FunctionType,
    ) -> Result<(), LoxResult> {
        let enclosing_func = self.current_fun_type.replace(func_type);
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }

        self.resolve(&function.body)?;
        self.end_scope();
        self.current_fun_type.replace(enclosing_func);
        Ok(())
    }

    fn begin_scope(&self) {
        self.scopes.borrow_mut().push(RefCell::new(HashMap::new()))
    }

    fn end_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    fn declare(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow().last() {
            if scope.borrow().contains_key(&name.as_string()) {
                self.error(
                    name.dup(),
                    "Already a variable with this name in this scope.",
                );
            }
            scope.borrow_mut().insert(name.as_string(), false);
        }
    }

    fn define(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow().last() {
            scope.borrow_mut().insert(name.as_string(), true);
        }
    }

    fn resolve_local(&self, expr: Rc<Expr>, name: &Token) {
        for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
            if map.borrow().contains_key(&name.as_string()) {
                self.interpreter.resolve(expr, scope);
                return;
            }
        }
    }

    fn error(&self, token: Token, message: &str) {
        self.has_error.replace(true);
        let lox_err = LoxResult::parse_error(token, message);
        lox_err.report()
    }

    pub fn success(&self) -> bool {
        !*self.has_error.borrow()
    }
}
