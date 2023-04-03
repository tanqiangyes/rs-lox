use crate::callable::LoxCallable;
use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::object::Object;
use std::time::SystemTime;

// --------------------------------------------Native functions ----------------------------------------------------------------
pub struct NativeClock;
impl LoxCallable for NativeClock {
    fn call(
        &self,
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
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

    fn to_string(&self) -> String {
        "<native clock>".to_string()
    }
}
// -----------------------------------------------------------------------------------------------------------------------------
