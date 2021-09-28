use syntax;
use crate::ast;
use crate::lexer::*;
use syntax::{Parseable, MatchResult};

impl syntax::Parseable<TokenKind> for ast::Code {
	type Output = ast::Code;
	
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Code> {
        while syntax::tk_iss!(stream, TokenKind::Semi) {}
        
        let ok = match stream.token().map(|x| x.kind()) {
            // Safe to unwrap here as the only way these can fail is if the initial keyword is not what is expected, which it is - because we wouldn't go into that parser otherwise
            Some(TokenKind::ReturnKeyword) => MatchResult::Ok(ast::Code::ReturnStmt(syntax::parse!(stream, ast::ReturnStmt::parse).unwrap())),
			Some(TokenKind::VarKeyword) => MatchResult::Ok(ast::Code::VarDeclaration(syntax::parse!(stream, ast::VarDeclaration::parse).unwrap())),
            
            _ => return MatchResult::Fail
        };

        while syntax::tk_iss!(stream, TokenKind::Semi) {}

        ok
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

        let mut expr = match stream.token().map(|x| x.kind()) {
            Some(TokenKind::OpenParen) => {
                stream.step();
                let expr = Box::new(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression inside of parenthesis")));
                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
                
                ast::Expr::Closed(ast::ClosedExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    expr
                })
            },
            Some(TokenKind::Number(s)) => {
                let number = s.to_string();
                stream.step();
                ast::Expr::NumberLit(ast::NumberLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    number
                })
            },
            Some(TokenKind::Ident(s)) => {
                let name = s.to_string();
                stream.step();
                ast::Expr::Name(ast::NameExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    name
                })
            },
            _ => return MatchResult::Fail
        };

        loop {
            match stream.token().map(|x| x.kind()) {
                Some(TokenKind::OpenParen) => {
                    stream.step();
                    
                    let mut args = Vec::new();
                    loop {
                        args.push(match syntax::parse!(stream, ast::Expr::parse) {
                            Some(x) => x,
                            None => break
                        });
            
                        if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
                    }
    
                    syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));

                    expr = ast::Expr::Call(ast::CallExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
                        object: Box::new(expr),
                        args
                    });
                },
                _ => break
            }
        }

        syntax::MatchResult::Ok(expr)
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

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));

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

        let mut var_type = None;
        if syntax::tk_iss!(stream, TokenKind::Colon) {
            var_type = Some(syntax::ex!(syntax::parse!(stream, ast::TypeExpr::parse), stream.error("Expected type")));
        }

		let mut expr = None;
        if syntax::tk_iss!(stream, TokenKind::Eq) {
			expr = Some(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression")));
		}

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));

        syntax::MatchResult::Ok(ast::VarDeclaration {
            span: syntax::Span::new(start, stream.tell_start()),
            name, expr, var_type
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::TypeExpr {
	type Output = ast::TypeExpr;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::TypeExpr> {
        let start = stream.tell_start();
        let mut path = Vec::new();
        loop {
            path.push(syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected identifier")).to_owned());
            stream.step();

            if !syntax::tk_iss!(stream, TokenKind::Dot) { break }
        }

        syntax::MatchResult::Ok(ast::TypeExpr {
            span: syntax::Span::new(start, stream.tell_start()),
            path
        })
    }
}

impl syntax::Parseable<TokenKind> for ast::FunctionParam {
	type Output = ast::FunctionParam;

    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::FunctionParam> {
        let start = stream.tell_start();
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Colon), stream.error("Expected ':'"));

        let param_type = syntax::ex!(syntax::parse!(stream, ast::TypeExpr::parse), stream.error("Expected type"));

        syntax::MatchResult::Ok(ast::FunctionParam {
            span: syntax::Span::new(start, stream.tell_start()),
            name,
			param_type
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

            if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));

        let mut returns = Vec::new();
        if syntax::tk_iss!(stream, TokenKind::Colon) {
            if syntax::tk_iss!(stream, TokenKind::OpenParen) {
                loop {
                    returns.push(match syntax::parse!(stream, ast::TypeExpr::parse) {
                        Some(x) => x,
                        None => break
                    });
        
                    if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
                }

                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
            } else {
                returns.push(syntax::ex!(syntax::parse!(stream, ast::TypeExpr::parse), stream.error("Expected return type")));
            }
        }
        
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
			return_types: returns,
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