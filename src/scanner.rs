use std::collections::HashMap;

use crate::{
    error,
    token::{Literal, Token},
    token_type::TokenType,
};

#[derive(Debug, Clone)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        let mut keywords = HashMap::new();
        keywords.insert(String::from("and"), TokenType::AND);
        keywords.insert(String::from("class"), TokenType::CLASS);
        keywords.insert(String::from("else"), TokenType::ELSE);
        keywords.insert(String::from("false"), TokenType::FALSE);
        keywords.insert(String::from("for"), TokenType::FOR);
        keywords.insert(String::from("fun"), TokenType::FUN);
        keywords.insert(String::from("if"), TokenType::IF);
        keywords.insert(String::from("nil"), TokenType::NIL);
        keywords.insert(String::from("or"), TokenType::OR);
        keywords.insert(String::from("print"), TokenType::PRINT);
        keywords.insert(String::from("return"), TokenType::RETURN);
        keywords.insert(String::from("super"), TokenType::SUPER);
        keywords.insert(String::from("this"), TokenType::THIS);
        keywords.insert(String::from("true"), TokenType::TRUE);
        keywords.insert(String::from("var"), TokenType::VAR);
        keywords.insert(String::from("while"), TokenType::WHILE);
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: keywords,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        let eof_token = Token::new(TokenType::EOF, String::from(""), Literal::None, self.line);
        self.tokens.push(eof_token);
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => {
                self.add_token_without_literal(TokenType::LEFTPAREN);
            }
            ')' => {
                self.add_token_without_literal(TokenType::RIGHTPAREN);
            }
            '{' => {
                self.add_token_without_literal(TokenType::LEFTBRACE);
            }
            '}' => {
                self.add_token_without_literal(TokenType::RIGHTBRACE);
            }
            ',' => {
                self.add_token_without_literal(TokenType::COMMA);
            }
            '.' => {
                self.add_token_without_literal(TokenType::DOT);
            }
            '-' => {
                self.add_token_without_literal(TokenType::MINUS);
            }
            '+' => {
                self.add_token_without_literal(TokenType::PLUS);
            }
            ';' => {
                self.add_token_without_literal(TokenType::SEMICOLON);
            }
            '*' => {
                self.add_token_without_literal(TokenType::STAR);
            }
            '!' => {
                if self.match_to_expected('=') {
                    self.add_token_without_literal(TokenType::BANGEQUAL);
                } else {
                    self.add_token_without_literal(TokenType::BANG);
                }
            }
            '=' => {
                if self.match_to_expected('=') {
                    self.add_token_without_literal(TokenType::EQUALEQUAL);
                } else {
                    self.add_token_without_literal(TokenType::EQUAL);
                }
            }
            '<' => {
                if self.match_to_expected('=') {
                    self.add_token_without_literal(TokenType::LESSEQUAL);
                } else {
                    self.add_token_without_literal(TokenType::LESS);
                }
            }
            '>' => {
                if self.match_to_expected('=') {
                    self.add_token_without_literal(TokenType::GREATEREQUAL);
                } else {
                    self.add_token_without_literal(TokenType::GREATER);
                }
            }
            '/' => {
                if self.match_to_expected('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token_without_literal(TokenType::SLASH);
                }
            }

            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => {
                self.line += 1;
            }

            '"' => {
                self.string();
            }

            '0'..='9' => self.number(),

            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),

            _ => {
                error::scanner_error(self.line, "Unexpected character.");
            }
        }
    }

    //文字列の現在の文字が期待された文字と一致するかどうかをチェックし、一致する場合にのみカーソルを進めるために使用されます
    fn match_to_expected(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn add_token_without_literal(&mut self, token_type: TokenType) {
        self.add_token(token_type, Literal::None);
    }

    fn add_token(&mut self, token_type: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token_type,
            String::from(text),
            literal,
            self.line,
        ))
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source
            .chars()
            .nth((self.current - 1).try_into().unwrap())
            .unwrap()
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error::scanner_error(self.line, "Unterminated string.");
        }

        self.advance();

        let value = String::from(self.source.get(self.start + 1..self.current - 1).unwrap());
        self.add_token(TokenType::STRING, Literal::String(value));
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let target_literal = self.source.get(self.start..self.current).unwrap();
        //println!("target_literal: {}", target_literal);

        let parsed_usize = target_literal.parse::<isize>();
        let parsed_float = target_literal.parse::<f64>();

        let literal = if parsed_usize.is_ok() {
            Literal::Isize(parsed_usize.ok().unwrap())
        } else if parsed_float.is_ok() {
            Literal::Float(parsed_float.ok().unwrap())
        } else {
            error::scanner_error(self.line, "Unexpected character.");
            panic!("")
        };
        self.add_token(TokenType::NUMBER, literal)
    }

    fn is_digit(&mut self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    //文字を消費しない先読み
    //現在の文字を返しますが、ファイルの終わりに達している場合はnull文字を返します
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        return self.source.chars().nth(self.current + 1).unwrap();
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source.get(self.start..self.current).unwrap();

        let keywords = self.keywords.clone();
        let token = keywords.get(text).unwrap_or(&TokenType::IDENTIFIER);
        self.add_token_without_literal(token.clone());
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => true,
            _ => false,
        }
    }
}
