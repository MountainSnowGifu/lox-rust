use crate::callable::LoxFunction;
use crate::error::Result;
use crate::lox_instance::LoxInstance;
use crate::{callable::LoxCallable, interpreter::Interpreter, object::Object};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> LoxClass {
        LoxClass { name, methods }
    }

    pub fn find_method(&self, name: String) -> Option<&LoxFunction> {
        if let Some(m) = self.methods.get(&name) {
            return Some(m);
        }
        None
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object> {
        let instance = LoxInstance::new(self.clone());
        if let Some(initializer) = self.find_method("init".to_string()) {
            initializer
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }
        Ok(Object::Instance(instance))
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init".to_string()) {
            initializer.arity()
        } else {
            0
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClassType {
    None,
    Class,
    //SubClass,
}
