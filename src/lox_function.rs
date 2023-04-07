use crate::callable::LoxCallable;
use crate::environment::Environment;
use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::lox_class::LoxClass;
use crate::object::Object;
use crate::stmt::*;
use crate::token::Token;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: Token,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>,
    is_initialized: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: &FunctionStmt,
        closure: &Rc<RefCell<Environment>>,
        is_initialized: bool,
    ) -> LoxFunction {
        LoxFunction {
            name: declaration.name.dup(),
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
            closure: Rc::clone(closure),
            is_initialized,
        }
    }

    pub fn bind(&self, instance: &Object) -> Object {
        let mut environment = Environment::new_with_enclosing(Rc::clone(&self.closure));
        environment.define("this".to_string(), instance.clone());

        Object::Func(Rc::new(LoxFunction {
            name: self.name.dup(),
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::new(RefCell::new(environment)),
            is_initialized: self.is_initialized,
        }))
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        _klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        let mut e = Environment::new_with_enclosing(Rc::clone(&self.closure));

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            e.define(param.as_string(), arg.clone());
        }
        match interpreter.execute_block(&self.body, e) {
            Err(LoxResult::ReturnValue { value }) => {
                if self.is_initialized {
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(value)
                }
            }
            Err(e) => Err(e),
            Ok(_) => {
                if self.is_initialized {
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(Object::Nil)
                }
            }
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let param_list = self
            .params
            .iter()
            .map(|p| p.as_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "Function {}({})", self.name.as_string(), param_list)
    }
}
