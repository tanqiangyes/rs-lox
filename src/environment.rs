use crate::error::LoxError;
use crate::object::Object;
use crate::token::Token;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Environment {
    variables: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxError> {
        if let Some(object) = self.variables.get(&name.as_string()) {
            Ok(object.clone())
        } else {
            Err(LoxError::runtime_error(
                name.dup(),
                &format!("Undefined variable '{}'.", &name.as_string()),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), LoxError> {
        if let Entry::Occupied(mut object) = self.variables.entry(name.as_string()) {
            object.insert(value);
            Ok(())
        } else {
            Err(LoxError::runtime_error(
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
