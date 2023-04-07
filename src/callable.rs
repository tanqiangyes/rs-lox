use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::lox_class::LoxClass;
use crate::object::Object;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct Callable {
    pub func: Rc<dyn LoxCallable>,
}

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult>;
    fn arity(&self) -> usize;
}

impl LoxCallable for Callable {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        self.func.call(interpreter, arguments, klass)
    }

    fn arity(&self) -> usize {
        self.func.arity()
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<callable>")
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
