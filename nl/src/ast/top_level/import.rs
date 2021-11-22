use syntax::Span;

use crate::lexer::{TokenKind, TokenStream};

#[derive(Debug)]
pub struct ImportStmt {
    pub span: Span,
    pub path: Vec<String>
}

impl ImportStmt {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ImportStmt> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::ImportKeyword));

        let mut path = Vec::new();
        loop {
            path.push(syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected identifier")).to_owned());
            stream.step();

            if !syntax::tk_iss!(stream, TokenKind::Dot) { break }
        }

        syntax::MatchResult::Ok(ImportStmt {
            span: syntax::Span::new(start, stream.tell_start()),
            path
        })
    }
}