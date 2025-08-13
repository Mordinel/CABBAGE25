use std::collections::HashMap;

use malachite::rational::Rational;

use crate::{error::Error, expr::Expr, number::Num};

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {
        |args: &[Expr]| -> Result<Expr, Error> {

            let numbers = parse_list_of_numbers(args)?;
            let first = numbers.first()
                .ok_or(Error::Reason("Expected at least one number".to_string()))?;

            let rest = &numbers[1..];

            fn trunk(prev: &Number, rest: &[Number]) -> bool {
                match rest.first() {
                    Some(fl) => $check_fn(prev, fl) && trunk(fl, &rest[1..]),
                    None => true,
                }
            }
            Ok(Expr::Bool(trunk(first, rest)))
        }
    };
}

#[derive(Debug)]
pub struct Env<'outer> {
    pub data: HashMap<String, Expr>,
    pub outer: Option<&'outer Env<'outer>>
}

pub fn default_env<'outer>() -> Env<'outer> {
    let mut data = HashMap::new();

    data.insert(
        "print".to_string(),
        Expr::Func(|args| {
            let first = args.first()
                .ok_or_else(|| Error::Reason("print: Expected at least one form.".to_string()))?;
            if args.len() > 1 {
                return Error::Reason("print: can only have one form.".to_string()).into();
            }
            print!("{first}");
            Ok(Expr::Nil)
        })
    );

    data.insert(
        "println".to_string(),
        Expr::Func(|args| {
            let first = args.first()
                .ok_or_else(|| Error::Reason("print: Expected at least one form.".to_string()))?;
            if args.len() > 1 {
                return Error::Reason("print: can only have one form.".to_string()).into();
            }
            println!("{first}");
            Ok(Expr::Nil)
        })
    );

    data.insert(
        "+".to_string(),
        Expr::Func(|args| {
            let sum = parse_list_of_numbers(args)?.iter()
                .fold(Num::Rat(Rational::from(0)), |acc, n| acc + n.clone());
            Ok(Expr::Number(sum))
        })
    );

    data.insert(
        "-".to_string(),
        Expr::Func(|args| {
            let numbers = parse_list_of_numbers(args)?;
            let first = numbers.first()
                .ok_or_else(|| Error::Reason("sub('-'): Expected at least one number.".to_string()))?;
            let sum_of_rest: Num = numbers[1..].iter()
                .fold(Num::Rat(Rational::from(0)), |acc, n| acc + n.clone());
            Ok(Expr::Number(first.clone() - sum_of_rest))
        })
    );

    data.insert(
        "*".to_string(),
        Expr::Func(|args| {
            let product = parse_list_of_numbers(args)?.iter()
                .fold(Num::Rat(Rational::from(0)), |acc, n| acc * n.clone());
            Ok(Expr::Number(product))
        })
    );

    data.insert(
        "/".to_string(),
        Expr::Func(|args| {
            let numbers = parse_list_of_numbers(args)?;
            let first = numbers.first()
                .ok_or_else(|| Error::Reason("div('/'): Expected at least one number.".to_string()))?;
            let divided = numbers[1..].iter()
                .fold(first.clone(), |acc, n| acc / n.clone());
            Ok(Expr::Number(divided))
        })
    );

    //data.insert(
    //    "pow".to_string(),
    //    Expr::Func(|args| {
    //        let numbers = parse_list_of_numbers(args)?;
    //        let first = *numbers.first()
    //            .ok_or(Error::Reason("pow: Expected at least one number.".to_string()))?;
    //        let second = *numbers.get(1)
    //            .ok_or(Error::Reason("pow: Expected second number.".to_string()))?;
    //        if args.len() > 2 {
    //            return Error::Reason("pow: can only have two forms.".to_string()).into();
    //        }
    //        Ok(Expr::Number(first.powf(second)))
    //    })
    //);

    //data.insert(
    //    "sqrt".to_string(),
    //    Expr::Func(|args| {
    //        let numbers = parse_list_of_numbers(args)?;
    //        let first = *numbers.first()
    //            .ok_or(Error::Reason("sqrt: Expected at least one number.".to_string()))?;
    //        if args.len() > 1 {
    //            return Error::Reason("sqrt: can only have one form.".to_string()).into();
    //        }
    //        Ok(Expr::Number(first.sqrt()))
    //    })
    //);

    data.insert(
        "=".to_string(),
        Expr::Func(ensure_tonicity!(|a, b| a == b))
    );
    data.insert(
        ">".to_string(),
        Expr::Func(ensure_tonicity!(|a, b| a > b))
    );
    data.insert(
        ">=".to_string(),
        Expr::Func(ensure_tonicity!(|a, b| a >= b))
    );
    data.insert(
        "<".to_string(),
        Expr::Func(ensure_tonicity!(|a, b| a < b))
    );
    data.insert(
        "<=".to_string(),
        Expr::Func(ensure_tonicity!(|a, b| a <= b))
    );

    Env { data, outer: None }
}

fn parse_list_of_numbers(args: &[Expr]) -> Result<Vec<Num>, Error> {
    args.iter()
        .map(parse_single_number)
        .collect()
}

fn parse_single_number(exp: &Expr) -> Result<Num, Error> {
    match exp {
        Expr::Number(n) => Ok(n.clone()),
        _ => Error::reason("Expected a number").into(),
    }
}

