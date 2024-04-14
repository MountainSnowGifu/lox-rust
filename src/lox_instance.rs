use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::error::{Error, Result};
use crate::{lox_class::LoxClass, object::Object, token::Token};

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    pub fields: Rc<RefCell<HashMap<String, Object>>>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> LoxInstance {
        LoxInstance {
            class: Rc::new(class),
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&mut self, name: &Token) -> Result<Object> {
        if let Some(o) = self.fields.borrow().get(&name.lexeme) {
            return Ok(o.clone());
        }

        if let Some(method) = self.class.find_method(name.lexeme.clone()) {
            return Ok(Object::Func(method.bind(self.clone())));
        }

        Err(Error::RuntimeError(
            name.clone(),
            format!("Undefined property '{}'", name.lexeme),
        ))
    }

    pub fn set(&mut self, name: &Token, value: &Object) {
        self.fields
            .borrow_mut()
            .insert(name.lexeme.clone(), value.clone());
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance, fields has", self.class.name)
    }
}
