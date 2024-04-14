mod ast_printer;
mod callable;
mod environment;
mod error;
mod expr;
mod interpreter;
mod lox_class;
mod lox_instance;
mod object;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    process::exit,
};

use interpreter::Interpreter;

use crate::{
    error::{Error, Result},
    resolver::Resolver,
};

fn main() {
    //run_prompt();
    let path = String::from(r"C:\Users\oasis\OneDrive\Desktop\RUST\rust_lox\src\source.txt");
    run_file(&path).unwrap();
}

fn run_file(path: &str) -> io::Result<()> {
    let f = File::open(path)?;
    let f = BufReader::new(f);
    let mut source = String::from("");
    let mut interpreter = interpreter::Interpreter::new();

    for line in f.lines() {
        source.push_str(&line.unwrap());
        source.push_str("\n")
    }

    if let Err(e) = run(&source, &mut interpreter) {
        eprintln!("{:?}", e);
        exit(70);
    };

    Ok(())
}

fn run_prompt() {
    let mut interpreter = interpreter::Interpreter::new();

    loop {
        print!("> ");
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        if line.is_empty() {
            break;
        }
        if let Err(e) = run(&line, &mut interpreter) {
            eprintln!("{:?}", e);
        };
    }
}

fn run(source: &str, interpreter: &mut Interpreter) -> Result<()> {
    //let mut had_error = false;
    let mut scanner = scanner::Scanner::new(source.to_string());
    let tokens = scanner.scan_tokens();
    for token in tokens.clone() {
        println!("token:{}", token);
    }
    let mut parser = parser::Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(result) => result,
        _ => return Err(Error::ParseError(String::from("parse error"))),
    };

    //println!("statements:{:#?}", statements);

    let mut resolver = Resolver::new(interpreter);
    resolver.resolve_statements(&statements)?;

    let _ = match interpreter.interpret(statements.clone()) {
        Ok(result) => result,
        _ => return Err(Error::ParseError(String::from("interpreter error"))),
    };

    // let mut ast_printer = ast_printer::AstPrinter {};
    // let result = ast_printer.print_statements(statements.clone());
    // println!("{}", result);
    Ok(())
}
