extern crate core;

use std::env::args;
use std::io::{self, stdout, Write};
use std::rc::Rc;

// mod ast_printer;
mod callable;
mod environment;
mod error;
mod expr;
mod interpreter;
mod lox_function;
mod native_functions;
mod object;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;

use crate::resolver::Resolver;
use error::*;
use interpreter::*;
use parser::*;
use scanner::*;

fn main() {
    let args: Vec<String> = args().collect();
    let lox = Lox::new();
    match args.len() {
        1 => {
            lox.run_prompt();
        }
        2 => {
            lox.run_file(&args[1]).expect("Error: something is wrong");
        }
        _ => {
            println!("Usage: rs-lox [script]");
            std::process::exit(64);
        }
    }
}

struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&self, path: &str) -> io::Result<()> {
        let buf = std::fs::read_to_string(path)?;
        if self.run(buf).is_err() {
            std::process::exit(65);
        }
        Ok(())
    }

    pub fn run_prompt(&self) {
        let stdin = io::stdin();
        print!("> ");
        let _ = stdout().flush();
        for line in stdin.lines() {
            if let Ok(line) = line {
                if line.is_empty() {
                    println!("Please enter something to execute");
                    print!("> ");
                    let _ = stdout().flush();
                    continue;
                }

                if self.run(line).is_err() {}
            } else {
                break;
            }
            print!("> ");
            let _ = stdout().flush();
        }
    }

    pub fn run(&self, source: String) -> Result<(), LoxResult> {
        if source == "@" {
            self.interpreter.print_environment();
            return Ok(());
        }

        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;

        // for token in tokens {
        //     println!("{:?}", token);
        // }
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        if parser.success() {
            let resolver = Resolver::new(&self.interpreter);
            resolver.resolve(&Rc::new(statements.clone()))?;
            if resolver.success() {
                self.interpreter.interpret(statements);
                Ok(())
            } else {
                Err(LoxResult::error(0, ""))
            }
        } else {
            Err(LoxResult::error(0, ""))
        }
    }
}
