use syntax::Span;

use crate::ast::Expr;
use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext};

#[derive(Debug)]
pub struct ReturnStmt {
    pub span: Span,
    pub expr: Option<Expr>
}

impl ReturnStmt {
    pub fn parse<'a>(stream: &mut TokenStream<'a>, terminated: bool) -> syntax::MatchResult<ReturnStmt> {
        let start = stream.tell_start();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::ReturnKeyword));

        let expr = syntax::parse!(stream, Expr::parse);

        if terminated {
            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));
        }

        syntax::MatchResult::Ok(ReturnStmt {
            span: syntax::Span::new(start, stream.tell_start()),
            expr
        })
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        if let Some(expr) = &self.expr {
            expr.append_ir_value(ctx, target, None)?;
        }

        target.push(ir::Ins::Ret);

        Ok(())
    }
}