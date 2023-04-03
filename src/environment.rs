use crate::error::*;
use crate::object::Object;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            variables: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            variables: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Some(object) = self.variables.get(&name.as_string()) {
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
    //     if let Some(_) = self.variables.get(name) {
    //         true
    //     } else if let Some(enclosing) = &self.enclosing {
    //         enclosing.borrow().get_by_name(name)
    //     } else {
    //         false
    //     }
    // }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), LoxResult> {
        if let Entry::Occupied(mut object) = self.variables.entry(name.as_string()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_type::TokenType;

    #[test]
    fn can_get_a_variable() {
        let mut e = Environment::new();

        e.define("One".to_string(), Object::Bool(true));
        e.define("One".to_string(), Object::Num(1.0));

        assert!(e.variables.contains_key("One"));
        let t = Token::new(TokenType::Identifier, "One".to_string(), None, 0);
        assert_eq!(e.get(&t).unwrap(), Object::Num(1.0))
    }

    #[test]
    fn can_a_not_define_variable() {
        let mut e = Environment::new();

        e.define("One".to_string(), Object::Num(1.0));

        assert!(e.variables.contains_key("One"));
        let t = Token::new(TokenType::Identifier, "Two".to_string(), None, 0);
        assert!(e.get(&t).is_err())
    }
}
