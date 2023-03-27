extern crate core;

use std::env::args;
use std::io::{self, stdout, BufRead, Write};

mod error;
use error::*;
mod token_type;

mod token;

mod scanner;
use scanner::*;

fn main() {
    let args: Vec<String> = args().collect();

    match args.len() {
        1 => {
            run_prompt();
        }
        2 => {
            run_file(&args[1]).expect("Error: something is wrong");
        }
        _ => {
            println!("Usage: rs-lox [script]");
            std::process::exit(64);
        }
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let buf = std::fs::read_to_string(path)?;
    if run(buf).is_err() {
        std::process::exit(65);
    }
    Ok(())
}

fn run_prompt() {
    let stdin = io::stdin();
    print!("> ");
    stdout().flush().expect("Error: flush Error");
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if line.is_empty() {
                break;
            }

            if run(line).is_err() {}
        } else {
            break;
        }
        print!("> ");
        stdout().flush().expect("Error: flush Error");
    }
}

fn run(source: String) -> Result<(), LoxError> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}
