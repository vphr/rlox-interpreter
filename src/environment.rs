use std::{
    cell::RefCell,
    collections::HashMap
};

use crate::{error::*, interpreter::*, scanner::*};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Box<RefCell<Environment>>>,
    pub values: HashMap<String, Value>,
}

impl Default for Environment{
    fn default() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn new(enclosing: RefCell<Environment>) -> Environment {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: &Vec<u8>, value: Value) {
        let name = String::from_utf8(name.to_vec()).expect("valid string");
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, token: &Token) -> Result<Value, RloxError> {
        let cloned_lexeme_vec = token.lexeme.to_vec();
        let name = String::from_utf8(cloned_lexeme_vec).expect("valid string");

        match self.values.get(&name) {
            Some(val) => Ok(val.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(token),
                None => Err(RloxError::RuntimeError {
                    lexeme: name.clone(),
                    message: format!("Undefined variable {}.", &name),
                }),
            },
        }
    }
    pub fn assign(&mut self, token: &Token, value: &Value) -> Result<(), RloxError> {
        let cloned_lexeme_vec = token.lexeme.to_vec();
        let name = String::from_utf8(cloned_lexeme_vec).expect("valid string");

        if self.values.contains_key(&name) {
            self.define(&token.lexeme, value.clone());
            return Ok(());
        }

        match &mut self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(token, value),
            None => Err(RloxError::RuntimeError {
                lexeme: name.clone(),
                message: format!("Undefined variable '{}'.", name),
            }),
        }
    }
}
