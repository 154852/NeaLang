use syntax;
use crate::ast;
use crate::lexer::*;
use syntax::{Parseable, MatchResult};

impl syntax::Parseable<TokenKind> for ast::Code {
	type Output = ast::Code;
	
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Code> {
        match stream.token().map(|x| x.kind()) {
            // Safe to unwrap here as the only way these can fail is if the initial keyword is not what is expected, which it is - because we wouldn't go into that parser otherwise
            Some(TokenKind::FuncKeyword) => MatchResult::Ok(ast::Code::Function(syntax::parse!(stream, ast::Function::parse).unwrap())),
            Some(TokenKind::ReturnKeyword) => MatchResult::Ok(ast::Code::ReturnStmt(syntax::parse!(stream, ast::ReturnStmt::parse).unwrap())),
			Some(TokenKind::VarKeyword) => MatchResult::Ok(ast::Code::VarDeclaration(syntax::parse!(stream, ast::VarDeclaration::parse).unwrap())),
            
            _ => MatchResult::Fail
        }
    }
}

impl ast::Expr {
    fn parse_primary<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Expr> {
        match stream.token().map(|x| x.kind()) {
            Some(TokenKind::OpenParen) => {
                stream.step();
                let expr = Box::new(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression inside of parenthesis")));
                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
                
                MatchResult::Ok(ast::Expr::Closed(ast::ClosedExpr { expr }))
            },
            Some(TokenKind::Number(s)) => {
                let number = s.to_string();
                stream.step();
                MatchResult::Ok(ast::Expr::NumberLit(ast::NumberLitExpr { number }))
            },
            Some(TokenKind::Ident(s)) => {
                let name = s.to_string();
                stream.step();
                MatchResult::Ok(ast::Expr::Name(ast::NameExpr { name }))
            },
            _ => MatchResult::Fail
        }
    }
}

impl syntax::Parseable<TokenKind> for ast::Expr {
	type Output = ast::Expr;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, ast::Expr::parse_primary));

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
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::ReturnKeyword));

        let expr = syntax::parse!(stream, ast::Expr::parse);

        while syntax::tk_iss!(stream, TokenKind::Semi) {}

        syntax::MatchResult::Ok(ast::ReturnStmt {
            expr
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::VarDeclaration {
	type Output = ast::VarDeclaration;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::VarDeclaration> {
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::VarKeyword));

		let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

		let mut expr = None;
        if syntax::tk_iss!(stream, TokenKind::Eq) {
			expr = Some(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression")));
		}

        syntax::MatchResult::Ok(ast::VarDeclaration {
            name, expr
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::FunctionParam {
	type Output = ast::FunctionParam;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::FunctionParam> {
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::MatchResult::Ok(ast::FunctionParam {
            name,
			param_type: ast::TypeExpr {
				path: vec!["i32".to_string()]
			}
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::Function {
	type Output = ast::Function;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Function> {
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
            name, params, code,
			return_types: Vec::new(),
			annotations: Vec::new()
        })
    }
}