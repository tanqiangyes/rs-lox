use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::object::Object;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct Callable {
    pub func: Rc<dyn LoxCallable>,
}

pub trait LoxCallable {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl LoxCallable for Callable {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {
        self.func.call(interpreter, arguments)
    }

    fn arity(&self) -> usize {
        self.func.arity()
    }

    fn to_string(&self) -> String {
        self.func.to_string()
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<callable>")
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            Rc::as_ptr(&self.func) as *const (),
            Rc::as_ptr(&other.func) as *const (),
        )
    }
}
