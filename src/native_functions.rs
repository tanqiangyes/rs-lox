use crate::callable::LoxCallable;
use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::lox_class::LoxClass;
use crate::object::Object;
use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

// --------------------------------------------Native functions ----------------------------------------------------------------
#[derive(Clone)]
pub struct LoxNative {
    pub func: Rc<dyn LoxCallable>,
}

impl PartialEq for LoxNative {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            Rc::as_ptr(&self.func) as *const (),
            Rc::as_ptr(&other.func) as *const (),
        )
    }
}

impl fmt::Debug for LoxNative {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Native-Function>")
    }
}

impl fmt::Display for LoxNative {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

pub struct NativeClock;

impl LoxCallable for NativeClock {
    fn call(
        &self,
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
        _klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(times) => Ok(Object::Num(times.as_millis() as f64)),
            Err(e) => Err(LoxResult::system_error(&format!(
                "Call to native function 'clock' failed, Error: {}",
                e
            ))),
        }
    }

    fn arity(&self) -> usize {
        0
    }
}
// -----------------------------------------------------------------------------------------------------------------------------
