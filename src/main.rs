extern crate core;

use std::env::args;
use std::io::{self, stdout, BufRead, Write};

mod ast_printer;
mod error;
mod expr;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod token;
mod token_type;

use ast_printer::*;
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
            interpreter: Interpreter {},
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
        stdout().flush().expect("Error: flush Error");
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                if line.is_empty() {
                    break;
                }

                if self.run(line).is_err() {}
            } else {
                break;
            }
            print!("> ");
            stdout().flush().expect("Error: flush Error");
        }
    }

    pub fn run(&self, source: String) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;

        let mut parser = Parser::new(tokens);
        match parser.parse() {
            None => {}
            Some(expr) => {
                // let printer = AstPrinter {};
                // println!("AST Printer:\n{}", printer.print(&expr)?);
                self.interpreter.interpret(&expr);
            }
        }
        Ok(())
    }
}
