#![feature(iter_intersperse)]

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::ops::Not;
use std::path::PathBuf;
use std::process;

use expr::Expr;

mod error;
mod expr;
mod env;
mod lex;
mod parse;
mod eval;
mod number;
mod unescape;
mod str_ext;

fn parse_eval(expr: &str, env: &mut env::Env) -> Result<expr::Expr, error::Error> {
    let (parsed_exp, _) = parse::parse(&lex::lex("<stdin>", expr))?;
    let evaled_exp = eval::eval(&parsed_exp, env)?;
    Ok(evaled_exp)
}

fn parse_eval_all(path: &str, expr: &str, env: &mut env::Env) -> Result<expr::Expr, error::Error> {
    let tokens = lex::lex(path, expr);
    let (mut parsed_exp, mut rest) = parse::parse(&tokens)?;
    loop {
        let result = eval::eval(&parsed_exp, env)?;
        if rest.is_empty() {
            return Ok(result);
        }
        (parsed_exp, rest) = parse::parse(&rest)?;
    }
}

fn slurp_expr() -> String {
    let mut expr = String::new();
    io::stdout().flush()
        .expect("Failed to flush stdout.");
    io::stdin().read_line(&mut expr)
        .expect("Failed to read line.");
    expr
}

fn no_terminal() -> ! {
    eprintln!("Please use a terminal/tty for the repl.");
    process::exit(1);
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.is_empty() {
        eprintln!("Invoke this program normally, idiot.");
        process::exit(1);
    }
    if args.len() == 1 {
        let program_name = PathBuf::from(args[0].clone())
            .file_name().expect("Bad file name.")
            .to_str().expect("Not Unicode.")
            .to_string();

        io::stdin().is_terminal().not().then(no_terminal);
        io::stdout().is_terminal().not().then(no_terminal);
        io::stderr().is_terminal().not().then(no_terminal);

        let env = &mut env::default_env();
        loop {
            print!("({program_name})> ");
            let expr = slurp_expr();
            if expr.trim().is_empty() {
                continue;
            }
            match parse_eval(&expr, env) {
                Ok(Expr::Nil) => (),
                Ok(res) => println!("{res}\n"),
                Err(err) => match err {
                    error::Error::Reason(reason) => eprintln!("Error: {reason}"),
                }
            }
        }
    }

    if args.len() == 2 {
        let path = &args[1];

        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(why) => {
                eprintln!("{why}");
                process::exit(1);
            },
        };
        let env = &mut env::default_env();
        match parse_eval_all(path, contents.trim(), env) {
            Ok(_) => (),
            Err(err) => match err {
                error::Error::Reason(reason) => eprintln!("Error: {reason}"),
            }
        }
    }
}

