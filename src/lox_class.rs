use crate::callable::LoxCallable;
use crate::error::LoxResult;
use crate::interpreter::Interpreter;
use crate::lox_instance::LoxInstance;
use crate::object::Object;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Object>,
    superclass: Option<Rc<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Object>,
    ) -> Self {
        Self {
            name,
            methods,
            superclass,
        }
    }

    pub fn find_method(&self, name: &String) -> Option<Object> {
        if let Some(obj) = self.methods.get(name).cloned() {
            Some(obj)
        } else if let Some(superclass) = self.superclass.clone() {
            superclass.find_method(name)
        } else {
            None
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let methods = self
            .methods
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "<Class {} {{ {methods} }}>", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        let instance = Object::Instance(Rc::new(LoxInstance::new(klass.clone().unwrap())));
        if let Some(Object::Func(init)) = self.find_method(&"init".to_string()) {
            if let Object::Func(init) = init.bind(&instance) {
                init.call(interpreter, arguments, klass)?;
            }
        }
        Ok(instance)
    }

    fn arity(&self) -> usize {
        if let Some(Object::Func(init)) = self.find_method(&"init".to_string()) {
            init.arity()
        } else {
            0
        }
    }
}
