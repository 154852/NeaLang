use syntax;
use crate::ast;
use crate::lexer::*;
use syntax::{Parseable, MatchResult};

impl syntax::Parseable<TokenKind> for ast::Code {
	type Output = ast::Code;
	
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Code> {
        match stream.token().map(|x| x.kind()) {
            // Safe to unwrap here as the only way these can fail is if the initial keyword is not what is expected, which it is - because we wouldn't go into that parser otherwise
            Some(TokenKind::ReturnKeyword) => MatchResult::Ok(ast::Code::ReturnStmt(syntax::parse!(stream, ast::ReturnStmt::parse).unwrap())),
			Some(TokenKind::VarKeyword) => MatchResult::Ok(ast::Code::VarDeclaration(syntax::parse!(stream, ast::VarDeclaration::parse).unwrap())),
            
            _ => MatchResult::Fail
        }
    }
}

impl syntax::Parseable<TokenKind> for ast::TopLevelNode {
	type Output = ast::TopLevelNode;
	
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::TopLevelNode> {
        match stream.token().map(|x| x.kind()) {
            Some(TokenKind::FuncKeyword) => MatchResult::Ok(ast::TopLevelNode::Function(syntax::parse!(stream, ast::Function::parse).unwrap())),
            
            _ => MatchResult::Fail
        }
    }
}

impl ast::Expr {
    fn parse_primary<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Expr> {
        let start = stream.tell_start();

        match stream.token().map(|x| x.kind()) {
            Some(TokenKind::OpenParen) => {
                stream.step();
                let expr = Box::new(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression inside of parenthesis")));
                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
                
                MatchResult::Ok(ast::Expr::Closed(ast::ClosedExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    expr
                }))
            },
            Some(TokenKind::Number(s)) => {
                let number = s.to_string();
                stream.step();
                MatchResult::Ok(ast::Expr::NumberLit(ast::NumberLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    number
                }))
            },
            Some(TokenKind::Ident(s)) => {
                let name = s.to_string();
                stream.step();
                MatchResult::Ok(ast::Expr::Name(ast::NameExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    name
                }))
            },
            _ => MatchResult::Fail
        }
    }
}

impl syntax::Parseable<TokenKind> for ast::Expr {
	type Output = ast::Expr;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, ast::Expr::parse_primary));
        let start = stream.tell_start();

        loop {
            match stream.token().map(|x| x.kind()) {
                Some(TokenKind::Add) | Some(TokenKind::Mul) | Some(TokenKind::Div) | Some(TokenKind::Sub) => {
                    let op = match stream.token().map(|x| x.kind()).unwrap() {
                        TokenKind::Add => ast::BinaryOp::Add,
                        TokenKind::Mul => ast::BinaryOp::Mul,
                        TokenKind::Sub => ast::BinaryOp::Sub,
                        TokenKind::Div => ast::BinaryOp::Div,
                        _ => unreachable!()
                    };
                    stream.step();

                    let right = syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected right hand side to expression"));

                    expr = ast::Expr::BinaryExpr(ast::BinaryExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
                        op,
                        left: Box::new(expr),
                        right: Box::new(right)
                    });
                },
                _ => break,
            }
        }

        syntax::MatchResult::Ok(expr)
    }
}

impl syntax::Parseable<TokenKind> for ast::ReturnStmt {
	type Output = ast::ReturnStmt;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::ReturnStmt> {
        let start = stream.tell_start();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::ReturnKeyword));

        let expr = syntax::parse!(stream, ast::Expr::parse);

        while syntax::tk_iss!(stream, TokenKind::Semi) {}

        syntax::MatchResult::Ok(ast::ReturnStmt {
            span: syntax::Span::new(start, stream.tell_start()),
            expr
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::VarDeclaration {
	type Output = ast::VarDeclaration;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::VarDeclaration> {
        let start = stream.tell_start();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::VarKeyword));

		let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

		let mut expr = None;
        if syntax::tk_iss!(stream, TokenKind::Eq) {
			expr = Some(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression")));
		}

        syntax::MatchResult::Ok(ast::VarDeclaration {
            span: syntax::Span::new(start, stream.tell_start()),
            name, expr
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::FunctionParam {
	type Output = ast::FunctionParam;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::FunctionParam> {
        let start = stream.tell_start();
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::MatchResult::Ok(ast::FunctionParam {
            span: syntax::Span::new(start, stream.tell_start()),
            name,
			param_type: ast::TypeExpr {
                span: syntax::Span::new(0, 0),
				path: vec!["i32".to_string()]
			}
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::Function {
	type Output = ast::Function;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Function> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::FuncKeyword));

        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenParen), stream.error("Expected '('"));

        let mut params = Vec::new();
        loop {
            params.push(match syntax::parse!(stream, ast::FunctionParam::parse) {
                Some(x) => x,
                None => break
            });

            while syntax::tk_iss!(stream, TokenKind::Comma) {}
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
        
        let end = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenCurly), stream.error("Expected '{'"));

        let mut code = Vec::new();
        loop {
            code.push(match syntax::parse!(stream, ast::Code::parse) {
                Some(x) => x,
                None => break
            });
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

        syntax::MatchResult::Ok(ast::Function {
            span: syntax::Span::new(start, end),
            name, params, code,
			return_types: Vec::new(),
			annotations: Vec::new()
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::TranslationUnit {
	type Output = ast::TranslationUnit;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::TranslationUnit> {
        let mut nodes = Vec::new();

        while !stream.finished() {
            nodes.push(syntax::ex!(syntax::parse!(stream, ast::TopLevelNode::parse), stream.error("Expected a function")));
        }

        syntax::MatchResult::Ok(ast::TranslationUnit {
            nodes
        })
    }
}