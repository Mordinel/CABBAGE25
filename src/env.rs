use std::collections::HashMap;

use crate::{error::Error, expr::Expr, number::Num};

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {
        |args: &[Expr]| -> Result<Expr, Error> {
            let numbers = parse_list_of_numbers(args)?;
            let first = numbers
                .first()
                .ok_or(Error::Reason("Expected at least one number".to_string()))?;

            let rest = &numbers[1..];

            fn trunk(prev: &Num, rest: &[Num]) -> bool {
                match rest.first() {
                    Some(fl) => $check_fn(prev, fl) && trunk(fl, &rest[1..]),
                    None => true,
                }
            }
            Ok(Expr::Bool(trunk(first, rest)))
        }
    };
}
    //data.insert(
    //    "=".to_string(),
    //    Expr::Func("=".into(), ensure_tonicity!(|a, b| a == b)),
    //);

macro_rules! numeric_binary_func {
    ($fname:expr, $fold_closure:expr, $data:expr) => {
        $data.insert(
            $fname.to_string(),
            Expr::Func($fname.to_string(), |args: &[Expr]| -> Result<Expr, Error> {
                let numbers = parse_list_of_numbers(args)?;
                let first = numbers.first().ok_or_else(|| {
                    Error::Reason(format!("({} n n ...): Missing first number.", $fname))
                })?;
                let folded = numbers
                    .get(1..)
                    .ok_or_else(|| {
                        Error::Reason(format!("({} n n ...): Missing second number.", $fname))
                    })?
                .iter()
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
                    .ok_or_else(|| Error::Reason(format!("({} n):Missing first number.", $fname)))?;
                if args.len() > 1 {
                    return Error::Reason(format!("({} n): Can only have one argument.", $fname)).into();
                }
                Ok(Expr::Number($func(first.clone())))
            })
        );
    };
}

#[derive(Debug)]
pub struct Env<'outer> {
    pub data: HashMap<String, Expr>,
    pub outer: Option<&'outer Env<'outer>>,
}

pub fn default_env<'outer>() -> Env<'outer> {
    let mut data = HashMap::new();

    data.insert(
        "print".to_string(),
        Expr::Func("print".into(), |args| {
            let first = args
                .first()
                .ok_or_else(|| Error::Reason("print: Expected at least one form.".to_string()))?;
            if args.len() > 1 {
                return Error::Reason("print: can only have one form.".to_string()).into();
            }
            print!("{first}");
            Ok(Expr::Nil)
        }),
    );

    data.insert(
        "println".to_string(),
        Expr::Func("println".into(), |args| {
            let first = args
                .first()
                .ok_or_else(|| Error::Reason("print: Expected at least one form.".to_string()))?;
            if args.len() > 1 {
                return Error::Reason("print: can only have one form.".to_string()).into();
            }
            println!("{first}");
            Ok(Expr::Nil)
        }),
    );

    data.insert(
        "=".to_string(),
        Expr::Func("=".into(), ensure_tonicity!(|a, b| a == b)),
    );
    data.insert(
        ">".to_string(),
        Expr::Func(">".into(), ensure_tonicity!(|a, b| a > b)),
    );
    data.insert(
        ">=".to_string(),
        Expr::Func(">=".into(), ensure_tonicity!(|a, b| a >= b)),
    );
    data.insert(
        "<".to_string(),
        Expr::Func("<".into(), ensure_tonicity!(|a, b| a < b)),
    );
    data.insert(
        "<=".to_string(),
        Expr::Func("<=".into(), ensure_tonicity!(|a, b| a <= b)),
    );

    numeric_binary_func!(    "+", |l, r| l + r.clone(),    data);
    numeric_binary_func!(    "-", |l, r| l - r.clone(),    data);
    numeric_binary_func!(    "*", |l, r| l * r.clone(),    data);
    numeric_binary_func!(    "/", |l, r| l / r.clone(),    data);
    numeric_binary_func!(    "^", |l, r| l.pow(r),         data);
    numeric_binary_func!(  "log", |l, r| l.log(r.clone()), data);

    numeric_unary_func! (   "ln", |n: Num| n   .ln(),      data);
    numeric_unary_func! ( "log2", |n: Num| n .log2(),      data);
    numeric_unary_func! ("log10", |n: Num| n.log10(),      data);
    numeric_unary_func! ( "sqrt", |n: Num| n .sqrt(),      data);
    numeric_unary_func! (  "exp", |n: Num| n  .exp(),      data);

    Env { data, outer: None }
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
