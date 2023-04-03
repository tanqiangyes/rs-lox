use crate::callable::LoxCallable;
use crate::environment::Environment;
use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::object::Object;
use crate::stmt::*;
use crate::token::Token;
use std::rc::Rc;

pub struct LoxFunction {
    name: Token,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
}

impl LoxFunction {
    pub fn new(declaration: &FunctionStmt) -> LoxFunction {
        LoxFunction {
            name: declaration.name.dup(),
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {
        let mut e = Environment::new_with_enclosing(interpreter.globals.clone());

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            e.define(param.as_string(), arg.clone());
        }
        interpreter.execute_block(&self.body, e)?;
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.name.as_string())
    }
}
