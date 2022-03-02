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
    /// Parses either nothing (inifinite loop), a single condition, or an initialiser, a condition and an increment.
    fn parse_ici<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<(Option<Box<Code>>, Option<Expr>, Option<Box<Code>>)> {
        let mut init = None;
        let mut condition = None;
        let mut inc = None;

        // Is it not an infinite loop?
        if !syntax::tk_iss!(stream, TokenKind::OpenCurly) {
            // Do we not skip the initialiser?
            if !syntax::tk_iss!(stream, TokenKind::Semi) {
                init = Some(Box::new(syntax::ex!(syntax::parse!(stream, Code::parse, false), stream.error("Expected initial statement"))));

                // If the condition is followed by a {, then it was a condition, not an init - so require that it was an expression and use that
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

        // Code in a for loop cannot use the same pattern as if, without curly brackets - since it would make parsing the ici difficult
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
        // 1. Load the initialiser (outside the loop), if there is one
        if let Some(init) = &self.init {
            init.append_ir(ctx, target)?;
        }

        // 2. Load the body
        let mut body = IrGenCodeTarget::new();
        for code in &self.code {
            code.append_ir(ctx, &mut body)?;
        }

        // 3. Load the increment if there is one
        let mut inc_body = IrGenCodeTarget::new();
        if let Some(inc) = &self.inc {
            inc.append_ir(ctx, &mut inc_body)?;
        }

        // 4. Load the condition, if there isn't one - it's just 1 (true)
        let mut condition_body = IrGenCodeTarget::new();
        if let Some(condition) = &self.condition {
            condition.append_ir_value(ctx, &mut condition_body, Some(&ir::ValueType::Bool))?;
        } else {
            condition_body.push(ir::Ins::PushLiteral(ir::ValueType::Bool, 1));
        }

        // 5. Do the loop
        target.push(ir::Ins::Loop(
            body.take(),
            condition_body.take(),
            inc_body.take(),
        ));

        Ok(())
    }
}