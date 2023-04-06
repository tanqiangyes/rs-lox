use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::lox_function::LoxFunction;
use crate::native_functions::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::Token;
use crate::token_type::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: RefCell<Rc<RefCell<Environment>>>,
    locals: RefCell<HashMap<Rc<Expr>, usize>>,
}

impl StmtVisitor<()> for Interpreter {
    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        let e = Environment::new_with_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)
    }

    fn visit_break_stmt(&self, _: Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Err(LoxResult::Break)
    }

    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.evaluate(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        let function = LoxFunction::new(stmt, &self.environment.borrow());
        self.environment.borrow().borrow_mut().define(
            stmt.name.as_string(),
            Object::Func(Callable {
                func: Rc::new(function),
            }),
        );
        Ok(())
    }

    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        if self.is_truthy(self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.then_branch.clone())
        } else if let Some(else_branch) = stmt.else_branch.clone() {
            self.execute(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        let value = self.evaluate(stmt.expression.clone())?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if let Some(value) = &stmt.value {
            Err(LoxResult::return_value(self.evaluate(value.clone())?))
        } else {
            Err(LoxResult::return_value(Object::Nil))
        }
    }

    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        let value = if let Some(initializer) = stmt.initializer.clone() {
            self.evaluate(initializer)?
        } else {
            Object::Nil
        };

        self.environment
            .borrow()
            .borrow_mut()
            .define(stmt.name.as_string(), value);
        Ok(())
    }

    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        while self.is_truthy(self.evaluate(stmt.condition.clone())?) {
            match self.execute(stmt.body.clone()) {
                Err(LoxResult::Break) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }
        Ok(())
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_assign_expr(&self, warpper: Rc<Expr>, expr: &AssignExpr) -> Result<Object, LoxResult> {
        let value = self.evaluate(expr.value.clone())?;

        if let Some(&distance) = self.locals.borrow().get(&warpper) {
            self.environment.borrow().borrow_mut().assign_at(
                distance,
                &expr.name,
                value.clone(),
            )?;
        } else {
            self.globals
                .borrow_mut()
                .assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;
        let right = self.evaluate(expr.right.clone())?;
        let op = expr.operator.token_type();
        let result = match (left, right) {
            (Object::Num(left), Object::Num(right)) => match op {
                TokenType::Plus => Object::Num(left + right),
                TokenType::Minus => Object::Num(left - right),
                TokenType::Slash => Object::Num(left / right),
                TokenType::Star => Object::Num(left * right),
                TokenType::Greater => Object::Bool(left > right),
                TokenType::GreaterEqual => Object::Bool(left >= right),
                TokenType::Less => Object::Bool(left < right),
                TokenType::LessEqual => Object::Bool(left <= right),
                TokenType::Equal => Object::Bool(left == right),
                TokenType::BangEqual => Object::Bool(left != right),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to num binary expression",
                    ));
                }
            },
            (Object::Str(left), Object::Str(right)) => match op {
                TokenType::Plus => Object::Str(left + &*right),
                TokenType::Greater => Object::Bool(left > right),
                TokenType::GreaterEqual => Object::Bool(left >= right),
                TokenType::Less => Object::Bool(left < right),
                TokenType::LessEqual => Object::Bool(left <= right),
                TokenType::Equal => Object::Bool(left == right),
                TokenType::BangEqual => Object::Bool(left != right),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to string binary expression",
                    ));
                }
            },
            (Object::Str(left), Object::Num(right)) => match op {
                TokenType::Plus => Object::Str(format!("{}{}", left, right)),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to num and string binary expression",
                    ));
                }
            },
            (Object::Num(left), Object::Str(right)) => match op {
                TokenType::Plus => Object::Str(format!("{}{}", left, right)),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to string and num binary expression",
                    ));
                }
            },
            (Object::Nil, Object::Nil) => match op {
                TokenType::Equal => Object::Bool(true),
                TokenType::BangEqual => Object::Bool(false),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to nil binary expression",
                    ));
                }
            },
            (Object::Nil, _) => match op {
                TokenType::Equal => Object::Bool(false),
                TokenType::BangEqual => Object::Bool(true),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to nil eq other binary expression",
                    ));
                }
            },
            (Object::Bool(left), Object::Bool(right)) => match op {
                TokenType::Equal => Object::Bool(left == right),
                TokenType::BangEqual => Object::Bool(left != right),
                _ => {
                    return Err(LoxResult::error(
                        expr.operator.line,
                        "Unreachable according to bool binary expression",
                    ));
                }
            },
            _ => {
                return Err(LoxResult::error(
                    expr.operator.line,
                    "Both operands of the comparison expression must be of the same type",
                ))
            }
        };

        if result == Object::ArithmeticError {
            Err(LoxResult::runtime_error(
                expr.operator.dup(),
                "Illegal expression",
            ))
        } else {
            Ok(result)
        }
    }

    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<Object, LoxResult> {
        let callee = self.evaluate(expr.callee.clone())?;

        let mut arguments = Vec::new();
        for argument in expr.arguments.clone() {
            arguments.push(self.evaluate(argument)?);
        }

        if let Object::Func(function) = callee {
            if arguments.len() != function.func.arity() {
                return Err(LoxResult::runtime_error(
                    expr.paren.dup(),
                    &format!(
                        "Expect {} arguments but got {}.",
                        function.func.arity(),
                        arguments.len()
                    ),
                ));
            }
            function.func.call(self, arguments)
        } else {
            Err(LoxResult::runtime_error(
                expr.paren.dup(),
                "Can only call functions and classes.",
            ))
        }
    }

    fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<Object, LoxResult> {
        self.evaluate(expr.expression.clone())
    }

    fn visit_literal_expr(&self, _: Rc<Expr>, expr: &LiteralExpr) -> Result<Object, LoxResult> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;

        if expr.operator.is(TokenType::Or) {
            // or
            if self.is_truthy(left.clone()) {
                return Ok(left);
            }
        } else if !self.is_truthy(left.clone()) {
            // and
            return Ok(left);
        }
        self.evaluate(expr.right.clone())
    }

    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<Object, LoxResult> {
        let right = self.evaluate(expr.right.clone())?;

        match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => Ok(Object::Num(-n)),
                _ => Ok(Object::Nil),
            },
            TokenType::Bang => Ok(Object::Bool(!self.is_truthy(right))),
            _ => Err(LoxResult::error(
                expr.operator.line,
                "Unreachable according to Unary expression",
            )),
        }
    }

    fn visit_variable_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &VariableExpr,
    ) -> Result<Object, LoxResult> {
        self.look_up_variable(&expr.name, wrapper)
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new()));
        globals.borrow_mut().define(
            "clock".to_string(),
            Object::Func(Callable {
                func: Rc::new(NativeClock {}),
            }),
        );
        Interpreter {
            globals: Rc::clone(&globals),
            environment: RefCell::new(Rc::clone(&globals)),
            locals: RefCell::new(HashMap::new()),
        }
    }

    fn evaluate(&self, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        // if let Err(e) = self.check_global_function("clock") {
        //     return Err(e);
        // }
        expr.accept(expr.clone(), self)
    }

    fn execute(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        // println!("{:?}", &stmt);
        stmt.accept(stmt.clone(), self)
    }

    pub fn execute_block(
        &self,
        statements: &[Rc<Stmt>],
        env: Environment,
    ) -> Result<(), LoxResult> {
        let previous = self.environment.replace(Rc::new(RefCell::new(env)));
        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement.clone()));
        self.environment.replace(previous);
        result
    }

    // fn check_global_function(&self, name: &str) -> Result<(), LoxResult> {
    //     if self.environment.borrow().borrow().get_by_name(name) {
    //         Err(LoxResult::system_error(
    //             &"Can't use global function name as identifier.",
    //         ))
    //     } else {
    //         Ok(())
    //     }
    // }

    fn is_truthy(&self, right: Object) -> bool {
        !matches!(right, Object::Nil | Object::Bool(false))
    }

    pub fn interpret(&self, statements: Vec<Rc<Stmt>>) -> bool {
        let mut success = true;
        for statement in statements {
            if let Err(e) = self.execute(statement.clone()) {
                e.report();
                success = false;
                break;
            }
        }
        success
    }

    pub fn print_environment(&self) {
        println!("{:?}", self.environment);
    }

    pub fn resolve(&self, expr: Rc<Expr>, depth: usize) {
        self.locals.borrow_mut().insert(expr, depth);
    }

    fn look_up_variable(&self, name: &Token, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        if let Some(distance) = self.locals.borrow().get(&expr) {
            self.environment
                .borrow()
                .borrow()
                .get_at(*distance, &name.as_string())
        } else {
            self.globals.borrow().get(name)
        }
    }
}
