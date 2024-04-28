use crate::{
    error,
    expr::Expr,
    stmt::Stmt,
    token::{Literal, Token},
    token_type::TokenType,
};

#[derive(Debug, Clone)]
pub struct ParserError(String);

type ParseResult<T> = Result<T, ParserError>;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        let result = if self.contains(&[TokenType::VAR]) {
            self.var_declaration()
        } else if self.contains(&[TokenType::CLASS]) {
            self.class_declaration()
        } else if self.contains(&[TokenType::FUN]) {
            self.function(String::from("function"))
        } else {
            self.statement()
        };

        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    fn class_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect class name.")?;

        let super_class = if self.contains(&[TokenType::LESS]) {
            self.consume(TokenType::IDENTIFIER, "Expect superclass name.");
            Some(Expr::Variable {
                name: self.previous().clone(),
            })
        } else {
            None
        };

        self.consume(TokenType::LEFTBRACE, "Expect '{' before class body.")?;

        let mut methods = vec![];
        while !self.check(TokenType::RIGHTBRACE) && !self.is_at_end() {
            let function = self.function(String::from("method"))?;
            methods.push(function);
        }
        self.consume(TokenType::RIGHTBRACE, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            super_class,
            methods,
        })
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.contains(&[TokenType::FOR]) {
            return self.for_statement();
        }

        if self.contains(&[TokenType::IF]) {
            return self.if_statement();
        }

        if self.contains(&[TokenType::PRINT]) {
            return self.print_statement();
        }

        if self.contains(&[TokenType::RETURN]) {
            return self.return_statemet();
        }

        if self.contains(&[TokenType::WHILE]) {
            return self.while_statement();
        }

        if self.contains(&[TokenType::LEFTBRACE]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LEFTPAREN, "Expect '(' after 'for'.")?;

        let initializer = if self.contains(&[TokenType::SEMICOLON]) {
            None
        } else if self.contains(&[TokenType::VAR]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::SEMICOLON) {
            self.expression()?
        } else {
            Expr::Literal {
                value: Literal::Bool(true),
            }
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RIGHTPAREN) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RIGHTPAREN, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if increment.is_some() {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: increment.unwrap(),
                    },
                ],
            }
        }
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };
        if initializer.is_some() {
            body = Stmt::Block {
                statements: vec![initializer.unwrap(), body],
            }
        }
        Ok(body)
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LEFTPAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHTPAREN, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.contains(&[TokenType::ELSE]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print { expression: value })
    }

    fn return_statemet(&mut self) -> ParseResult<Stmt> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::SEMICOLON) {
            self.expression()?
        } else {
            Expr::Literal {
                value: Literal::None,
            }
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LEFTPAREN, "Expectct '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHTPAREN, "Expectct ')' after condition.")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn function(&mut self, kind: String) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, &format!("Expect {} name.", kind))?;
        self.consume(
            TokenType::LEFTPAREN,
            &format!("Expect '(' after {} name.", kind),
        )?;
        let mut parameters = vec![];
        loop {
            if !self.check(TokenType::RIGHTPAREN) {
                if parameters.len() >= 255 {
                    return Err(Parser::error(
                        self.peek().clone(),
                        "Cannot have more than 255 parameters.",
                    ));
                }
                parameters.push(self.consume(TokenType::IDENTIFIER, "Expect parameter name.")?)
            }
            if !self.contains(&[TokenType::COMMA]) {
                break;
            }
        }
        self.consume(TokenType::RIGHTPAREN, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LEFTBRACE,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
        })
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ':' after value.")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        let result = self.assignment();

        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.check(TokenType::RIGHTBRACE) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }
        self.consume(TokenType::RIGHTBRACE, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if self.contains(&[TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            return match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                Expr::Get { object, name } => Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                }),
                _ => Err(Parser::error(equals, "Invalid assignment target.")),
            };
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;
        while self.contains(&[TokenType::OR]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;
        while self.contains(&[TokenType::AND]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.")?;

        let initializer = if self.contains(&[TokenType::EQUAL]) {
            self.expression()?
        } else {
            Expr::Literal {
                value: Literal::None,
            }
        };

        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;
        while self.contains(&[TokenType::BANGEQUAL, TokenType::EQUALEQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = match self.term() {
            Ok(result) => result,
            Err(err) => return Err(err),
        };

        while self.contains(&[
            TokenType::GREATER,
            TokenType::GREATEREQUAL,
            TokenType::LESS,
            TokenType::LESSEQUAL,
        ]) {
            let operator = self.previous().clone();
            let right = match self.term() {
                Ok(result) => result,
                Err(err) => return Err(err),
            };
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    //addition
    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = match self.factor() {
            Ok(multiplication) => multiplication,
            Err(err) => return Err(err),
        };

        while self.contains(&[TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().clone();
            let right = match self.factor() {
                Ok(multiplication) => multiplication,
                Err(err) => return Err(err),
            };
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    //multiplication
    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = match self.unary() {
            Ok(unary) => unary,
            Err(err) => return Err(err),
        };

        while self.contains(&[TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = match self.unary() {
                Ok(unary) => unary,
                Err(err) => return Err(err),
            };
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.contains(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = match self.unary() {
                Ok(unary) => unary,
                Err(err) => return Err(err),
            };
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        return self.call();
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut arguments = vec![];
        if !self.check(TokenType::RIGHTPAREN) {
            loop {
                if arguments.len() >= 255 {
                    Parser::error(self.peek().clone(), "Cannot have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.contains(&[TokenType::COMMA]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RIGHTPAREN, "Expect ')' after arguments.")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.contains(&[TokenType::LEFTPAREN]) {
                expr = self.finish_call(expr)?;
            } else if self.contains(&[TokenType::DOT]) {
                let name =
                    self.consume(TokenType::IDENTIFIER, "Expect property name after '.'.")?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.contains(&[TokenType::FALSE]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        };

        if self.contains(&[TokenType::TRUE]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }

        if self.contains(&[TokenType::NIL]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }

        if self.contains(&[TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal {
                value: self.previous().literal.clone(),
            });
        }

        if self.contains(&[TokenType::SUPER]) {
            let keyword = self.previous().clone();
            self.consume(TokenType::DOT, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenType::IDENTIFIER, "Expect superclass method name.")?;
            return Ok(Expr::Super { keyword, method });
        }

        if self.contains(&[TokenType::THIS]) {
            return Ok(Expr::This {
                keyword: self.previous().clone(),
            });
        }

        if self.contains(&[TokenType::IDENTIFIER]) {
            return Ok(Expr::Variable {
                name: self.previous().clone(),
            });
        }

        if self.contains(&[TokenType::LEFTPAREN]) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHTPAREN, "Expect')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(Parser::error(self.peek().clone(), "Expect expression."))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> ParseResult<Token> {
        if self.check(token_type) {
            return Ok(self.advance().clone());
        }
        let token = self.peek();

        Err(Parser::error(token.clone(), message))
    }

    fn error(token: Token, message: &str) -> ParserError {
        error::parser_error(token, &message);
        ParserError(String::from(message))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {
                return;
            }

            match self.peek().token_type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    // equal to match function on Java implementation
    fn contains(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&mut self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
