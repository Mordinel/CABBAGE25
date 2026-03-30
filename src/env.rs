use std::{collections::HashMap, ops::{Add, Div, Mul, Neg, Sub}};

use crate::{error::Error, expr::Expr};
use number::Number;

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

                fn trunk(prev: &Number, rest: &[Number]) -> bool {
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

        comparative_binary_func!( "=", |a, b| a == b, data);
        comparative_binary_func!("!=", |a, b| a != b, data);
        comparative_binary_func!( "<", |a, b| a <  b, data);
        comparative_binary_func!("<=", |a, b| a <= b, data);
        comparative_binary_func!( ">", |a, b| a >  b, data);
        comparative_binary_func!(">=", |a, b| a >= b, data);

        numeric_binary_func!(    "+", |l, r| l.add(r.clone()), data);
        numeric_binary_func!(    "-", |l, r| l.sub(r.clone()), data);
        numeric_binary_func!(    "*", |l, r| l.mul(r.clone()), data);
        numeric_binary_func!(    "/", |l, r| l.div(r.clone()), data);
        numeric_binary_func!(    "^", |l, r| l.pow(r.clone()), data);
        numeric_binary_func!(  "log", |l, r| l.log_n(r.clone()), data);
        numeric_binary_func!("atan2", |l, r| l.div(r.clone()).atan(), data);

        numeric_unary_func! (   "ln", |n: Number| n   .ln(), data);
        numeric_unary_func! ( "log2", |n: Number| n .log2(), data);
        numeric_unary_func! ("log10", |n: Number| n.log10(), data);
        numeric_unary_func! ( "sqrt", |n: Number| n .sqrt(), data);
        numeric_unary_func! (  "exp", |n: Number| n  .exp(), data);
        numeric_unary_func! (  "abs", |n: Number| n  .abs(), data);
        numeric_unary_func! (    "-", |n: Number| n  .neg(), data);

        numeric_unary_func! ("sin", |n: Number| n  .sin(), data);
        numeric_unary_func! ("tan", |n: Number| n  .tan(), data);
        numeric_unary_func! ("sec", |n: Number| n  .sec(), data);
        numeric_unary_func! ("cos", |n: Number| n  .cos(), data);
        numeric_unary_func! ("cot", |n: Number| n  .cot(), data);
        numeric_unary_func! ("csc", |n: Number| n  .csc(), data);

        numeric_unary_func! ("asin", |n: Number| n  .asin(), data);
        numeric_unary_func! ("atan", |n: Number| n  .atan(), data);
        numeric_unary_func! ("asec", |n: Number| n  .asec(), data);
        numeric_unary_func! ("acos", |n: Number| n  .acos(), data);
        numeric_unary_func! ("acot", |n: Number| n  .acot(), data);
        numeric_unary_func! ("acsc", |n: Number| n  .acsc(), data);

        numeric_constant_func!("pi", || Number::pi(), data);
        numeric_constant_func!( "e", || Number:: e(), data);
        numeric_constant_func!( "i", || Number:: i(), data);

        let mut me = Env { data, outer: None };

        me.data.insert(
            "print".to_string(),
            Expr::Func("print".into(), |args| {
                let s = args.iter()
                    .map(|x| x.to_string())
                    .collect::<String>();
                print!("{s}");
                Ok(Expr::Nil)
            }),
        );

        me.data.insert(
            "println".to_string(),
            Expr::Func("println".into(), |args| {
                let s = args.iter()
                    .map(|x| x.to_string())
                    .collect::<String>();
                println!("{s}");
                Ok(Expr::Nil)
            }),
        );

        me
    }
}

fn parse_list_of_numbers(args: &[Expr]) -> Result<Vec<Number>, Error> {
    args.iter().map(parse_single_number).collect()
}

fn parse_single_number(exp: &Expr) -> Result<Number, Error> {
    match exp {
        Expr::Number(n) => Ok(n.clone()),
        _ => Error::reason("Expected a number").into(),
    }
}

