use syntax::Span;

use crate::ast::Expr;
use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext, IrGenErrorKind, value_type_to_string};

#[derive(Debug)]
pub struct DropStmt {
    pub span: Span,
    pub expr: Expr
}

impl DropStmt {
    pub fn parse<'a>(stream: &mut TokenStream<'a>, terminated: bool) -> syntax::MatchResult<DropStmt> {
        let start = stream.tell_start();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::DropKeyword));

		let expr = syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected RHS"));

        if terminated {
            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));
        }

        syntax::MatchResult::Ok(DropStmt {
            span: syntax::Span::new(start, stream.tell_start()),
            expr
        })
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        // 1. Load expr
        let vt = self.expr.append_ir_value(ctx, target, None)?;
        let st = match vt { // ...check that it is a heap value
            ir::ValueType::Ref(st) => st, // FIXME: String globals are also Refs, but are not heap values - what should they be?
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidDropType(value_type_to_string(&vt))))
        };

        // 2. Free it
        match st.as_ref() {
            ir::StorableType::Compound(_) | ir::StorableType::Value(_) => target.push(ir::Ins::Free(st.as_ref().clone())),
            ir::StorableType::Slice(slice_type) => target.push(ir::Ins::FreeSlice(slice_type.as_ref().clone())),
            ir::StorableType::SliceData(_) => unreachable!(),
        }

        Ok(())
    }
}