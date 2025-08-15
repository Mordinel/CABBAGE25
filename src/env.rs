use std::{collections::HashMap, ops::{Add, Div, Mul, Sub}};

use crate::{error::Error, expr::Expr, number::Num};

macro_rules! comparative_binary_func {
    ($fname:expr, $check_fn:expr, $data:expr) => {
        $data.insert(
            $fname.to_string(), 
            Expr::Func($fname.to_string(), |args: &[Expr]| -> Result<Expr, Error> {
                let numbers = parse_list_of_numbers(args)?;
                let first = numbers.first()
                    .ok_or_else(|| Error::Reason(format!("({} n n ...): Missing first number.", $fname)))?;
                let rest = &numbers.get(1..)
                    .ok_or_else(|| Error::Reason(format!("({} n n ...): Missing second number.", $fname)))?;

                fn trunk(prev: &Num, rest: &[Num]) -> bool {
                    match rest.first() {
                        Some(fl) => $check_fn(prev, fl) && trunk(fl, &rest[1..]),
                        None => true,
                    }
                }
                Ok(Expr::Bool(trunk(first, rest)))
            }),
        );
    };
}

macro_rules! numeric_binary_func {
    ($fname:expr, $fold_closure:expr, $data:expr) => {
        $data.insert(
            $fname.to_string(),
            Expr::Func($fname.to_string(), |args: &[Expr]| -> Result<Expr, Error> {
                let numbers = parse_list_of_numbers(args)?;
                let first = numbers.first()
                    .ok_or_else(|| Error::Reason(format!("({} n n ...): Missing first number.", $fname)))?;
                let folded = numbers.get(1..)
                    .ok_or_else(|| Error::Reason(format!("({} n n ...): Missing second number.", $fname)))?.iter()
                    .fold(first.clone(), $fold_closure);
                Ok(Expr::Number(folded))
            }),
        )
    };
}

macro_rules! numeric_unary_func {
    ($fname:expr, $func:expr, $data:expr) => {
        $data.insert(
            $fname.to_string(),
            Expr::Func($fname.to_string(), |args: &[Expr]| -> Result<Expr, Error> {
                let numbers = parse_list_of_numbers(args)?;
                let first = numbers.first()
                    .ok_or_else(|| Error::Reason(format!("({} n): Missing first number.", $fname)))?;
                if args.len() > 1 {
                    return Error::Reason(format!("({} n): Can only have one argument.", $fname)).into();
                }
                Ok(Expr::Number($func(first.clone())))
            })
        );
    };
}

macro_rules! numeric_constant_func {
    ($fname:expr, $func:expr, $data:expr) => {
        $data.insert(
            $fname.to_string(),
            Expr::Func($fname.to_string(), |args: &[Expr]| -> Result<Expr, Error> {
                if !args.is_empty() {
                    return Error::Reason(format!("({}): No arguments allowed in a constant function.", $fname)).into();
                }
                Ok(Expr::Number($func()))
            })
        );
    };
}


#[derive(Debug)]
pub struct Env<'outer> {
    pub data: HashMap<String, Expr>,
    pub outer: Option<&'outer Env<'outer>>,
}

impl<'outer> Env<'outer> {
    pub fn get(&self, key: &str) -> Option<Expr> {
        match self.data.get(key) {
            Some(expr) => Some(expr.clone()),
            None => match self.outer {
                Some(outer) => outer.get(key),
                None => None,
            },
        }
    }
}

impl<'outer> Default for Env<'outer> {
    fn default() -> Self {
        let mut data = HashMap::new();

        data.insert(
            "print".to_string(),
            Expr::Func("print".into(), |args| {
                let first = args.first()
                    .ok_or_else(|| Error::reason("(print n): Missing first argument."))?;
                if args.len() > 1 {
                    return Error::reason("(print n): Can only have one argument.").into();
                }
                print!("{first}");
                Ok(Expr::Nil)
            }),
        );

        data.insert(
            "println".to_string(),
            Expr::Func("println".into(), |args| {
                let first = args.first()
                    .ok_or_else(|| Error::reason("(println n): Missing first argument."))?;
                if args.len() > 1 {
                    return Error::reason("(println n): Can only have one argument.").into();
                }
                println!("{first}");
                Ok(Expr::Nil)
            }),
        );

        comparative_binary_func!( "=", |a, b| a == b, data);
        comparative_binary_func!("!=", |a, b| a != b, data);
        comparative_binary_func!( "<", |a, b| a <  b, data);
        comparative_binary_func!("<=", |a, b| a <= b, data);
        comparative_binary_func!( ">", |a, b| a >  b, data);
        comparative_binary_func!(">=", |a, b| a >= b, data);

        numeric_binary_func!(  "+", |l, r| l.add(r.clone()), data);
        numeric_binary_func!(  "-", |l, r| l.sub(r.clone()), data);
        numeric_binary_func!(  "*", |l, r| l.mul(r.clone()), data);
        numeric_binary_func!(  "/", |l, r| l.div(r.clone()), data);
        numeric_binary_func!(  "^", |l, r| l        .pow(r), data);
        numeric_binary_func!("log", |l, r| l.log(r.clone()), data);

        numeric_unary_func! (   "ln", |n: Num| n   .ln(), data);
        numeric_unary_func! ( "log2", |n: Num| n .log2(), data);
        numeric_unary_func! ("log10", |n: Num| n.log10(), data);
        numeric_unary_func! ( "sqrt", |n: Num| n .sqrt(), data);
        numeric_unary_func! (  "exp", |n: Num| n  .exp(), data);
        numeric_unary_func! (  "abs", |n: Num| n  .abs(), data);

        numeric_constant_func!("pi", || Num::pi(), data);
        numeric_constant_func!( "e", || Num:: e(), data);

        Env { data, outer: None }
    }
}

fn parse_list_of_numbers(args: &[Expr]) -> Result<Vec<Num>, Error> {
    args.iter().map(parse_single_number).collect()
}

fn parse_single_number(exp: &Expr) -> Result<Num, Error> {
    match exp {
        Expr::Number(n) => Ok(n.clone()),
        _ => Error::reason("Expected a number").into(),
    }
}

