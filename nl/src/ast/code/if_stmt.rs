use syntax::Span;

use crate::ast::Expr;
use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext, IrGenErrorKind};

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

        // Parse the condition
        let expr = syntax::ex!(syntax::parse!(stream, Expr::parse));

        // Parse the code - either it is 0+ lines surrounded by curly brackets or it is 1 line without
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

        // If there is an else, parse it as for the body
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
        // 1. Load the condition
        let mut cond = IrGenCodeTarget::new();
        if self.condition.append_ir_value(ctx, &mut cond, Some(&ir::ValueType::Bool))? != ir::ValueType::Bool {
            return Err(IrGenError::new(self.condition.span().clone(), IrGenErrorKind::NotABool));
        }

        // 2. Load the true then code
        let mut true_then = IrGenCodeTarget::new();
        for code in &self.code {
            code.append_ir(ctx, &mut true_then)?;
        }

        // 3. If there is else code, load it and emit an IfElse...
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
            // ...otherwise a simple If will suffice
            target.push(ir::Ins::If(
                true_then.take(),
                cond.take()
            ));
        }

        Ok(())
    }
}