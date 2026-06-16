use std::{
    cmp::PartialEq,
    iter::{Iterator, Peekable},
    fmt::Display,
};

use crate::sym::Strings;

pub fn fatal(msg: &str, pos: Pos) -> ! {
    panic!("{} at {},{}", msg, pos.line, pos.col)
}

pub fn fatal_tok(msg: &str, tok: Token) -> ! {
    let Token { kind, pos } = tok;

    panic!("{}, found {} at {},{}", msg, kind, pos.line, pos.col)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Comma,                      // ,
    Semi,                       // ;
    Colon,                      // :
    Assign,                     // =
    Plus,                       // +
    Minus,                      // -
    Star,                       // *
    Slash,                      // /
    Incr,                       // ++
    Decr,                       // --
    Caret,                      // ^
    Not,                        // !
    Mod,                        // %
    And,                        // &&
    Or,                         // ||
    Eql,                        // ==
    Neq,                        // !=
    Lss,                        // <
    Leq,                        // <=
    Gtr,                        // >
    Geq,                        // >=

    LeftBrk,                    // [
    RightBrk,                   // ]
    LeftParen,                  // (
    RightParen,                 // )
    LeftAngl,                   // {
    RightAngl,                  // }

    // Types
    Array,                      // array
    Bool,                       // boolean
    Char,                       // char
    Integer,                    // integer
    String,                     // string
    Void,                       // void

    // Other Keywords
    Else,                       // else
    For,                        // for
    Function,                   // function
    If,                         // if
    Print,                      // print
    Return,                     // return

    Ident(usize),               // Ident names are stored into a separate structure.
                                // We just store an index here.

    // Literals
    True,                       // true
    False,                      // false
    IntLit(i64),
    CharLit(char),
    StringLit(usize),           // String literals are stored into a separate structure.
                                // We just store an index here.
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            TokenKind::Comma => ",".to_string(),
            TokenKind::Semi => ";".to_string(),
            TokenKind::Colon => ":".to_string(),
            TokenKind::Assign => "=".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Incr => "++".to_string(),
            TokenKind::Decr => "--".to_string(),
            TokenKind::Caret => "^".to_string(),
            TokenKind::Not => "!".to_string(),
            TokenKind::Mod => "%".to_string(),
            TokenKind::And => "&&".to_string(),
            TokenKind::Or => "||".to_string(),
            TokenKind::Eql => "==".to_string(),
            TokenKind::Neq => "!=".to_string(),
            TokenKind::Lss => "<".to_string(),
            TokenKind::Leq => "<=".to_string(),
            TokenKind::Gtr => ">".to_string(),
            TokenKind::Geq => ">=".to_string(),

            TokenKind::LeftBrk => "[".to_string(),
            TokenKind::RightBrk => "]".to_string(),
            TokenKind::LeftParen => "(".to_string(),
            TokenKind::RightParen => ")".to_string(),
            TokenKind::LeftAngl => "{".to_string(),
            TokenKind::RightAngl => "}".to_string(),

            // Types
            TokenKind::Array => "array".to_string(),
            TokenKind::Bool => "boolean".to_string(),
            TokenKind::Char => "char".to_string(),
            TokenKind::Integer => "integer".to_string(),
            TokenKind::String => "string".to_string(),
            TokenKind::Void => "void".to_string(),

            // Other Keywords
            TokenKind::Else => "else".to_string(),
            TokenKind::For => "for".to_string(),
            TokenKind::Function => "function".to_string(),
            TokenKind::If => "if".to_string(),
            TokenKind::Print => "print".to_string(),
            TokenKind::Return => "return".to_string(),

            TokenKind::Ident(val) => format!("Ident({val})"),
            TokenKind::True => "true".to_string(),
            TokenKind::False => "false".to_string(),
            TokenKind::IntLit(val) => val.to_string(),
            TokenKind::CharLit(c) => c.to_string(),
            TokenKind::StringLit(val) => format!("String({val})"),
        };

        write!(f, "{}", repr)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos { line: usize, col: usize }

impl Pos {
    pub(crate) fn at(line: usize, col: usize) -> Self {
        Pos { line, col }
    }
}

#[derive(Clone, Copy, Debug, Eq)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) pos: Pos,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Token {
    pub fn at(kind: TokenKind, pos: Pos) -> Self {
        Token { kind, pos }
    }

    pub fn is_type(&self) -> bool {
        matches!(self.kind, TokenKind::Array
                          | TokenKind::Bool
                          | TokenKind::Char
                          | TokenKind::Integer
                          | TokenKind::String)
    }

    pub fn is_function_type(&self) -> bool {
        self.kind == TokenKind::Void || self.is_type()
    }
}

// Useful mostly for testing because the position is not useful
impl From<TokenKind> for Token {
    fn from(value: TokenKind) -> Self {
        Token::at(value, Pos::at(0, 0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Char {
    c: char,
    pos: Pos,
}

impl Char {
    fn at(c: char, pos: Pos) -> Self {
        Char { c, pos }
    }
}

pub struct Scanner<I>
    where I: Iterator<Item=char>,
{
    source: Peekable<I>,
    line: usize,
    col: usize,
    last_char: Option<Char>,
    token_buffer: Vec<Token>,
    strings: Strings,
}

impl<I> Scanner<I>
    where I: Iterator<Item=char>,
{
    pub fn new(source: Peekable<I>) -> Self {
        Scanner {
            source,
            line: 1,
            col: 1,
            last_char: None,
            token_buffer: vec![],
            strings: Strings::new(),
        }
    }

    pub fn into_strings(self) -> Strings {
        self.strings
    }

    fn put_char(&mut self, c: Char) {
        self.last_char = Some(c)
    }

    pub fn put_token(&mut self, token: Token) {
        self.token_buffer.push(token);
    }

    pub fn consume_token(&mut self) -> Token {
        self.token_buffer.pop().expect("No token to consume")
    }

    pub fn discard_token(&mut self) {
        let _ = self.token_buffer.pop();
    }

    fn advance_char(&mut self, c: char) -> Char {
        let (col, line) = (self.col, self.line);

        if c == '\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }

        Char::at(c, Pos::at(line, col))
    }

    fn next_if(&mut self, test: impl FnOnce(&char) -> bool) -> Option<Char> {
        if let Some(c) = self.last_char.take() {
           Some(c)
        } else {
            self.source.next_if(test)
                .map(|c| self.advance_char(c))
        }
    }

    fn next(&mut self) -> Option<Char> {
        self.next_if(|_| true)
    }

    fn next_if_eq(&mut self, tc: char) -> Option<Char> {
        self.next_if(|&c| c == tc)
    }

    /// Ensures that there's a next token and that it is of some specific kind.
    /// If we're at EOF or the token doesn't match, this function will panic.
    /// Otherwise there is no effect besides consuming the accepted token.
    pub fn must_be_kind(&mut self, kind: TokenKind) {
        let Some(tk) = self.scan() else { panic!("Found EOF while expecting token '{kind}'") };

        if tk.kind != kind {
            let msg = format!("Expected '{kind}'");
            fatal_tok(&msg, tk)
        }
    }

    pub fn is_eof(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Tests if there is a next token and if it matches certain specific kind.
    /// If `discard` is `true`, and the next token matches, the next token
    /// is discarded. Otherwise it's kept back.
    ///
    /// The function returns `true` upon match, and `false` if there is no match
    /// or the token stream is at EOF.
    pub fn maybe_kind(&mut self, kind: TokenKind, discard: bool) -> bool {
        let Some(tk) = self.peek() else { return false };
        if tk.kind == kind {
            if discard { self.discard_token(); }
            true
        } else {
            false
        }
    }

    pub fn must_be(&mut self, c: char) {
        let Some(ch) = self.next() else { panic!("Found EOF while expecting char '{c}'") };

        if ch.c != c {
            let msg = format!("Expected '{c}' but found '{}' instead", ch.c);
            fatal(&msg, ch.pos)
        }
    }

    fn process_line_comment(&mut self) {
        while let Some(ch) = self.next() {
            if ch.c == '\n' {
                break;
            }
        }
    }

    fn process_multiline_comment(&mut self, pos: Pos) {
        while let Some(ch) = self.next() {
            if ch.c == '*' {
                let Some(ch) = self.next() else { fatal("Found EOF while processing comment starting", pos) };

                if ch.c == '/' {
                    break;
                }
            }
        }
    }

    fn skip(&mut self) -> Option<Char> {
        while let Some(ch) = self.next() {
            if !ch.c.is_whitespace() {
                return Some(ch)
            }
        }

        None
    }

    pub fn scan_char(&mut self) -> char {
        let Some(Char { c, pos }) = self.next() else { panic!("Found EOF while processing a character literal") };

        let ret = match c {
            '\'' => fatal("Empty character", pos),
            '\\' => if self.next_if_eq('n').is_some() {
                '\n'
            } else if self.next_if_eq('0').is_some() {
                '\0'
            } else if let Some(ch) = self.next() {
                ch.c
            } else {
                panic!("Found EOF while processing a character literal")
            },
            _ => c,
        };
        self.must_be('\'');

        ret
    }

    pub fn scan_string(&mut self, pos: Pos) -> String {
        let mut vec_chars = vec![];

        while let Some(Char { c, .. }) = self.next() {
            match c {
                '"' => return vec_chars.into_iter().collect(),
                '\\' => if self.next_if_eq('n').is_some() {
                    vec_chars.push('\n');
                } else if self.next_if_eq('0').is_some() {
                    vec_chars.push('\0');
                },
                c => vec_chars.push(c)
            }
        }

        fatal("Found EOF while recognizing string that started", pos)
    }

    pub fn scan_ident(&mut self, first: char) -> String {
        let mut vec_chars = vec![first];

        while let Some(ch) = self.next_if(|c| c.is_ascii_alphanumeric() || *c == '_') {
            vec_chars.push(ch.c);
        }

        vec_chars.into_iter().collect()
    }

    pub fn scan_number(&mut self, first: char, radix: u32) -> i64 {
        let mut res = first.to_digit(radix).unwrap() as i64;

        while let Some(ch) = self.next_if(|&c| c.is_ascii_digit()) {
            res = res * 10 + ch.c.to_digit(radix).unwrap() as i64;
        }

        res
    }

    fn scan_alternate_operators(&mut self, pos: Pos, follower: char, if_true: TokenKind, if_false: TokenKind) -> Option<Token> {
        let kind = if self.next_if_eq(follower).is_some() { if_true } else { if_false };
        Some(Token { kind, pos })
    }

    pub fn scan(&mut self) -> Option<Token> {
        if let Some(t) = self.token_buffer.pop() {
            Some(t)
        } else {
            if let Some(Char { c, pos }) = self.skip() {
                match c {
                    ',' => Some(Token { kind: TokenKind::Comma, pos }),
                    ';' => Some(Token { kind: TokenKind::Semi, pos }),
                    ':' => Some(Token { kind: TokenKind::Colon, pos }),
                    '<' => self.scan_alternate_operators(pos, '=', TokenKind::Leq, TokenKind::Lss),
                    '>' => self.scan_alternate_operators(pos, '=', TokenKind::Geq, TokenKind::Gtr),
                    '!' => self.scan_alternate_operators(pos, '=', TokenKind::Neq, TokenKind::Not),
                    '=' => self.scan_alternate_operators(pos, '=', TokenKind::Eql, TokenKind::Assign),
                    '^' => Some(Token { kind: TokenKind::Caret, pos }),
                    '+' => self.scan_alternate_operators(pos, '+', TokenKind::Incr, TokenKind::Plus),
                    '-' => self.scan_alternate_operators(pos, '-', TokenKind::Decr, TokenKind::Minus),
                    '*' => Some(Token { kind: TokenKind::Star, pos }),
                    '/' => {
                        let prelim = Some(Token { kind: TokenKind::Slash, pos });
                        match self.next() {
                            Some(Char { c: '/', .. }) => {
                                self.process_line_comment(); self.scan()
                            },
                            Some(Char { c: '*', .. }) => {
                                self.process_multiline_comment(pos); self.scan()
                            },
                            Some(c) => {
                                self.put_char(c);
                                prelim
                            },
                            _ => prelim,
                        }
                    },
                    '%' => Some(Token { kind: TokenKind::Mod, pos }),
                    '&' => if self.next_if_eq('&').is_some() {
                        Some(Token { kind: TokenKind::And, pos })
                    } else {
                        fatal("Expected '&&'", pos)
                    },
                    '|' => if self.next_if_eq('|').is_some() {
                        Some(Token { kind: TokenKind::Or, pos })
                    } else {
                        fatal("Expected '||'", pos)
                    },
                    '(' => Some(Token { kind: TokenKind::LeftParen, pos }),
                    ')' => Some(Token { kind: TokenKind::RightParen, pos }),
                    '[' => Some(Token { kind: TokenKind::LeftBrk, pos }),
                    ']' => Some(Token { kind: TokenKind::RightBrk, pos }),
                    '{' => Some(Token { kind: TokenKind::LeftAngl, pos }),
                    '}' => Some(Token { kind: TokenKind::RightAngl, pos }),
                    '\'' => Some(Token { kind: TokenKind::CharLit(self.scan_char()), pos }),
                    '"' => {
                        let st = self.scan_string(pos);
                        Some(Token { kind: TokenKind::StringLit(self.strings.add(st)), pos })
                    },
                    c if c.is_ascii_digit() => {
                        // I've added the radix so that later on I can scan octal/hexadecimal
                        // as well.
                        let number = self.scan_number(c, 10);
                        Some(Token { kind: TokenKind::IntLit(number), pos })
                    }
                    c if c.is_ascii_alphabetic() || c == '_' => {
                        // Scan an identifier or keyword
                        let ident = self.scan_ident(c);
                        if let Some(kind) = self.keyword(&ident) {
                            Some(Token { kind, pos })
                        } else {
                            Some(Token { kind: TokenKind::Ident(self.strings.add(ident)), pos })
                        }
                    },
                    c => {
                        let msg = format!("Unrecognized character '{c}'");
                        fatal(&msg, pos)
                    }
                }
            } else {
                None
            }
        }
    }

    pub fn peek(&mut self) -> Option<Token> {
        match self.scan() {
            Some(t) => { 
                self.put_token(t);
                Some(t)
            },
            None => None,
        }
    }

    fn keyword(&self, name: &str) -> Option<TokenKind> {
        match name {
            "array" => Some(TokenKind::Array),
            "boolean" => Some(TokenKind::Bool),
            "char" => Some(TokenKind::Char),
            "integer" => Some(TokenKind::Integer),
            "string" => Some(TokenKind::String),
            "void" => Some(TokenKind::Void),
            "else" => Some(TokenKind::Else),
            "for" => Some(TokenKind::For),
            "function" => Some(TokenKind::Function),
            "if" => Some(TokenKind::If),
            "print" => Some(TokenKind::Print),
            "return" => Some(TokenKind::Return),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            _ => None,
        }
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::IntoChars;
    use rstest::{rstest, fixture};

    type TestScanner = Scanner<IntoChars>;

    #[fixture]
    fn scanner(#[default("")] code: &str) -> TestScanner {
        let s = String::from(code);
        Scanner::new(s.into_chars().peekable())
    }

    #[test]
    fn test_token_at() {
        let t = Token::at(TokenKind::Semi, Pos::at(3, 7));
        assert_eq!(t.kind, TokenKind::Semi);
        assert_eq!(t.pos, Pos::at(3, 7));
    }

    #[test]
    fn test_put_char() {
        let mut scn= scanner("");
        assert!(scn.last_char.is_none());
        let ch = Char::at('a', Pos::at(1,1 ));
        scn.put_char(ch);
        assert_eq!(scn.last_char, Some(ch));
    }

    #[test]
    fn test_put_token() {
        let mut scn= scanner("");
        assert!(scn.token_buffer.pop().is_none());
        let t = Token::at(TokenKind::Semi, Pos::at(1, 1));
        scn.put_token(t);
        assert_eq!(scn.scan(), Some(t));
    }

    #[test]
    fn test_consume_token() {
        let mut scn = scanner("");
        let token = Token { kind: TokenKind::And, pos: Pos::at(0, 0) };
        scn.put_token(token);

        assert_eq!(scn.consume_token(), token);
    }

    #[test]
    #[should_panic]
    fn test_consume_empty_token_panics() {
        let mut scn = scanner("");
        scn.consume_token();
    }

    #[rstest]
    fn test_discard_token() {
        let mut scn = scanner("");
        let token = Token { kind: TokenKind::And, pos: Pos::at(0, 0) };
        scn.put_token(token);

        scn.discard_token(); // Discard the putback token
        scn.discard_token(); // No putback token, don't complain anyway
    }

    #[rstest]
    #[case("text", Some(Char::at('t', Pos::at(1, 1))), Some(Char::at('e', Pos::at(1, 2))), Some(Char::at('x', Pos::at(1, 3))))]
    #[case("if", Some(Char::at('i', Pos::at(1, 1))), Some(Char::at('f', Pos::at(1, 2))), None)]
    #[case("i", Some(Char::at('i', Pos::at(1, 1))), None, None)]
    #[case("", None, None, None)]
    fn test_next(#[case] input: &str, #[case] val1: Option<Char>, #[case] val2: Option<Char>, #[case] val3: Option<Char>) {
        let mut scn = scanner(input);

        assert_eq!(scn.next(), val1);
        assert_eq!(scn.next(), val2);
        assert_eq!(scn.next(), val3);
    }

    #[test]
    fn test_next_with_putback_token() {
        let mut scn = scanner("");
        let ch = Char::at('k', Pos::at(1, 1));

        scn.put_char(ch);
        assert_eq!(scn.next(), Some(ch));
    }

    #[rstest]
    #[case("if", 'i', Some(Char::at('i', Pos::at(1, 1))))]
    #[case("if", 'f', None)]
    fn test_next_if_eq(#[case] input: &str, #[case] ch: char, #[case] expected: Option<Char>) {
        assert_eq!(scanner(input).next_if_eq(ch), expected);
    }

    #[rstest]
    #[case("inp", 'i')]
    #[should_panic(expected = "Expected 'e' but found 'i'")]
    #[case("inp", 'e')]
    fn test_must_be(#[case] input: &str, #[case] expected: char) {
        scanner(input).must_be(expected);
    }

    #[rstest]
    #[case(TokenKind::Incr, TokenKind::Incr)]
    #[case(TokenKind::Comma, TokenKind::Comma)]
    #[case(TokenKind::Semi, TokenKind::Semi)]
    #[case(TokenKind::If, TokenKind::If)]
    fn test_unit_variant_eq(#[case] a: TokenKind, #[case] b: TokenKind) {
        assert_eq!(a, b);
    }

    #[rstest]
    #[case(TokenKind::Incr, TokenKind::Comma)]
    #[case(TokenKind::Semi, TokenKind::Colon)]
    #[case(TokenKind::If, TokenKind::IntLit(0))]
    fn test_unit_variant_ne(#[case] a: TokenKind, #[case] b: TokenKind) {
        assert_ne!(a, b);
    }

    #[rstest]
    fn test_token_kind_int_lit(#[values(42, 0, -1, i64::MAX, i64::MIN)] val: i64) {
        assert_eq!(TokenKind::IntLit(val), TokenKind::IntLit(val));
        assert_ne!(TokenKind::IntLit(val), TokenKind::IntLit(val.wrapping_add(1)));
    }

    #[test]
    fn test_token_clone() {
        let t = Token::at(TokenKind::If, Pos::at(1, 2));
        assert_eq!(t.clone(), t);
    }

    #[test]
    fn test_token_equality_does_not_depend_on_position() {
        let a = Token::at(TokenKind::Star, Pos::at(1, 1));
        let b = Token::at(TokenKind::Star, Pos::at(1, 2));
        assert_eq!(a, b);
    }

    #[test]
    fn test_debug_format() {
        let t = Token::at(TokenKind::Plus, Pos::at(0, 0));
        let d = format!("{:?}", t);
        assert!(!d.is_empty());
    }

    #[rstest]
    fn test_token_kind_ident(#[values(0, usize::MAX)] val: usize) {
        assert_eq!(TokenKind::Ident(val), TokenKind::Ident(val));
    }

    #[rstest]
    fn test_token_kind_string_lit(#[values(5, 0)] val: usize) {
        assert_eq!(TokenKind::StringLit(val), TokenKind::StringLit(val));
    }

    #[test]
    fn test_token_kind_ident_vs_string_lit() {
        assert_ne!(TokenKind::Ident(3), TokenKind::StringLit(3));
    }

    #[test]
    fn test_token_copy() {
        let a = Token::at(TokenKind::Assign, Pos::at(2, 4));
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_pos_zero() {
        let t = Token::at(TokenKind::IntLit(0), Pos::at(0, 0));
        assert_eq!(t.pos, Pos::at(0, 0));
    }

    #[rstest]
    #[case(",", TokenKind::Comma)]
    #[case(";", TokenKind::Semi)]
    #[case(":", TokenKind::Colon)]
    #[case("=", TokenKind::Assign)]
    #[case("==", TokenKind::Eql)]
    #[case("!", TokenKind::Not)]
    #[case("!=", TokenKind::Neq)]
    #[case("<", TokenKind::Lss)]
    #[case("<=", TokenKind::Leq)]
    #[case(">", TokenKind::Gtr)]
    #[case(">=", TokenKind::Geq)]
    #[case("^", TokenKind::Caret)]
    #[case("+", TokenKind::Plus)]
    #[case("-", TokenKind::Minus)]
    #[case("*", TokenKind::Star)]
    #[case("/", TokenKind::Slash)]
    #[case("/ 5", TokenKind::Slash)] // Slightly different case from the above - for coverage
    #[case("(", TokenKind::LeftParen)]
    #[case(")", TokenKind::RightParen)]
    #[case("[", TokenKind::LeftBrk)]
    #[case("]", TokenKind::RightBrk)]
    #[case("{", TokenKind::LeftAngl)]
    #[case("}", TokenKind::RightAngl)]
    #[should_panic(expected = "Unrecognized character")]
    #[case("á", TokenKind::Semi)]
    fn test_basic_tokenization(#[case] input: &str, #[case] kind: TokenKind) {
        assert_eq!(scanner(input).scan(), Some(Token::at(kind, Pos::at(1, 1))));
    }

    #[test]
    fn test_no_more_tokens() {
        assert!(scanner("").scan().is_none())
    }

    #[rstest]
    #[case("1", 1)]
    #[case("15", 15)]
    #[case("999999", 999999)]
    fn test_int_literal(#[case] input: &str, #[case] expected: i64) {
        assert_eq!(scanner(input).scan(), Some(Token::at(TokenKind::IntLit(expected), Pos::at(1, 1))))
    }

    #[rstest]
    #[case("++", TokenKind::Incr)]
    #[case("--", TokenKind::Decr)]
    #[case("%", TokenKind::Mod)]
    fn test_remaining_operators(#[case] input: &str, #[case] kind: TokenKind) {
        assert_eq!(scanner(input).scan(), Some(Token::at(kind, Pos::at(1, 1))));
    }

    #[rstest]
    #[case("&&", TokenKind::And)]
    #[case("||", TokenKind::Or)]
    fn test_double_operators(#[case] input: &str, #[case] kind: TokenKind) {
        assert_eq!(scanner(input).scan(), Some(Token::at(kind, Pos::at(1, 1))));
    }

    #[test]
    fn test_line_comment() {
        assert_eq!(scanner("// comment\n,").scan(), Some(Token::at(TokenKind::Comma, Pos::at(2, 1))));
    }

    #[test]
    fn test_block_comment() {
        assert_eq!(scanner("/* comment */\n,").scan(), Some(Token::at(TokenKind::Comma, Pos::at(1, 13))));
    }

    #[rstest]
    #[case("'a'", 'a')]
    #[case("'\\n'", '\n')]
    #[case("'\\0'", '\0')]
    #[case(r"'\a'", 'a')]
    #[should_panic(expected = "Empty character")]
    #[case("''", '\0')]
    #[should_panic(expected = "Found EOF while processing a character literal")]
    #[case("'", '\0')]
    fn test_char_literal(#[case] input: &str, #[case] expected: char) {
        assert_eq!(scanner(input).scan(), Some(Token::at(TokenKind::CharLit(expected), Pos::at(1, 1))));
    }

    #[rstest]
    #[case("\"hello\"", 0)]
    #[case("\"\"", 0)]
    #[case("\"a b c\"", 0)]
    #[should_panic(expected = "Found EOF while recognizing string")]
    #[case("\"foo bar", 0)]
    fn test_string_literal(#[case] input: &str, #[case] expected: usize) {
        assert_eq!(scanner(input).scan(), Some(Token::at(TokenKind::StringLit(expected), Pos::at(1, 1))));
    }

    #[test]
    fn test_string_interning() {
        let mut s = scanner("\"hello\" \"hello\"");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::StringLit(0), Pos::at(1, 1))));
        assert_eq!(s.scan(), Some(Token::at(TokenKind::StringLit(0), Pos::at(1, 9))));
    }

    #[test]
    fn test_string_escape_newline() {
        let mut s = scanner("\"a\\nb\"");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::StringLit(0), Pos::at(1, 1))));
    }

    #[test]
    fn test_string_escape_null() {
        let mut s = scanner("\"a\\0b\"");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::StringLit(0), Pos::at(1, 1))));
    }

    #[test]
    fn test_string_then_token() {
        let mut s = scanner("\"hello\",");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::StringLit(0), Pos::at(1, 1))));
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Comma, Pos::at(1, 8))));
    }

    #[rstest]
    #[case("array", TokenKind::Array)]
    #[case("boolean", TokenKind::Bool)]
    #[case("char", TokenKind::Char)]
    #[case("integer", TokenKind::Integer)]
    #[case("string", TokenKind::String)]
    #[case("void", TokenKind::Void)]
    #[case("else", TokenKind::Else)]
    #[case("for", TokenKind::For)]
    #[case("function", TokenKind::Function)]
    #[case("if", TokenKind::If)]
    #[case("print", TokenKind::Print)]
    #[case("return", TokenKind::Return)]
    #[case("true", TokenKind::True)]
    #[case("false", TokenKind::False)]
    fn test_keyword(#[case] input: &str, #[case] kind: TokenKind) {
        assert_eq!(scanner(input).scan(), Some(Token::at(kind, Pos::at(1, 1))));
    }

    #[test]
    fn test_not_a_keyword() {
        assert!(scanner("").keyword("foo").is_none())
    }

    #[rstest]
    #[case("foo", 0)]
    #[case("_var", 0)]
    #[case("x123", 0)]
    fn test_identifier(#[case] input: &str, #[case] expected: usize) {
        assert_eq!(scanner(input).scan(), Some(Token::at(TokenKind::Ident(expected), Pos::at(1, 1))));
    }

    #[test]
    fn test_ident_interning() {
        let mut s = scanner("foo foo");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Ident(0), Pos::at(1, 1))));
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Ident(0), Pos::at(1, 5))));
    }

    #[test]
    fn test_ident_then_token() {
        let mut s = scanner("foo;");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Ident(0), Pos::at(1, 1))));
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Semi, Pos::at(1, 4))));
    }

    #[test]
    fn test_ident_and_string_interning_shared() {
        let mut s = scanner("foo \"foo\"");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Ident(0), Pos::at(1, 1))));
        assert_eq!(s.scan(), Some(Token::at(TokenKind::StringLit(0), Pos::at(1, 5))));
    }

    #[test]
    fn test_ident_not_keyword() {
        assert_eq!(scanner("ifx").scan(), Some(Token::at(TokenKind::Ident(0), Pos::at(1, 1))));
    }

    #[test]
    fn test_whitespace_between_tokens() {
        let mut s = scanner("  ,  ;");
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Comma, Pos::at(1, 3))));
        assert_eq!(s.scan(), Some(Token::at(TokenKind::Semi, Pos::at(1, 6))));
    }

    #[test]
    #[should_panic(expected = "Expected '&&'")]
    fn test_single_ampersand_fails() {
        scanner("&").scan();
    }

    #[test]
    #[should_panic(expected = "Expected '||'")]
    fn test_single_pipe_fails() {
        scanner("|").scan();
    }

    #[rstest]
    #[case(TokenKind::Comma, false)]
    #[case(TokenKind::Integer, true)]
    #[case(TokenKind::Char, true)]
    #[case(TokenKind::String, true)]
    #[case(TokenKind::Array, true)]
    #[case(TokenKind::Void, true)]
    fn test_is_function_type(#[case] kind: TokenKind, #[case] is_ft: bool) {
        let tk = Token { kind, pos: Pos::at(0, 0) };

        assert_eq!(tk.is_function_type(), is_ft)
    }


    // Tests added for coverage
    #[rstest]
    #[case(TokenKind::Comma, ",")]
    #[case(TokenKind::Semi , ";")]
    #[case(TokenKind::Colon , ":")]
    #[case(TokenKind::Assign , "=")]
    #[case(TokenKind::Plus , "+")]
    #[case(TokenKind::Minus , "-")]
    #[case(TokenKind::Star , "*")]
    #[case(TokenKind::Slash , "/")]
    #[case(TokenKind::Incr , "++")]
    #[case(TokenKind::Decr , "--")]
    #[case(TokenKind::Caret , "^")]
    #[case(TokenKind::Not , "!")]
    #[case(TokenKind::Mod , "%")]
    #[case(TokenKind::And , "&&")]
    #[case(TokenKind::Or , "||")]
    #[case(TokenKind::Eql , "==")]
    #[case(TokenKind::Neq , "!=")]
    #[case(TokenKind::Lss , "<")]
    #[case(TokenKind::Leq , "<=")]
    #[case(TokenKind::Gtr , ">")]
    #[case(TokenKind::Geq , ">=")]

    #[case(TokenKind::LeftBrk , "[")]
    #[case(TokenKind::RightBrk , "]")]
    #[case(TokenKind::LeftParen , "(")]
    #[case(TokenKind::RightParen , ")")]
    #[case(TokenKind::LeftAngl , "{")]
    #[case(TokenKind::RightAngl , "}")]

    // Types
    #[case(TokenKind::Array , "array")]
    #[case(TokenKind::Bool , "boolean")]
    #[case(TokenKind::Char , "char")]
    #[case(TokenKind::Integer , "integer")]
    #[case(TokenKind::String , "string")]
    #[case(TokenKind::Void , "void")]

    // Other Keywords
    #[case(TokenKind::Else , "else")]
    #[case(TokenKind::For , "for")]
    #[case(TokenKind::Function , "function")]
    #[case(TokenKind::If , "if")]
    #[case(TokenKind::Print , "print")]
    #[case(TokenKind::Return , "return")]

    #[case(TokenKind::True , "true")]
    #[case(TokenKind::False , "false")]
    #[case(TokenKind::CharLit('a'), "a")]
    #[case(TokenKind::CharLit('Z'), "Z")]
    #[case(TokenKind::IntLit(5), "5")]
    #[case(TokenKind::IntLit(98452), "98452")]
    fn test_tokenkind_display(#[case] tk: TokenKind, #[case] expected: &str) {
        let dsp = format!("{}", tk);
        assert_eq!(dsp.as_str(), expected)
    }

    #[test]
    fn test_is_eof_true() {
        assert!(scanner("").is_eof());
    }

    #[test]
    fn test_is_eof_false() {
        let mut scn = scanner(",");
        assert!(!scn.is_eof());
    }

    #[test]
    fn test_is_eof_after_scan() {
        let mut scn = scanner(",");
        scn.scan();
        assert!(scn.is_eof());
    }

    #[test]
    #[should_panic(expected = "Found EOF while processing a character literal")]
    fn test_char_literal_eof_after_backslash() {
        scanner("'\\").scan();
    }
}
