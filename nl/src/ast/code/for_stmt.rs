use syntax::Span;

use crate::{ast::Expr, irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext}, lexer::{TokenKind, TokenStream}};

use super::Code;

#[derive(Debug)]
pub struct ForStmt {
	pub span: Span,
	pub init: Option<Box<Code>>,
	pub condition: Option<Expr>,
	pub inc: Option<Box<Code>>,
	pub code: Vec<Code>,
}

impl ForStmt {
    fn parse_ici<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<(Option<Box<Code>>, Option<Expr>, Option<Box<Code>>)> {
        let mut init = None;
        let mut condition = None;
        let mut inc = None;

        if !syntax::tk_iss!(stream, TokenKind::OpenCurly) {
            if !syntax::tk_iss!(stream, TokenKind::Semi) {
                init = Some(Box::new(syntax::ex!(syntax::parse!(stream, Code::parse, false), stream.error("Expected initial statement"))));

                if syntax::tk_iss!(stream, TokenKind::OpenCurly) {
                    condition = match *init.take().unwrap() {
                        Code::ExprStmt(expr) => Some(expr),
                        _ => return syntax::MatchResult::Err(stream.error("Cannot use statement as condition"))
                    };
                    return syntax::MatchResult::Ok((init, condition, inc));
                }

                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));
            }

            condition = Some(syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected condition")));

            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));

            inc = Some(Box::new(syntax::ex!(syntax::parse!(stream, Code::parse, false), stream.error("Expected increment"))));

            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenCurly), stream.error("Expected '{'"));
        }

        syntax::MatchResult::Ok((init, condition, inc))
    }

    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ForStmt> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::ForKeyword));

        let (init, condition, inc) = syntax::ex!(syntax::parse!(stream, ForStmt::parse_ici));

        let mut code = Vec::new();
        loop {
            code.push(match syntax::parse!(stream, Code::parse, true) {
                Some(x) => x,
                None => break
            });
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

        syntax::MatchResult::Ok(ForStmt {
            span: syntax::Span::new(start, stream.tell_start()),
            init,
            condition,
            inc,
            code,
        })
    }

	pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		if let Some(init) = &self.init {
			init.append_ir(ctx, target)?;
		}

		let mut body = IrGenCodeTarget::new();
		for code in &self.code {
			code.append_ir(ctx, &mut body)?;
		}

		let mut inc_body = IrGenCodeTarget::new();
		if let Some(inc) = &self.inc {
			inc.append_ir(ctx, &mut inc_body)?;
		}

		let mut condition_body = IrGenCodeTarget::new();
		if let Some(condition) = &self.condition {
			condition.append_ir_value(ctx, &mut condition_body, Some(&ir::ValueType::Bool))?;
		}

		target.push(ir::Ins::Loop(
			body.take(),
			condition_body.take(),
			inc_body.take(),
		));

		Ok(())
	}
}