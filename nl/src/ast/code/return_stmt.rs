use syntax::Span;

use crate::ast::Expr;
use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext, IrGenErrorKind, value_type_to_string};

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
        // 1. Load the expression, if there is one
        if let Some(expr) = &self.expr {
            let result = expr.append_ir_value(ctx, target, None)?;

            // Not possible to have more than 1 in NL
            if let Some(return_type) = ctx.func().signature().returns().get(0) {
                if return_type != &result {
                    return Err(IrGenError::new(expr.span().clone(), IrGenErrorKind::IncorrectReturnType(value_type_to_string(&result), value_type_to_string(return_type))));
                }
            } else{
                return Err(IrGenError::new(expr.span().clone(), IrGenErrorKind::ReturnValueWhenVoid));
            }
        } else if ctx.func().signature().return_count() != 0 {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::NoReturnValue));
        }

        // 2. Ret
        target.push(ir::Ins::Ret);

        Ok(())
    }
}