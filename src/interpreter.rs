use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::{Clock, LoxCallable, LoxFunction},
    environment::Environment,
    error::{Error, Result},
    expr::{self, Acceptor as ExprAcceptor, Expr},
    lox_class::LoxClass,
    object::Object,
    stmt::{self, Acceptor as StmtAcceptor, Stmt},
    token::{Literal, Token},
    token_type::TokenType,
};

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals
            .borrow_mut()
            .define(String::from("clock"), &Object::Clock(Clock {}));
        Interpreter {
            globals: Rc::clone(&globals),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in statements {
            match self.execute(&statement) {
                Ok(_) => {}
                Err(r) => eprintln!("{:?}", r),
            }
        }
        Ok(())
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) -> Result<()> {
        self.locals.insert(expr, depth);
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn is_truthy(&self, object: Object) -> bool {
        match object {
            Object::Literal(literal) => match literal {
                Literal::None => false,
                Literal::Bool(b) => b,
                _ => true,
            },
            // FIXME
            _ => false,
        }
    }

    fn look_up_variable(&mut self, name: &Token, expr: &Expr) -> Result<Object> {
        match self.locals.get(&expr) {
            Some(distance) => self
                .environment
                .borrow()
                .get_at(distance.clone(), name.lexeme.clone()),
            _ => self.globals.borrow().get(name),
        }
    }

    fn is_equal(&self, a: Object, b: Object) -> bool {
        match (a, b) {
            (Object::Literal(ola), Object::Literal(olb)) => match (ola, olb) {
                (Literal::None, Literal::None) => true,
                (Literal::Bool(a), Literal::Bool(b)) => a == b,
                (Literal::String(a), Literal::String(b)) => a == b,
                (Literal::Isize(a), Literal::Isize(b)) => a == b,
                (Literal::Float(a), Literal::Float(b)) => a == b,
                _ => false,
            },
            // FIXME
            _ => false,
        }
    }

    pub fn execute_block(&mut self, statements: &[Stmt], environment: Environment) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => {}
                Err(e) => {
                    self.environment = previous;
                    return Err(e);
                }
            };
        }
        self.environment = previous;
        Ok(())
    }
}

impl expr::Visitor<Result<Object>> for Interpreter {
    fn visit_literal(&mut self, expr: &Literal) -> Result<Object> {
        Ok(Object::Literal(expr.clone()))
    }
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Object> {
        use crate::token::Literal::{Bool, Float, Isize, None, String as LString};
        use crate::token_type::TokenType::{
            BANGEQUAL, EQUALEQUAL, GREATER, GREATEREQUAL, LESS, LESSEQUAL, MINUS, PLUS, SLASH, STAR,
        };
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            GREATER => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Bool(l > r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Bool(l as f64 > r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Bool(l > r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Bool(l > r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            GREATEREQUAL => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Bool(l >= r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Bool(l as f64 >= r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Bool(l >= r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Bool(l >= r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            LESS => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Bool(l < r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Bool((l as f64) < r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Bool(l < r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Bool(l < r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            LESSEQUAL => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Bool(l <= r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Bool((l as f64) <= r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Bool(l <= r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Bool(l <= r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            MINUS => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Isize(l - r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Float((l as f64) - r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Float(l - r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Float(l - r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            PLUS => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Isize(l + r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Float((l as f64) + r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Float(l + r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Float(l + r))),
                    (LString(l), LString(r)) => Ok(Object::Literal(LString(format!("{}{}", l, r)))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            SLASH => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Isize(l / r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Float((l as f64) / r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Float(l / r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Float(l / r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            STAR => match (left, right) {
                (Object::Literal(oll), Object::Literal(olr)) => match (oll, olr) {
                    (Isize(l), Isize(r)) => Ok(Object::Literal(Isize(l * r))),
                    (Isize(l), Float(r)) => Ok(Object::Literal(Float((l as f64) * r))),
                    (Float(l), Isize(r)) => Ok(Object::Literal(Float(l * r as f64))),
                    (Float(l), Float(r)) => Ok(Object::Literal(Float(l * r))),
                    _ => Err(Error::RuntimeError(
                        operator.clone(),
                        String::from("Operands must be numbers."),
                    )),
                },
                _ => unreachable!(),
            },

            BANGEQUAL => Ok(Object::Literal(Bool(!self.is_equal(left, right)))),
            EQUALEQUAL => Ok(Object::Literal(Bool(self.is_equal(left, right)))),
            _ => Ok(Object::Literal(None)),
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Result<Object> {
        self.evaluate(expression)
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Result<Object> {
        use crate::token::Literal::{Bool, Float, Isize, None};

        let right = self.evaluate(right)?;
        match (operator.token_type, right) {
            (TokenType::MINUS, Object::Literal(lit)) => match lit {
                Isize(r) => Ok(Object::Literal(Isize(-r))),
                Float(r) => Ok(Object::Literal(Float(-r))),
                _ => Err(Error::RuntimeError(
                    operator.clone(),
                    String::from("Operand must be a number."),
                )),
            },
            (TokenType::BANG, Object::Literal(lit)) => {
                Ok(Object::Literal(Bool(!self.is_truthy(Object::Literal(lit)))))
            }
            _ => Ok(Object::Literal(None)),
        }
    }

    fn visit_variable(&mut self, name: &Token) -> Result<Object> {
        let expr = Expr::Variable { name: name.clone() };
        self.look_up_variable(name, &expr)
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> Result<Object> {
        let evaluated_value = self.evaluate(value)?;
        let expr = Expr::Assign {
            name: name.clone(),
            value: Box::new(value.clone()),
        };
        match self.locals.get(&expr) {
            Some(distance) => {
                self.environment.borrow_mut().assign_at(
                    distance.clone(),
                    name.clone(),
                    evaluated_value.clone(),
                );
            }
            None => self.globals.borrow_mut().assign(name, &evaluated_value)?,
        }
        Ok(evaluated_value)
    }

    fn visit_logical(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Object> {
        let evaluated_left = self.evaluate(left);
        let is_left_truthy = self.is_truthy(evaluated_left.clone()?);

        if operator.token_type == TokenType::OR {
            if is_left_truthy {
                return evaluated_left;
            }
        } else if !is_left_truthy {
            return evaluated_left;
        }
        self.evaluate(right)
    }

    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> Result<Object> {
        let callee = self.evaluate(callee)?;
        let mut evaluated_args = vec![];
        for argument in arguments {
            evaluated_args.push(self.evaluate(argument)?)
        }
        match callee {
            Object::Func(func) => {
                if evaluated_args.len() != func.arity() {
                    return Err(Error::RuntimeError(
                        paren.clone(),
                        format!(
                            "Expected {} arguments but got {}.",
                            func.arity(),
                            evaluated_args.len()
                        ),
                    ));
                }
                Ok(func.call(self, evaluated_args)?)
            }
            Object::Clock(func) => {
                if evaluated_args.len() != func.arity() {
                    return Err(Error::RuntimeError(
                        paren.clone(),
                        format!(
                            "Expected {} arguments but got {}.",
                            func.arity(),
                            evaluated_args.len()
                        ),
                    ));
                }
                Ok(func.call(self, evaluated_args)?)
            }
            Object::Class(class) => {
                if evaluated_args.len() != class.arity() {
                    return Err(Error::RuntimeError(
                        paren.clone(),
                        format!(
                            "Expected {} arguments but got {}.",
                            class.arity(),
                            evaluated_args.len()
                        ),
                    ));
                }
                Ok(class.call(self, evaluated_args)?)
            }
            _ => Err(Error::RuntimeError(
                paren.clone(),
                String::from("Can only call functions and classes."),
            )),
        }
    }

    fn visit_get(&mut self, object: &Expr, name: &Token) -> Result<Object> {
        let evaluated_object = self.evaluate(object)?;
        match evaluated_object {
            Object::Instance(mut instance) => Ok(instance.get(name)?),
            _ => Err(Error::RuntimeError(
                name.clone(),
                String::from("Only instances have properties."),
            )),
        }
    }

    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<Object> {
        let evaluated_object = self.evaluate(object)?;
        match evaluated_object {
            Object::Instance(mut instance) => {
                let evaluated_value = self.evaluate(value)?;
                instance.set(name, &evaluated_value);
                Ok(evaluated_value)
            }
            _ => Err(Error::RuntimeError(
                name.clone(),
                String::from("Only instances have fields."),
            )),
        }
    }

    fn visit_super(&mut self, keyword: &Token, method: &Token) -> Result<Object> {
        let distance = self
            .locals
            .get(&Expr::Super {
                keyword: keyword.clone(),
                method: method.clone(),
            })
            .expect(&format!("super found on locals: {:?}", self.locals));
        let object_super = self
            .environment
            .borrow()
            .get_at(distance.clone(), "super".to_string())?;
        if let Object::Class(superclass) = object_super {
            let this = self
                .environment
                .borrow()
                .get_at(distance - 1, "this".to_string())?;
            if let Object::Instance(object) = this {
                if let Some(method) = superclass.find_method(method.lexeme.clone()) {
                    return Ok(Object::Func(method.bind(object)));
                }
                return Err(Error::RuntimeError(
                    method.clone(),
                    format!("Undefined propertiesperty '{}' '.", method.lexeme),
                ));
            }
            return Err(Error::RuntimeError(
                method.clone(),
                format!("'this' should be instance but actually: {}'.", this),
            ));
        }
        Err(Error::RuntimeError(
            method.clone(),
            format!("'super' should be class but actually: {}'.", object_super),
        ))
    }

    fn visit_this(&mut self, keyword: &Token) -> Result<Object> {
        let expr = Expr::This {
            keyword: keyword.clone(),
        };
        self.look_up_variable(keyword, &expr)
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<()> {
        let result = self.evaluate(expression)?;
        // if self.environment.borrow().is_repl {
        //     println!("{}", result);
        // }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<()> {
        let value = self.evaluate(expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> Result<()> {
        let value = self.evaluate(initializer)?;

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), &value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<()> {
        //let is_repl = self.environment.borrow().is_repl;
        self.execute_block(
            statements,
            Environment::new(Some(Rc::clone(&self.environment))),
        )?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        let evaluated = self.evaluate(condition)?;
        if self.is_truthy(evaluated) {
            self.execute(then_branch)?
        }
        match else_branch {
            Some(eb) => self.execute(&*eb)?,
            None => {}
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        loop {
            let evaluated_condition = self.evaluate(condition)?;
            if self.is_truthy(evaluated_condition) {
                self.execute(body)?
            } else {
                break;
            }
        }
        Ok(())
    }

    fn visit_function_stmt(&mut self, name: &Token, params: &[Token], body: &[Stmt]) -> Result<()> {
        use super::callable::LoxFunction;
        let function = Object::Func(LoxFunction::new(
            name.clone(),
            params.to_vec(),
            body.to_vec(),
            Rc::clone(&self.environment),
            false,
        ));
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), &function);
        Ok(())
    }

    fn visit_return_stmt(&mut self, _keyword: &Token, v: &Expr) -> Result<()> {
        let evaluated_value = match v {
            Expr::Literal { value } => match value {
                Literal::None => Object::Literal(Literal::None),
                _ => self.evaluate(&v)?,
            },
            _ => self.evaluate(&v)?,
        };
        Err(Error::Return(evaluated_value))
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        super_class: &Option<Expr>,
        class_methods: &[Stmt],
    ) -> Result<()> {
        let evaluated_super_class = match super_class {
            Some(sc) => match self.evaluate(sc)? {
                Object::Class(lc) => Some(Box::new(lc)),
                _ => {
                    if let Expr::Variable { name: scname } = sc {
                        return Err(Error::RuntimeError(
                            scname.clone(),
                            "Superclass must be a class.".to_string(),
                        ));
                    }
                    unreachable!()
                }
            },
            None => None,
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), &Object::Literal(Literal::None));

        if super_class.is_some() {
            let new_env = Environment::new(Some(Rc::clone(&self.environment)));
            self.environment = Rc::new(RefCell::new(new_env));
            self.environment.borrow_mut().define(
                "super".to_string(),
                &Object::Class(
                    *evaluated_super_class
                        .clone()
                        .expect("superclass does not exist."),
                ),
            );
        }

        let mut methods: HashMap<String, LoxFunction> = HashMap::new();
        for method in class_methods {
            match method {
                Stmt::Function {
                    name: func_name,
                    params,
                    body,
                } => {
                    let function = LoxFunction::new(
                        func_name.clone(),
                        params.to_vec(),
                        body.to_vec(),
                        Rc::clone(&self.environment),
                        func_name.lexeme == "init",
                    );
                    methods.insert(func_name.lexeme.clone(), function);
                }
                _ => unreachable!(),
            }
        }

        let klass = LoxClass::new(name.lexeme.clone(), evaluated_super_class.clone(), methods);

        if evaluated_super_class.is_some() {
            let enclosing = self
                .environment
                .borrow()
                .enclosing
                .clone()
                .expect("doesn't have enclosing");
            self.environment = enclosing;
        }

        self.environment
            .borrow_mut()
            .assign(name, &Object::Class(klass))?;
        Ok(())
    }
}
