use core::fmt;
use std::rc::Rc;

use crate::{error::Error, number::Num};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Bool(bool),
    Ident(String),
    Number(Num),
    Char(char),
    String(String),
    List(Vec<Expr>),
    Func(fn(&[Expr]) -> Result<Expr, Error>),
    Lambda(Lambda),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Expr::Nil => "nil".to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Ident(ident) => ident.clone(),
            Expr::Number(n) => n.to_string(),
            Expr::Char(c) => c.to_string(),
            Expr::String(s) => s.to_string(),
            Expr::List(list) => format!(
                "({})",
                list.iter()
                    .map(|i| i.to_string())
                    .intersperse(",".to_string())
                    .collect::<String>(),
            ),
            Expr::Func(_fun) => format!("Function {{}}"),
            Expr::Lambda(_l) => format!("Lambda {{}}"),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub args: Rc<Expr>,
    pub body: Rc<Expr>,
}

