use std::rc::Rc;

use crate::error::Error;
use crate::expr::Expr;
use crate::lex::{Base, LiteralKind, Token, TokenKind};
use crate::str_ext::Substr;
use crate::unescape::{unescape_char, unescape_unicode, Mode};
use number::Number;

enum Prec {
    None = 0,
    Assign,
    Equality,
    Compare,
    AddSub,
    MulDiv,
    Power,
    Call,
}

impl Prec {
    fn lbp(&self) -> u8 {
        match self {
            Prec::None => 0,
            Prec::Assign => 1,
            Prec::Equality => 2,
            Prec::Compare => 3,
            Prec::AddSub => 4,
            Prec::MulDiv => 5,
            Prec::Power => 6,
            Prec::Call => 10,
        }
    }
}

pub struct Parser<'src> {
    source: &'src str,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Parser<'src> {
        Parser {
            source,
        }
    }

    pub fn parse<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        self.parse_expr(tokens, Prec::None)
    }

    fn parse_expr<'tk>(&mut self, tokens: &'tk [Token], min_prec: Prec) -> Result<(Expr, &'tk [Token]), Error> {
        if tokens.is_empty() {
            return Error::reason("empty expression").into();
        }

        let (mut left, mut rest) = self.parse_prefix(tokens)?;

        while let Some(op) = rest.first() {
            let op_prec = self.precedence(&op.kind);
            if op_prec.lbp() <= min_prec.lbp() {
                break;
            }
            let (right, new_rest) = self.parse_expr(&rest[1..], op_prec)?;
            left = self.desugar_infix(op, left, right);
            rest = new_rest;
        }

        while let Some(next) = rest.first() {
            if self.is_start_of_expr(next) {
                let (arg, new_rest) = self.parse_expr(rest, Prec::Call)?;
                left = Expr::List(vec![left, arg]);
                rest = new_rest;
            } else {
                break;
            }
        }

        Ok((left, rest))
    }

    fn is_start_of_expr(&self, tok: &Token) -> bool {
        use TokenKind::*;
        matches!(
            tok.kind,
            Ident | Literal { .. } | OpenParen | OpenBrace | Fn | If | Let
        )
    }

    fn precedence(&self, kind: &TokenKind) -> Prec {
        use TokenKind::*;
        use Prec::*;
        match kind {
            EqEq => Equality,
            Equals => Assign,
            Lt | LtEq | Gt | GtEq | NotEq => Compare,
            Plus | Minus => AddSub,
            Mul | Div => MulDiv,
            TokenKind::Power => Prec::Power,
            _ => Prec::None,
        }
    }

    fn parse_prefix<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (tok, rest) = tokens.split_first()
            .ok_or_else(|| Error::reason("parse_prefix: unexpected end of input"))?;

        use TokenKind::*;
        match tok.kind {
            Let => self.parse_let(rest),
            Fn => self.parse_fn(rest),
            If => self.parse_if(rest),
            Do => self.parse_do(rest),
            OpenParen => self.parse_grouped(rest),
            OpenBrace => self.parse_block(rest),
            _ => Ok((self.parse_atom(tok)?, rest)),
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

    fn parse_literal(&self, token: &Token) -> Result<Expr, Error> {
        let (start, len) = token.pos();
        let substr = self.source.substr(start, len).unwrap();
        match token.kind {
            TokenKind::Literal {
                kind: LiteralKind::Int { base, empty_int }
            } => {
                if empty_int {
                    return Ok(Expr::Number(Number::Discrete(0.into())));
                }
                if let Ok(n) = i128::from_str_radix(substr, match base {
                    Base::Binary => 2,
                    Base::Octal => 8,
                    Base::Decimal => 10,
                    Base::Hexadecimal => 16,
                }) {
                    return Ok(Expr::Number(Number::Discrete(n.into())));
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
                    return Ok(Expr::Number(n.into()));
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

    fn parse_let<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (name_tok, rest) = tokens.split_first().ok_or_else(|| Error::reason("let: expected name"))?;
        let name = self.parse_atom(name_tok)?;
        let (eq, rest) = rest.split_first().ok_or_else(|| Error::reason("let: expected ="))?;
        if eq.kind != TokenKind::Equals { return Error::reason("let: expected =").into(); }
        let (value, rest) = self.parse_expr(rest, Prec::None)?;
        Ok((Expr::List(vec![Expr::Ident("let".to_string()), name, value]), rest))
    }

    fn parse_fn<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let mut params = vec![];
        let mut rest = tokens;

        while let Some(tok) = rest.first() {
            if tok.kind == TokenKind::OpenBrace {
                break;
            }
            let param = self.parse_atom(tok)?;
            params.push(param);
            rest = &rest[1..];
        }

        let (open, new_rest) = rest.split_first()
            .ok_or_else(|| Error::reason("fn: expected {"))?;

        if open.kind != TokenKind::OpenBrace {
            return Error::reason("fn: expected {").into();
        }

        rest = new_rest;

        let (body, new_rest) = self.parse_expr(rest, Prec::None)?;

        let (close, rest) = new_rest.split_first()
            .ok_or_else(|| Error::reason("fn: expected }"))?;
        if close.kind != TokenKind::CloseBrace {
            return Error::reason("fn: expected }").into();
        }

        let params_list = Expr::List(params);
        Ok((
            Expr::List(vec![
                Expr::Ident("fn".to_string()),
                params_list,
                body,
            ]),
            rest
        ))
    }

    fn parse_if<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (test, rest) = self.parse_expr(tokens, Prec::None)?;

        let (then_tok, rest) = rest.split_first()
            .ok_or_else(|| Error::reason("if: expected 'then'"))?;
        if then_tok.kind != TokenKind::Then {
            return Error::reason("if: expected 'then'").into();
        }

        let (then_branch, rest) = self.parse_expr(rest, Prec::None)?;

        let (else_branch, rest) = if let Some(else_tok) = rest.first() {
            if else_tok.kind == TokenKind::Else {
                let (_, r) = rest.split_first().unwrap();
                let (e, r) = self.parse_expr(r, Prec::None)?;
                (e, r)
            } else {
                (Expr::Nil, rest)
            }
        } else {
            (Expr::Nil, rest)
        };

        Ok((
            Expr::List(vec![
                Expr::Ident("if".to_string()),
                test,
                then_branch,
                else_branch,
            ]),
            rest
        ))
    }

    fn parse_do<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (open, rest) = tokens.split_first().ok_or_else(|| Error::reason("do: expected { or list"))?;
        if open.kind == TokenKind::OpenBrace {
            let (body, rest) = self.parse_expr(rest, Prec::None)?;
            let (close, rest) = rest.split_first().ok_or_else(|| Error::reason("do: expected }"))?;
            if close.kind != TokenKind::CloseBrace { return Error::reason("do: expected }").into(); }
            Ok((body, rest))
        } else {
            self.read_seq(tokens)
        }
    }

    fn parse_grouped<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (expr, rest) = self.parse_expr(tokens, Prec::None)?;
        let (close, rest) = rest.split_first().ok_or_else(|| Error::reason("expected )"))?;
        if close.kind != TokenKind::CloseParen { return Error::reason("expected )").into(); }
        Ok((expr, rest))
    }

    fn parse_block<'tk>(&mut self, tokens: &'tk [Token]) -> Result<(Expr, &'tk [Token]), Error> {
        let (expr, rest) = self.parse_expr(tokens, Prec::None)?;
        let (close, rest) = rest.split_first().ok_or_else(|| Error::reason("expected }"))?;
        if close.kind != TokenKind::CloseBrace { return Error::reason("expected }").into(); }
        Ok((expr, rest))
    }

    fn desugar_infix(&self, op: &Token, left: Expr, right: Expr) -> Expr {
        let name = match op.kind {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Mul => "*",
            TokenKind::Div => "/",
            TokenKind::Power => "^",
            TokenKind::EqEq => "=",
            TokenKind::GtEq => ">=",
            TokenKind::Gt => ">",
            TokenKind::LtEq => "<=",
            TokenKind::Lt => "<",
            TokenKind::NotEq => "!=",
            TokenKind::Not => "!",
            _ => return Expr::List(vec![Expr::Ident("unknown_op".into()), left, right]),
        };
        Expr::List(vec![Expr::Ident(name.to_string()), left, right])
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

