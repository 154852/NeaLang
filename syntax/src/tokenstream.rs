use crate::SyntaxError;

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start: start, end: end }
    }
}

pub trait TokenKind {
    fn is_whitespace(&self) -> bool;
}

#[derive(Debug)]
pub struct Token<T: std::fmt::Debug + TokenKind> {
    kind: T,
    span: Span
}

impl<T: std::fmt::Debug + TokenKind> Token<T> {
    pub fn new(kind: T, span: Span) -> Token<T> {
        Token { kind: kind, span: span }
    }

    pub fn kind(&self) -> &T {
        &self.kind
    }
}

pub trait TokenMatcher<T: std::fmt::Debug + TokenKind> {
    // NOTE: string is already offseted, offset is just for the Span generation
    fn next<'a>(&mut self, string: &'a str, offset: usize) -> Option<(usize, Token<T>)>;
}

pub struct TokenStream<'a, T: std::fmt::Debug + TokenKind> {
    string: &'a str,
    offset: usize,
    token: Option<Token<T>>,
    token_length: Option<usize>,
    matcher: Box<dyn TokenMatcher<T>>,
    skip_whitespace: bool
}

impl<'a, T: std::fmt::Debug + TokenKind> TokenStream<'a, T> {
    pub fn new(string: &'a str, matcher: Box<dyn TokenMatcher<T>>) -> TokenStream<'a, T> {
        TokenStream {
            string,
            offset: 0,
            token: None,
            token_length: None,
            matcher: matcher,
            skip_whitespace: true
        }
    }

    pub fn step(&mut self) {
        loop {
            if self.finished() { return }

            if let Some((len, token)) = self.matcher.next(&self.string[self.offset..], self.offset) {
                self.offset += len;

                if !self.skip_whitespace || !token.kind.is_whitespace() {
                    self.token_length = Some(len);
                    self.token = Some(token);
                    return;
                } else {
                    self.token_length = None;
                    self.token = None;
                }
            } else {
                panic!("Could not tokenize - any TokenKind should have a fallback Char type");
            }
        }
    }

    pub fn token(&self) -> Option<&Token<T>> {
        self.token.as_ref()
    }

    pub fn finished(&self) -> bool {
        self.offset == self.string.len()
    }

    pub fn error<U: Into<String>>(&self, msg: U) -> SyntaxError {
        SyntaxError::new(match &self.token {
            None => Span::new(self.offset, self.offset),
            Some(t) => t.span.clone()
        }, msg.into())
    }

    pub fn tell(&self) -> usize {
        self.offset
    }

    pub fn tell_start(&self) -> usize {
        self.offset - self.token_length.unwrap_or(0)
    }
}

/// Create keyword parsers. Keywords must be alphanumeric.
#[macro_export]
macro_rules! keywords {
    ( $string:expr , $offset:expr , $( $word:expr => $name:expr ),* ) => {
        $(
            if $string.starts_with($word) && ($string.len() == $word.len() || !$string.as_bytes()[$word.len()].is_ascii_alphanumeric()) {
                return Some(($word.len(), Token::new($name, ::syntax::Span::new($offset, $offset + $word.len()))));
            }
        )*
    };
}

/// Create a generic identifier parser, parsing /[a-zA-Z_]][a-zA-Z0-9_]*/
#[macro_export]
macro_rules! ident {
    ( $string:expr , $offset:expr , $( $i:ident )::* ) => {
        if $string.starts_with('_') || $string.as_bytes()[0].is_ascii_alphabetic() {
            let mut len = 1;
            while $string.as_bytes().get(len).map_or(false, |x| x.is_ascii_alphanumeric() || *x as char == '_') {
                len += 1;
            }
            return Some((len, Token::new($( $i )::* ($string[0..len].to_string()), ::syntax::Span::new($offset, $offset + len))));
        }
    };
}

/// Create a generic whitespace parser, parsing /\s+/
#[macro_export]
macro_rules! whitespace {
    ( $string:expr , $offset:expr , $( $i:ident )::* ) => {
        if $string.as_bytes()[0].is_ascii_whitespace() {
            let mut len = 1;
            while $string.as_bytes().get(len).map_or(false, |x| x.is_ascii_whitespace()) {
                len += 1;
            }
            return Some((len, Token::new($( $i )::*, ::syntax::Span::new($offset, $offset + len))));
        }
    };
}

/// Create a generic number parser, parsing /[0-9]+/
#[macro_export]
macro_rules! number {
    ( $string:expr , $offset:expr , $( $i:ident )::* ) => {
        if $string.as_bytes()[0].is_ascii_digit() {
            let mut len = 1;
            while $string.as_bytes().get(len).map_or(false, |x| x.is_ascii_digit()) {
                len += 1;
            }
            return Some((len, Token::new($( $i )::* ($string[0..len].to_string()), ::syntax::Span::new($offset, $offset + len))));
        }
    };
}

/// Match a single character, either alphabetic or not
#[macro_export]
macro_rules! exact {
    ( $string:expr , $offset:expr , $( $chr:expr => $( $i:ident )::* ),* ) => {
        $(
            if $string.starts_with($chr) {
                return Some((1, Token::new($( $i )::*, ::syntax::Span::new($offset, $offset + 1))));
            }
        )*
    };
}

/// Match a continuous string, either alphabetic or not
#[macro_export]
macro_rules! exact_long {
    ( $string:expr , $offset:expr , $( $str:expr => $( $i:ident )::* ),* ) => {
        $(
            if $string.starts_with($str) {
                return Some(($str.len(), Token::new($( $i )::*, ::syntax::Span::new($offset, $offset + $str.len()))));
            }
        )*
    };
}

/// Create a generic fallback character parser, it cannot fail.
#[macro_export]
macro_rules! char {
    ( $string:expr , $offset:expr , $( $i:ident )::* ) => {
        return Some((1, Token::new($( $i )::* ($string.as_bytes()[0] as char), ::syntax::Span::new($offset, $offset + 1))));
    };
}