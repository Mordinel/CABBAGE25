use std::rc::Rc;

use crate::error::Error;
use crate::expr::Expr;
use crate::number::Num;
use crate::lex::{Base, Cursor, LiteralKind, Token, TokenKind};
use crate::str_ext::Substr;
use crate::unescape::{unescape_char, unescape_unicode, Mode};

pub struct Parser<'src> {
    cursor: Cursor<'src>,
    source: &'src str,
}

impl<'src> Parser<'src> {
    pub fn new(file_name: &'src str, source: &'src str) -> Parser<'src> {
        Parser {
            cursor: Cursor::<'src>::new(file_name, source),
            source,
        }
    }

    pub fn parse<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (token, rest) = tokens.split_first()
            .ok_or_else(|| Error::Reason("Could not get token".to_string()))?;
        match token.kind {
            TokenKind::OpenParen => self.read_seq(rest),
            TokenKind::CloseParen => Error::Reason("Unexpected `)`.".to_string()).into(),
            _ => Ok((self.parse_atom(token)?, rest)),
        }
    }

    pub fn parse_list_of_symbol_strings(&mut self, form: Rc<Expr>) -> Result<Vec<String>, Error> {
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

    fn read_seq<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
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
            let (item, new_excess) = self.parse(&excess)?;
            list_items.push(item);
            excess = new_excess;
        }
    }

    fn parse_literal(&mut self, token: &Token) -> Result<Expr, Error> {
        let (start, len) = token.pos();
        let substr = self.source.substr(start, len).unwrap();
        match token.kind {
            TokenKind::Literal {
                kind: LiteralKind::Int { base, empty_int }
            } => {
                if empty_int {
                    return Ok(Expr::Number(Num::Int(0.into())));
                }
                if let Ok(n) = i128::from_str_radix(substr, match base {
                    Base::Binary => 2,
                    Base::Octal => 8,
                    Base::Decimal => 10,
                    Base::Hexadecimal => 16,
                }) {
                    return Ok(Expr::Number(Num::Int(n.into())));
                }
                return Error::reason("literal: Failed to parse Integer.").into();
            },
            TokenKind::Literal {
                kind: LiteralKind::Float { base, .. }
            } => {
                if base != Base::Decimal {
                    return Error::reason("literal: Can only parse floats as decimal.").into();
                }
                if let Ok(n) = substr.parse::<f64>() {
                    return Ok(Expr::Number(Num::Fp(n)));
                }
                return Error::reason("literal: Failed to parse float.").into();
            },
            TokenKind::Literal {
                kind: LiteralKind::Char { terminated }
            } => {
                if !terminated {
                    return Error::reason("literal: Unterminated char quote.").into();
                }

                let c = match unescape_char(substr) {
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
                let mut buf = Ok(Vec::with_capacity(len));
                unescape_unicode(&substr[1..(len-1)], Mode::Str, &mut |range, c| {
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

    fn parse_atom(&mut self, token: &Token) -> Result<Expr, Error> {
        let (start, len) = token.pos();
        let substr = self.source.substr(start, len).unwrap();
        match token.kind {
            TokenKind::Nil => Ok(Expr::Nil),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Literal { .. } => Ok(self.parse_literal(token)?),
            TokenKind::Ident => Ok(Expr::Ident(substr.to_string())),
            _ => Ok(Expr::Ident(substr.to_string())),
        }
    }

}

