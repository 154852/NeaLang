#[derive(Debug)]
pub enum TokenKind {
    FuncKeyword, ReturnKeyword, VarKeyword,
    Ident(String),
    Char(char),
    Number(String),
    OpenCurly, CloseCurly, OpenParen, CloseParen,
    Semi, Comma, Add, Mul, Div, Sub, Eq,
    Whitespace
}

impl syntax::TokenKind for TokenKind {
    fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Whitespace)
    }
}

pub type Token = syntax::Token<TokenKind>;
pub type TokenStream<'a> = syntax::TokenStream<'a, TokenKind>;

pub struct Matcher;
impl syntax::TokenMatcher<TokenKind> for Matcher {
    fn next<'a>(&mut self, string: &'a str, offset: usize) -> Option<(usize, syntax::Token<TokenKind>)> {
        syntax::exact!(string, offset, 
            '{' => TokenKind::OpenCurly,
            '}' => TokenKind::CloseCurly,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            '+' => TokenKind::Add,
            '*' => TokenKind::Mul,
            '/' => TokenKind::Div,
            '-' => TokenKind::Sub,
			'=' => TokenKind::Eq
        );

        syntax::keywords!(string, offset,
            "func" => TokenKind::FuncKeyword,
            "return" => TokenKind::ReturnKeyword,
			"var" => TokenKind::VarKeyword
        );

        syntax::ident!(string, offset, TokenKind::Ident);
        syntax::whitespace!(string, offset, TokenKind::Whitespace);
        syntax::number!(string, offset, TokenKind::Number);
        
        syntax::char!(string, offset, TokenKind::Char);
    }
}