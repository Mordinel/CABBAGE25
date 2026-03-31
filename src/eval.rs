
use std::{fs, rc::Rc};
use crate::lex;
use crate::lex::Token;
use crate::parse::{Parser};
use crate::env::Env;
use crate::expr::{Expr, Lambda};
use crate::error::Error;

pub struct Eval<'src> {
    parser: Parser<'src>,
    env: Env<'src>,
}

impl<'src> Eval<'src> {
    pub fn new(parser: Parser<'src>, env: Env<'src>) -> Eval<'src> {
        Eval { parser, env }
    }
}

fn env_get(key: &str, env: &Env) -> Option<Expr> {
    match env.data.get(key) {
        Some(exp) => Some(exp.clone()),
        None => {
            match &env.outer {
                Some(outer_env) => env_get(key, outer_env),
                None => None,
            }
        }
    }
}

pub fn eval(exp: &Expr, env: &mut Env) -> Result<Expr, Error> {
    use Expr::*;
    match exp {
        Ident(key) => {
            env_get(key, env)
                .ok_or_else(|| Error::Reason(format!("eval: unexpected symbol key='{key}'")))
                .map(|xpr| xpr.clone())
        },

        Char(_) => Ok(exp.clone()),
        String(_) => Ok(exp.clone()),
        Number(_) => Ok(exp.clone()),
        Bool(_) => Ok(exp.clone()),
        Nil => Ok(exp.clone()),

        List(list) => {
            let first_form = list.first()
                .ok_or_else(|| Error::reason("eval: Expected a non-empty list"))?;

            let arg_forms = &list[1..];
            match eval_built_in_form(first_form, arg_forms, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first_form, env)?;
                    match first_eval {
                        Func(_name, f) => {
                            let evaluated_args = eval_forms(arg_forms, env)?;
                            f(&evaluated_args)
                        },

                        Lambda(l) => {
                            eval(
                                &l.body,
                                &mut env_for_lambda(l.args, arg_forms, env)?
                            )
                        },

                        _ => Ok(Expr::List(eval_forms(&list, env)?)),
                    }
                }
            }
        },

        Func(_, _) => Ok(exp.clone()),
        Lambda(_)  => Ok(exp.clone()),
    }
}

fn env_for_lambda<'outer>(
    params: Rc<Expr>,
    arg_forms: &[Expr],
    outer_env: &'outer mut Env,
) -> Result<Env<'outer>, Error> {
    let keys = Parser::parse_list_of_symbol_strings(params)?;

    let mut values = eval_forms(arg_forms, outer_env)?;
    let mut real_args = vec![];
    if let [Expr::List(inner)] = values.as_slice() {
        real_args.extend_from_slice(inner);
        values = real_args;
    }
    if keys.len() != values.len() {
        return Error::Reason(format!("env_for_lambda: expected {} arguments, got {}. Params: {:?}, Args: {:?}",
                keys.len(), values.len(), keys, values)).into();
    }

    Ok(Env {
        data: keys.iter()
            .zip(values.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        outer: Some(outer_env),
    })
}

fn eval_forms(arg_forms: &[Expr], env: &mut Env) -> Result<Vec<Expr>, Error> {
    arg_forms.iter()
        .map(|form| eval(form, env))
        .collect()
}

fn eval_built_in_form(
    exp: &Expr,
    arg_forms: &[Expr],
    env: &mut Env,
) -> Option<Result<Expr, Error>> {
    match exp {
        Expr::Ident(i) => match i.as_ref() {
            "do" => Some(eval_do_args(arg_forms, env)),
            "if" => Some(eval_if_args(arg_forms, env)),
            "let" => Some(eval_let_args(arg_forms, env)),
            "src" => Some(eval_src_args(arg_forms, env)),
            "cat" => Some(eval_cat_args(arg_forms, env)),
            "fn" => Some(eval_lambda_args(arg_forms)),
            _ => None,
        } ,
        _ => None,
    }
}

fn eval_cat_args(
    arg_forms: &[Expr],
    env: &mut Env,
) -> Result<Expr, Error> {
    if arg_forms.is_empty() {
        return Error::Reason("eval_cat_args: Expected at least one list.".to_string()).into();
    }

    let strings = eval_into_strings(arg_forms, env)?;
    let concatenated = strings.iter()
        .fold(String::new(), |acc, s| acc + &s);

    Ok(Expr::String(concatenated))
}

fn eval_into_strings(
    arg_forms: &[Expr],
    env: &mut Env,
) -> Result<Vec<String>, Error> {
    let paths = arg_forms.iter()
        .map(|exp| eval(exp, env)
            .map(|res| res.to_string()))
        .collect::<Vec<_>>();
    if paths.iter().all(|p| p.is_ok()) {
        Ok(paths.into_iter()
            .map(|p| p.unwrap())
            .collect())
    } else {
        Error::Reason(format!("eval_into_strings: Not all expressions result in strings: {paths:?}")).into()
    }
}

fn eval_do_args(
    arg_forms: &[Expr],
    env: &mut Env,
) -> Result<Expr, Error> {
    if arg_forms.is_empty() {
        return Error::Reason("eval_do_args: Expected at least one list.".to_string()).into();
    }

    let mut eval_res = None;
    for exp in arg_forms.iter() {
        match exp {
            Expr::List(_) => (),
            _ => return Error::Reason("eval_do_args: Expected only list expressions.".to_string()).into(),
        }
        eval_res = Some(eval(exp, env));
    }
    if let Some(res) = eval_res {
        res
    } else {
        Error::Reason("eval_do_args: No expressions.".to_string()).into()
    }
}

fn eval_src_args(
    arg_forms: &[Expr],
    env: &mut Env,
) -> Result<Expr, Error> {
    if arg_forms.is_empty() {
        return Error::Reason("eval_src_args: Expected at least one form.".to_string()).into();
    }

    let mut paths = Vec::with_capacity(arg_forms.len());

    for exp in arg_forms.iter() {
        let path = eval(exp, env)?.to_string();
        paths.push(path);
    }

    for path in paths.iter() {
        let contents = match fs::read_to_string(path) {
            Ok(cont) => cont,
            Err(why) => return Error::Reason(format!("eval_src_args: could not read file `{path}`: {why}")).into(),
        };
        let tokens = lex::lex(path, &contents);
        let mut parser = Parser::new(&contents);
        if let Err(why) = eval_all(&mut parser, &tokens, env) {
            return Error::Reason(format!("eval_src_args: {why}")).into();
        }
    }

    Ok(Expr::Nil)
}

fn eval_all(parser: &mut Parser, tokens: &[Token], env: &mut Env) -> Result<(), Error> {
    let mut remaining = tokens;
    loop {
        let (exp, rest) = match parser.parse(remaining) {
            Ok(p) => p,
            Err(why) => return Error::Reason(
                format!("eval_all: could not parse program: {why}")
            ).into(),
        };
        let _ = eval(&exp, env)?;
        if rest.is_empty() {
            break;
        }
        remaining = rest;
    }
    Ok(())
}

fn eval_lambda_args(arg_forms: &[Expr]) -> Result<Expr, Error> {
    let params = arg_forms.first()
        .ok_or_else(|| Error::reason("eval_lambda_args: Expected args form."))?;

    let body = arg_forms.get(1)
        .ok_or_else(|| Error::reason("eval_lambda_args: Expected second form."))?;

    if arg_forms.len() > 2 {
        return Error::reason("eval_lambda_args: can only have two forms.").into();
    }

    Ok(Expr::Lambda(
        Lambda {
            body: Rc::new(body.clone()),
            args: Rc::new(params.clone()),
        }
    ))
}

fn eval_if_args(
    arg_forms: &[Expr],
    env: &mut Env,
) -> Result<Expr, Error> {
    if arg_forms.len() < 2 {
        return Error::reason("eval_if_args: Expected at least test and then branch").into();
    }

    let test_form = &arg_forms[0];
    let then_form = &arg_forms[1];
    let else_form = arg_forms.get(2).unwrap_or(&Expr::Nil);

    let test_eval = eval(test_form, env)?;

    match test_eval {
        Expr::Bool(true) => eval(then_form, env),
        Expr::Bool(false) => eval(else_form, env),
        _ => Error::Reason(format!("eval_if_args: test did not evaluate to bool, got {:?}", test_eval)).into(),
    }
}

fn eval_let_args(
    arg_forms: &[Expr],
    env: &mut Env,
) -> Result<Expr, Error> {
    let first_form = arg_forms.first()
        .ok_or_else(|| Error::reason("eval_let_args: Expected first form."))?;

    let first_str = match first_form {
        Expr::Ident(i) => Ok(i.clone()),
        _ => Error::reason("eval_let_args: Expected first form to be an ident").into(),
    }?;

    let second_form = arg_forms.get(1)
        .ok_or_else(|| Error::reason("eval_let_args: Expected second form."))?;

    if arg_forms.len() > 2 {
        return Error::reason("eval_let_args: can only have two forms.").into();
    }

    let second_eval = eval(second_form, env)?;
    env.data.insert(first_str, second_eval);

    Ok(first_form.clone())
}

