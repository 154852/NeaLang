use crate::lexer::{TokenKind, TokenStream};
use crate::ast::Function;

use super::{ImportStmt, StructDeclaration};

#[derive(Debug)]
pub enum TopLevelNode {
	Function(Function),
	StructDeclaration(StructDeclaration),
	Import(ImportStmt)
}

impl TopLevelNode {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<TopLevelNode> {
        match stream.token_kind() {
            Some(TokenKind::FuncKeyword) => syntax::MatchResult::Ok(TopLevelNode::Function(syntax::parse!(stream, Function::parse).unwrap())),
            Some(TokenKind::StructKeyword) => syntax::MatchResult::Ok(TopLevelNode::StructDeclaration(syntax::parse!(stream, StructDeclaration::parse).unwrap())),
            Some(TokenKind::ImportKeyword) => syntax::MatchResult::Ok(TopLevelNode::Import(syntax::parse!(stream, ImportStmt::parse).unwrap())),
            
            _ => syntax::MatchResult::Fail
        }
    }
}