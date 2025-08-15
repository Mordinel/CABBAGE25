
use std::collections::VecDeque;
use std::str::Chars;
use crate::str_ext::*;

pub fn lex(file_name: &str, input: &str) -> Vec<Token> {
    let mut cursor = Cursor::new(file_name, input);
    let mut vec = vec![];
    loop {
        let token = cursor.advance_token();
        match token.kind {
            TokenKind::Eof => break,
            TokenKind::BlockComment { terminated } if terminated => continue,
            TokenKind::LineComment => continue,
            TokenKind::WhiteSpace => continue,
            TokenKind::Newline => continue,
            _ => (),
        }
        vec.push(token);
    }
    vec
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    offset: u32,
    len: u16,
}

impl Token {
    pub fn new(kind: TokenKind, offset: u32, len: u16) -> Token {
        Token {
            kind,
            offset,
            len,
        }
    }

    /// returns the offset into the file and the length of the token
    pub fn pos(&self) -> (usize, usize) {
        (self.offset as usize, self.len as usize)
    }
}

/// Given any string slice, return the 'TokenKind' enum variant.
/// If the string slice does not contain a keyword, return `None`.
pub fn keyword(c: char, string: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match (c, string) {
        ('i', "f") => If,
        ('f', "n") => Fn,
        ('n', "il") => Nil,
        ('l', "et") => Let,
        ('d', "o") => Do,
        ('p', "rintln") => Println,
        ('p', "rint") => Print,
        ('t', "rue") => True,
        ('f', "alse") => False,
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // keywords
    If,
    Do,
    Fn,
    Let,
    Nil,
    True,
    False,
    Print,
    Println,

    // comment
    LineComment,

    /* comment */
    BlockComment { terminated: bool },

    /// #!/usr/bin/env ...
    Shebang,

    WhiteSpace,
    Newline,

    Ident,

    Literal { kind: LiteralKind, },

    // Punctuation
    /// ";;"
    DoubleSemi,
    /// ";"
    Semi,
    /// ":"
    Colon, 
    /// ","
    Comma,
    /// "."
    Dot,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,
    /// "@"
    At,
    /// "#"
    Pound,
    /// "?"
    Question,
    /// "$"
    Dollar,

    /// "="
    Equals,
    /// "=>"
    FatArrow,

    /// "!="
    NotEq,
    /// "!"
    Not,
    /// "~"
    BitNot,

    /// "<<"
    ShiftLeft,
    /// "<="
    LtEq,
    /// "<"
    Lt,

    /// ">>"
    ShiftRight,
    /// ">="
    GtEq,
    /// ">"
    Gt,

    /// "->"
    Arrow,

    /// "-"
    Minus,

    /// "and"
    And,
    /// "&"
    BitAnd,

    /// "or"
    Or,
    /// "|"
    BitOr,

    /// "+"
    Plus,
    /// "*"
    Mul,
    /// "/"
    Div,

    /// "xor"
    Xor,
    /// "^"
    Power,

    /// "%"
    Mod,

    /// Unknown token, not expected by the lexer.
    Unknown,

    /// End of input
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiteralKind {
    /// `12_u8`, `0o100`, `0b120i99`, `1f32`.
    Int { base: Base, empty_int: bool },
    /// `12.34f32`, `1e3`, but not `1f32`.
    Float { base: Base, empty_exponent: bool },
    /// `'a'`, `'\\'`, `'''`, `';`
    Char { terminated: bool },
    /// `"abc"`, `"abc`
    Str { terminated: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    /// Literal starts with "0b"
    Binary = 2,
    /// Literal starts with "0o"
    Octal = 8,
    /// Literal doesn't contain a prefix.
    Decimal = 10,
    /// Literal starts with "0x"
    Hexadecimal = 16,
}

/// Peekable iterator over a character sequence.
///
/// Next characters can be peeked via `first` method, 
/// and position can be shifted forward via `bump` method.
pub struct Cursor<'src> {
    chars_consumed: usize,
    len_remaining: usize,
    errors: Errors<'src>,
    chars: Chars<'src>,
    source: &'src str,
}

pub(crate) const EOF_CHAR: char = '\0';

use TokenKind::*;
use LiteralKind::*;

impl Cursor<'_> {
    pub fn advance_token(&mut self) -> Token {
        let first_char = match self.bump() {
            Some(c) => c,
            None => {
                let (start, _) = self.token_pos();
                return Token::new(TokenKind::Eof, start as u32, 0)
            },
        };
        let token_kind = match first_char {
            '/' => match self.first() {
                '*' => self.block_comment(),
                '/' => self.line_comment(),
                _ => Div,
            },

            '#' if self.first() == '!' => self.shebang(),
            '#' =>TokenKind::Pound,

            c if is_whitespace(c) => self.whitespace(),
            c if is_newline(c) => self.newline(),

            // Must be before ident check or else will be skipped due to `is_id_start()`
            'a' if self.matches(['n', 'd']) => And,
            'm' if self.matches(['o', 'd']) => Mod,
            'n' if self.matches(['o', 't']) => Not,
            'o' if self.matches(['r'])      => Or,
            'x' if self.matches(['o', 'r']) => Xor,

            c if is_id_start(c) => self.ident_or_unknown_prefix(c),

            c @ '0'..='9' => {
                let kind = self.number(c);
                self.eat_literal_suffix();
                TokenKind::Literal { kind }
            },


            '~' => BitNot,
            '=' if self.first() == '>' => FatArrow,
            '=' => Equals,
            '!' => self.not(),
            '<' => self.lt(),
            '>' => self.gt(),
            '-' if self.first() == '>' => Arrow,
            '-' => Minus,
            '&' => BitAnd,
            '|' => BitOr,
            '+' => Plus,
            '*' => Mul,
            '^' => Power,
            '%' => Mod,
            ';' => self.semi(),
            ':' => Colon,

            '?' => Question,
            ',' => Comma,
            '.' => Dot,
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '@' => At,
            '$' => Dollar,

            '\'' => {
                let terminated = self.single_quoted_string();
                if terminated {
                    self.eat_literal_suffix();
                }
                let kind = Char { terminated };
                Literal { kind }
            },
            '"' => {
                let terminated = self.double_quoted_string();
                if terminated {
                    self.eat_literal_suffix();
                }
                let kind = Str { terminated };
                Literal { kind }
            },

            _ => Unknown,
        };

        let (start, _) = self.token_pos();
        let offset = self.pos_within_token();
        let res = Token::new(token_kind, start as u32, offset as u16);
        self.reset_pos_within_token();
        res
    }

    fn line_comment(&mut self) -> TokenKind {
        self.bump();
        self.eat_while(|c| c != '\n');
        LineComment
    }

    fn block_comment(&mut self) -> TokenKind {
        self.bump();

        let mut depth = 1usize;
        while let Some(c) = self.bump() {
            match c {
                '/' if self.first() == '*' => {
                    self.bump();
                    depth += 1;
                },
                '*' if self.first() == '/' => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }

        BlockComment { terminated: depth == 0 }
    }

    fn ident_or_unknown_prefix(&mut self, chr: char) -> TokenKind {
        // start is already eaten, eat the rest of the identifier.
        let keyword = self.eat_while_and_yield_keyword(chr, |c| is_id_continue(*c));
        if let Some(k) = keyword {
            return k;
        }
        Ident
    }

    fn whitespace(&mut self) -> TokenKind {
        self.eat_while(is_whitespace);
        WhiteSpace
    }

    fn newline(&mut self) -> TokenKind {
        self.eat_while(is_newline);
        Newline
    }

    fn number(&mut self, first_digit: char) -> LiteralKind {
        let mut base = Base::Decimal;
        if first_digit == '0' {
            // attempt to parse encoding base.
            match self.first() {
                'b' => {
                    base = Base::Binary;
                    self.bump();
                    if !self.eat_decimal_digits() {
                        return Int { base, empty_int: true };
                    }
                },
                'o' => {
                    base = Base::Octal;
                    self.bump();
                    if !self.eat_decimal_digits() {
                        return Int { base, empty_int: true }
                    }
                },
                'x' => {
                    base = Base::Hexadecimal;
                    self.bump();
                    if !self.eat_hexadecimal_digits() {
                        return Int { base, empty_int: true }
                    }
                },

                // not a base prefix, eat additional digits.
                '0'..='9' | '_' => {
                    self.eat_decimal_digits();
                },

                // also not a base prefix, nothing more to do
                '.' | 'e' | 'E' => {},

                // just a zero
                _ => return Int { base, empty_int: false }
            }
        } else {
            // no base prefix, just eat the digits
            self.eat_decimal_digits();
        };

        match self.first() {
            '.' if self.second() != '.' && !is_id_start(self.second()) => {
                // might have stuff after the . and if it does, it needs to start with a number.
                self.bump();
                let mut empty_exponent = false;
                if self.first().is_ascii_digit() {
                    self.eat_decimal_digits();
                    match self.first() {
                        'e' | 'E' => {
                            self.bump();
                            empty_exponent = !self.eat_float_exponent();
                        },
                        _ => (),
                    }
                }
                Float { base, empty_exponent }
            },
            'e' | 'E' => {
                self.bump();
                let empty_exponent = !self.eat_float_exponent();
                Float { base, empty_exponent }
            },
            _ => Int { base, empty_int: false },
        }
    }

    fn single_quoted_string(&mut self) -> bool {
        // check if it's a one-symbol literal
        if self.second() == '\'' && self.first() != '\\' {
            self.bump();
            self.bump();
            return true;
        }

        // literal has more than one symbol.

        // parse until either quotes are terminated or error is detected.
        loop {
            match self.first() {
                // quotes are terminated, finish parsing
                '\'' => {
                    self.bump();
                    return true;
                },
                // newline without following '\'' means an unclosed quote, stop parsing.
                '\n' if self.second() != '\'' => break,
                // end of file, stop parsing
                EOF_CHAR if self.is_eof() => break,
                // escaped slash is considered one char, bump twice.
                '\\' => {
                    self.bump();
                    self.bump();
                },
                // skip the char
                _ => {
                    self.bump();
                }
            }
        }
        // string was not terminated
        false
    }

    /// eats double-quited string and returns true if string is terminated.
    fn double_quoted_string(&mut self) -> bool {
        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    return true;
                },
                '\\' if self.first() == '\\' || self.first() == '"' => {
                    // bump again to skip escaped char
                    self.bump();
                },
                _ => (),
            }
        }
        // eof reached.
        false
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                },
                '0'..='9' => {
                    has_digits = true;
                    self.bump();
                },
                _ => break,
            }
        }
        has_digits
    }

    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                },
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }

    // eats the float exponent, returns true if at least one digit was met
    // and returns false otherwise.
    fn eat_float_exponent(&mut self) -> bool {
        if self.first() == '-' || self.first() == '+' {
            self.bump();
        }
        self.eat_decimal_digits()
    }

    // eats the suffix of the literal.
    // (`u8`, _f64) etc.
    fn eat_literal_suffix(&mut self) {
        self.eat_identifier();
    }

    // eats the identifier. Note: succeeds on `_`, which isn't a valid identifier.
    fn eat_identifier(&mut self) {
        if !is_id_start(self.first()) {
            return;
        }
        self.bump();
        self.eat_while(is_id_continue);
    }

    // eats a shebang.
    fn shebang(&mut self) -> TokenKind {
        self.eat_while(|c| c != '\n');
        Shebang
    }

    // eats a greaterthan, greaterthan or equal to, or shift right operator.
    fn gt(&mut self) -> TokenKind {
        match self.first() {
            '=' => { self.bump(); GtEq },
            '>' => { self.bump(); ShiftRight },
            _ => Gt,
        }
    }

    // eats a lessthan, lessthan or equal to, or shift left operator.
    fn lt(&mut self) -> TokenKind {
        match self.first() {
            '=' => { self.bump(); LtEq },
            '<' => { self.bump(); ShiftLeft },
            _ => Lt,
        }
    }

    // eats a boolean not or not equal operator.
    fn not(&mut self) -> TokenKind {
        match self.first() {
            '=' => { self.bump(); NotEq },
            _ => Not,
        }
    }

    fn semi(&mut self) -> TokenKind {
        match self.first() {
            ';' => { self.bump(); DoubleSemi },
            _ => Semi,
        }
    }
}

#[allow(dead_code)]
impl<'src> Cursor<'src> {
    pub fn new(file_name: &'src str, input: &'src str) -> Cursor<'src> {
        Cursor {
            chars_consumed: 0,
            len_remaining: input.len(),
            errors: Errors::new(file_name, input),
            chars: input.chars(),
            source: input,
        }
    }

    pub fn as_str(&self) -> &'src str {
        self.chars.as_str()
    }

    /// Peeks the next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    pub fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Peeks the second symbol from the input stream without consuming it.
    pub(crate) fn second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    /// Peeks the third symbol from the input stream without consuming it.
    pub fn third(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    /// If the next N chars match `chars`, consume them and return true.
    /// If at any point, a char does not match `chars`, return false without eating the chars.
    pub fn matches<const N: usize>(&mut self, chars: [char; N]) -> bool {
        let mut iter = self.chars.clone();
        for ch in chars {
            if Some(ch) != iter.next() {
                return false;
            }
        }
        for _ in chars {
            self.bump();
        }
        true
    }

    /// Checks if there is nothing more to consume.
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn token_pos(&self) -> (usize, usize) {
        let start = self.chars_consumed;
        let len = self.pos_within_token();
        (start, len)
    }

    /// Returns amount of already consumed symbols.
    pub(crate) fn pos_within_token(&self) -> usize {
        self.len_remaining - self.chars.as_str().len()
    }

    /// Resets the number of bytes consumed to 0.
    pub(crate) fn reset_pos_within_token(&mut self) {
        self.chars_consumed += self.pos_within_token();
        self.len_remaining = self.chars.as_str().len();
    }

    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        Some(c)
    }

    pub(crate) fn eat_while_and_yield_keyword(
        &mut self,
        chr: char,
        predicate: impl FnMut(&char) -> bool,
    ) -> Option<TokenKind> {
        let iter = self.chars.clone();
        let count = iter.take_while(predicate).count();
        let kword = self.chars.as_str().get(..count);
        for _ in 0..count {
            self.bump();
        }
        if let Some(kword) = kword {
            return keyword(chr, kword);
        }
        None
    }

    /// Eats symbols while predicate returns true or until the end of the file is reached.
    pub(crate) fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    offset: usize,
    len: usize,
}

impl Error {
    pub fn new(message: String, offset: usize, len: usize) -> Self {
        Self {
            message,
            offset,
            len,
        }
    }

    fn format(&self, file_name: &str, file_content: &str) -> String {
        let (line, col) = file_content.pos(self.offset);
        let substr = file_content.line(line);
        assert!(col <= substr.len());

        let (left, right) = substr.split_at(col);
        let (r_left, r_right) = right.split_at(self.len);
        let substr = format!("{}{}{}", left, r_left, r_right);

        let pad_n = line.to_string().len();
        let pad_l = vec![' '; pad_n].iter().collect::<String>();
        let pad_r = vec![' '; col].iter().collect::<String>();
        let uline = vec!['^'; self.len].iter().collect::<String>();

        format!(
            "error: {}\n{pad_l}--> {file_name}@{line}:{}\n{line} | {substr}\n{pad_l}   {pad_r}{uline}",
            self.message,
            col + 1,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Errors<'s> {
    had_error: bool,
    errors: VecDeque<Error>,
    pub(crate) source: &'s str,
    file_name: &'s str,
}

impl<'s> Errors<'s> {
    pub fn new(file_name: &'s str, source: &'s str) -> Self {
        Self {
            had_error: false,
            errors: VecDeque::new(),
            source,
            file_name,
        }
    }

    pub fn push(&mut self, message: &str, offset: usize, len: usize) {
        self.had_error = true;
        self.errors.push_back(Error::new(message.to_string(), offset, len));
    }

    pub fn pop(&mut self) -> Option<Error> {
        self.errors.pop_front()
    }

    pub fn report(&self, error: &Error) {
        eprintln!("{}", error.format(self.file_name, self.source))
    }

    pub fn report_all(&mut self) {
        while let Some(err) = self.pop() {
            self.report(&err);
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn reset_errors(&mut self) {
        self.had_error = false;
    }
}

impl Iterator for Errors<'_> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let err = self.errors.pop_front();
        if let Some(e) = err {
            return Some(e.format(self.file_name, self.source));
        }
        None
    }
}

/// returns true if `c` is considered a whitespace
pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'   // \t
                     // \n is significant
        | '\u{000b}' // \v
        | '\u{000c}' // form feed
        | '\u{000d}' // \r
        | '\u{0020}' // space
    )
}

/// returns true if `c` is a newline
#[inline]
pub fn is_newline(c: char) -> bool {
    c == '\n'
}

/// returns true if `c` is a valid first char of an identifier.
#[inline]
pub fn is_id_start(c: char) -> bool {
    (c == '_')
        || ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
}

/// returns true if `c` is valid as a non-first char of an ident
#[inline]
pub fn is_id_continue(c: char) -> bool {
    (c == '_')
        || ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || ('0' <= c && c <= '9')
}

#[inline]
#[allow(dead_code)]
pub fn is_ident(string: &str) -> bool {
    let mut chars = string.chars();
    if let Some(start) = chars.next() {
        is_id_start(start) && chars.all(is_id_continue)
    } else {
        false
    }
}
