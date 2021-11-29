use syntax::Span;

use crate::ast::Expr;
use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext};

use super::Code;

#[derive(Debug)]
pub struct IfStmt {
    pub span: Span,
    pub condition: Expr,
    pub code: Vec<Code>,
    pub else_code: Option<Vec<Code>>
}

impl IfStmt {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<IfStmt> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::IfKeyword));

        let expr = syntax::ex!(syntax::parse!(stream, Expr::parse));

        let code = if syntax::tk_iss!(stream, TokenKind::OpenCurly) {
            let mut code = Vec::new();
            loop {
                code.push(match syntax::parse!(stream, Code::parse, true) {
                    Some(x) => x,
                    None => break
                });
            }

            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

            code
        } else {
            vec![
                syntax::ex!(syntax::parse!(stream, Code::parse, true), stream.error("Expected statement"))
            ]
        };

        let else_code = if syntax::tk_iss!(stream, TokenKind::ElseKeyword) {
            if syntax::tk_iss!(stream, TokenKind::OpenCurly) {
                let mut code = Vec::new();
                loop {
                    code.push(match syntax::parse!(stream, Code::parse, true) {
                        Some(x) => x,
                        None => break
                    });
                }

                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

                Some(code)
            } else {
                Some(vec![
                    syntax::ex!(syntax::parse!(stream, Code::parse, true), stream.error("Expected statement"))
                ])
            }
        } else {
            None
        };

        syntax::MatchResult::Ok(IfStmt {
            span: syntax::Span::new(start, stream.tell_start()),
            condition: expr,
            code, else_code
        })
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        let mut cond = IrGenCodeTarget::new();
        self.condition.append_ir_value(ctx, &mut cond, Some(&ir::ValueType::Bool))?;

        let mut true_then = IrGenCodeTarget::new();
        for code in &self.code {
            code.append_ir(ctx, &mut true_then)?;
        }

        if let Some(else_code) = &self.else_code {
            let mut false_then = IrGenCodeTarget::new();
            for code in else_code {
                code.append_ir(ctx, &mut false_then)?;
            }

            target.push(ir::Ins::IfElse(
                true_then.take(),
                false_then.take(),
                cond.take()
            ));
        } else {
            target.push(ir::Ins::If(
                true_then.take(),
                cond.take()
            ));
        }

        Ok(())
    }
}