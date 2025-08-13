use std::rc::Rc;

use crate::error::Error;
use crate::expr::Expr;
use crate::number::Num;
use crate::lex::{Base, LiteralKind, Token, TokenKind};
use crate::unescape::{unescape_char, unescape_unicode, Mode};

pub fn parse<'tk>(tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
    let (token, rest) = tokens.split_first()
        .ok_or_else(|| Error::Reason("Could not get token".to_string()))?;
    match token.kind {
        TokenKind::OpenParen => read_seq(rest),
        TokenKind::CloseParen => Error::Reason("Unexpected `)`.".to_string()).into(),
        _ => Ok((parse_atom(token)?, rest)),
    }
}

pub fn parse_list_of_symbol_strings(form: Rc<Expr>) -> Result<Vec<String>, Error> {
    match form.as_ref() {
        Expr::List(l) => Ok(l.clone()),
        _ => Error::Reason("Expected args form to be a list.".to_string()).into(),
    }?
    .iter()
        .map(|exp| match exp {
            Expr::Ident(i) => Ok(i.clone()),
            _ => Error::Reason("Expected identifiers in the argument list.".to_string()).into(),
        })
    .collect()
}

fn read_seq<'tk>(tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
    let mut list_items: Vec<Expr> = vec![];
    let mut excess = tokens;
    loop {
        let (next_token, rest) = excess.split_first()
            .ok_or_else(|| Error::Reason("seq: Could not find closing `)`.".to_string()))?;

        match next_token.kind {
            TokenKind::CloseParen => 
                return Ok((Expr::List(list_items), rest)), // skip `)`, head to the token after

            TokenKind::WhiteSpace | TokenKind::Newline => {
                let (_, new_excess) = excess.split_first()
                    .ok_or_else(|| Error::reason("seq: Could not find closing `)`."))?;
                excess = new_excess;
                continue
            },
            _ => (),
        }
        let (item, new_excess) = parse(&excess)?;
        list_items.push(item);
        excess = new_excess;
    }
}

fn parse_literal(token: &Token) -> Result<Expr, Error> {
    match token.kind {
        TokenKind::Literal {
            kind: LiteralKind::Int { base, empty_int }
        } => {
            if empty_int {
                return Ok(Expr::Number(Num::Int(0)));
            }
            if let Ok(n) = i64::from_str_radix(&token.data, match base {
                Base::Binary => 2,
                Base::Octal => 8,
                Base::Decimal => 10,
                Base::Hexadecimal => 16,
            }) {
                return Ok(Expr::Number(Num::Int(n)));
            }
            return Error::reason("literal: Failed to parse Integer.").into();
        },
        TokenKind::Literal {
            kind: LiteralKind::Float { base, .. }
        } => {
            if base != Base::Decimal {
                return Error::reason("literal: Can only parse floats as decimal.").into();
            }
            if let Ok(n) = token.data.parse::<Float>() {
                return Ok(Expr::Number(Num::Fp()));
            }
            return Error::reason("literal: Failed to parse float.").into();
        },
        TokenKind::Literal {
            kind: LiteralKind::Char { terminated }
        } => {
            if !terminated {
                return Error::reason("literal: Unterminated char quote.").into();
            }

            let c = match unescape_char(&token.data) {
                Ok(c) => c,
                Err(e) => return Error::Reason(
                    format!("literal: failed to unescape char-string: {e:?}")
                ).into(),
            };

            return Ok(Expr::Char(c));
        },
        TokenKind::Literal {
            kind: LiteralKind::Str { terminated }
        } => {
            if !terminated {
                return Error::reason("literal: Unterminated string quote.").into();
            }
            let len = token.data.len();
            let mut buf = Ok(Vec::with_capacity(len));
            unescape_unicode(&token.data[1..(len-1)], Mode::Str, &mut |range, c| {
                if let Ok(b) = &mut buf {
                    match c {
                        Ok(c) => b.push(c),
                        Err(e) => buf = Err((range, e)),
                    }
                }
            });
            let str = match buf {
                Ok(b) => b.iter().collect::<String>(),
                Err((_, e)) => return Error::Reason(
                    format!("literal: failed to unescape string: {e:?}")
                ).into(),
            };
            return Ok(Expr::String(str));
        },
        _ => todo!(),
    }
}

fn parse_atom(token: &Token) -> Result<Expr, Error> {
    match token.kind {
        TokenKind::Nil => Ok(Expr::Nil),
        TokenKind::True => Ok(Expr::Bool(true)),
        TokenKind::False => Ok(Expr::Bool(false)),
        TokenKind::Literal { .. } => Ok(parse_literal(token)?),
        TokenKind::Ident => Ok(Expr::Ident(token.data.clone())),
        _ => Ok(Expr::Ident(token.data.clone())),
    }
}

