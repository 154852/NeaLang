#[derive(Debug)]
pub enum TokenKind {
    FuncKeyword, ReturnKeyword, VarKeyword, IfKeyword, ElseKeyword, ForKeyword, ExternKeyword, StructKeyword,
    AsKeyword, NewKeyword, ImportKeyword,
    Ident(String),
    Char(char),
    Number(String),
    StringLit(String),
    OpenCurly, CloseCurly, OpenParen, CloseParen, OpenBracket, CloseBracket,
    Colon, Semi, Dot, Comma, Add, Mul, Div, Sub, Eq,
    DblEq, NotEq, Lt, Gt, Le, Ge,
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
        syntax::exact_long!(string, offset, 
            "==" => TokenKind::DblEq,
            "!=" => TokenKind::NotEq,
            "<=" => TokenKind::Le,
            ">=" => TokenKind::Ge
        );

        syntax::exact!(string, offset, 
            '{' => TokenKind::OpenCurly,
            '}' => TokenKind::CloseCurly,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semi,
            '+' => TokenKind::Add,
            '*' => TokenKind::Mul,
            '/' => TokenKind::Div,
            '-' => TokenKind::Sub,
			'=' => TokenKind::Eq,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt
        );

        syntax::keywords!(string, offset,
            "func" => TokenKind::FuncKeyword,
            "return" => TokenKind::ReturnKeyword,
			"var" => TokenKind::VarKeyword,
            "if" => TokenKind::IfKeyword,
            "else" => TokenKind::ElseKeyword,
            "for" => TokenKind::ForKeyword,
            "extern" => TokenKind::ExternKeyword,
            "struct" => TokenKind::StructKeyword,
            "as" => TokenKind::AsKeyword,
            "new" => TokenKind::NewKeyword,
            "import" => TokenKind::ImportKeyword
        );

        syntax::ident!(string, offset, TokenKind::Ident);
        syntax::whitespace!(string, offset, TokenKind::Whitespace);
        syntax::number!(string, offset, TokenKind::Number);
        syntax::cstring!(string, offset, TokenKind::StringLit);
        
        syntax::char!(string, offset, TokenKind::Char);
    }
}