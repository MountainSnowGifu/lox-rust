use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::time::SystemTime;

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::lox_instance::LoxInstance;
use crate::stmt::Stmt;
use crate::token::{Literal, Token};
use crate::{interpreter::Interpreter, object::Object};

pub trait LoxCallable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object>;
    fn arity(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    // Note: declaration
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> LoxFunction {
        LoxFunction {
            name,
            params,
            body,
            closure: env,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: LoxInstance) -> LoxFunction {
        let environement = Environment::new(Some(Rc::clone(&self.closure)));
        environement.define("this".to_string(), &Object::Instance(instance));
        LoxFunction::new(
            self.name.clone(),
            self.params.clone(),
            self.body.clone(),
            Rc::new(RefCell::new(environement)),
            self.is_initializer,
        )
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object> {
        let environement = Environment::new(Some(Rc::clone(&self.closure)));

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            environement.define(param.lexeme.clone(), arg)
        }

        match interpreter.execute_block(&self.body, environement) {
            Ok(_) => {
                if self.is_initializer {
                    return self.closure.borrow().get_at(0, "this".to_string());
                }
                Ok(Object::Literal(Literal::None))
            }
            Err(Error::Return(return_value)) => Ok(return_value),
            Err(e) => Err(e),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}

#[derive(Debug, Clone)]
pub struct Clock {}

impl LoxCallable for Clock {
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<Object>) -> Result<Object> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => Ok(Object::Literal(Literal::Isize(n.as_millis() as isize))),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    fn arity(&self) -> usize {
        0
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}
