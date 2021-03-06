use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext, value_type_to_string};
use crate::lexer::{TokenKind, TokenStream};
use crate::ast::{Expr, TypeExpr};

#[derive(Debug)]
pub struct VarDeclaration {
    pub span: Span,
    pub name: String,
    pub expr: Option<Expr>,
    pub var_type: Option<TypeExpr>
}

impl VarDeclaration {
    pub fn parse<'a>(stream: &mut TokenStream<'a>, terminated: bool) -> syntax::MatchResult<VarDeclaration> {
        let start = stream.tell_start();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::VarKeyword));

        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

        // Optional explicit type
        let mut var_type = None;
        if syntax::tk_iss!(stream, TokenKind::Colon) {
            var_type = Some(syntax::ex!(syntax::parse!(stream, TypeExpr::parse), stream.error("Expected type")));
        }

        // Optional value
        let mut expr = None;
        if syntax::tk_iss!(stream, TokenKind::Eq) {
            expr = Some(syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected expression")));
        }

        if terminated {
            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'"));
        }

        syntax::MatchResult::Ok(VarDeclaration {
            span: syntax::Span::new(start, stream.tell_start()),
            name, expr, var_type
        })
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        // 1. Load the explicit type if it's given
        let expected_type = if let Some(var_type) = &self.var_type {
            Some(var_type.to_ir_value_type(ctx.ir_unit)?)
        } else {
            None
        };

        // 2. Load actual type of the expr
        let expr_type = match &self.expr {
            Some(expr) => Some(expr.resultant_type(ctx, expected_type.as_ref())?),
            None => None
        };

        // 3. Either...
        let expr_type = if let Some(var_type) = expected_type {
            if let Some(expr_type) = expr_type {
                if var_type != expr_type {
                    // If expr_type is not None, then self.expr is not None, so safe to unwrap
                    return Err(IrGenError::new(self.expr.as_ref().unwrap().span().clone(), 
                        IrGenErrorKind::AssignmentTypeMismatch(value_type_to_string(&expr_type), value_type_to_string(&var_type))
                    ));
                }
                expr_type // ...the types are both defined, and they match, or...
            } else {
                var_type // ...only the explicit type is given, or...
            }
        } else if let Some(expr_type) = expr_type {
            expr_type // ...only the value is given, or...
        } else { // ...neither is given
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CannotInferType));
        };

        // Create a new local
        let idx = ctx.push_local(&self.name, ir::StorableType::Value(expr_type.clone()));

        // 4. If there is an expression...
        if let Some(expr) = &self.expr {
            // ...push a path to the local
            target.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(
                ir::ValuePathOrigin::Local(idx, ir::StorableType::Value(expr_type.clone())),
            ), expr_type.clone()));

            // ...load the value
            let v = expr.append_ir_value(ctx, target, Some(&expr_type))?;
            assert_eq!(&v, &expr_type);
            
            // ...push it
            target.push(ir::Ins::Pop(expr_type));
        }

        Ok(())
    }
}