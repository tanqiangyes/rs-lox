use crate::error::*;
use crate::object::Object;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Object, LoxResult> {
        if distance == 0 {
            Ok(self.values.get(name).unwrap().clone())
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(distance - 1, name)
        }
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<(), LoxResult> {
        if distance == 0 {
            self.values.insert(name.as_string(), value);
            Ok(())
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(distance - 1, name, value)
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Some(object) = self.values.get(&name.as_string()) {
            Ok(object.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(LoxResult::runtime_error(
                name.dup(),
                &format!("Undefined variable '{}'.", &name.as_string()),
            ))
        }
    }
    // pub fn get_by_name(&self, name: &str) -> bool {
    //     if let Some(_) = self.values.get(name) {
    //         true
    //     } else if let Some(enclosing) = &self.enclosing {
    //         enclosing.borrow().get_by_name(name)
    //     } else {
    //         false
    //     }
    // }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), LoxResult> {
        if let Entry::Occupied(mut object) = self.values.entry(name.as_string()) {
            object.insert(value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(LoxResult::runtime_error(
                name.dup(),
                &format!("Undefined variable '{}'.", &name.as_string()),
            ))
        }
    }
}
