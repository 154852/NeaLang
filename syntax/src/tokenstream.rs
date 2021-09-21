use crate::SyntaxError;

#[derive(Debug, Clone, Copy)]
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
	matcher: Box<dyn TokenMatcher<T>>,
	skip_whitespace: bool
}

impl<'a, T: std::fmt::Debug + TokenKind> TokenStream<'a, T> {
	pub fn new(string: &'a str, matcher: Box<dyn TokenMatcher<T>>) -> TokenStream<'a, T> {
		TokenStream {
			string,
			offset: 0,
			token: None,
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
					self.token = Some(token);
					return;
				} else {
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
			Some(t) => t.span
		}, msg.into())
	}
}

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

#[macro_export]
macro_rules! char {
	( $string:expr , $offset:expr , $( $i:ident )::* ) => {
		return Some((1, Token::new($( $i )::* ($string.as_bytes()[0] as char), ::syntax::Span::new($offset, $offset + 1))));
	};
}