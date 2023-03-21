mod error;
use error::*;
mod token_type;
use token_type::*;
mod token;
use token::*;
mod scanner;
use scanner::*;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() > 2 {
        println!("Usage: rs-lox [script]");
        std::process::exit(64);
    } else if args.len() == 1 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
    println!("Hello, world!");
}

fn run_file(path: &String) -> io::Result<()> {
    let buf = std::fs::read_to_string(path)?;
    match run(buf) {
        Ok(_) => {}
        Err(m) => {
            m.report("".to_string());
            std::process::exit(65);
        }
    }
    Ok(())
}

fn run_prompt() {
    let stdin = io::stdin();
    print!("> ");
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if line.is_empty() {
                break;
            }
            match run(line) {
                Ok(_) => {}
                Err(m) => {
                    m.report("".to_string());
                    std::process::exit(65);
                }
            }
        } else {
            break;
        }
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
