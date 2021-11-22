use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};
use crate::ast::TypeExpr;

use super::Expr;

#[derive(Debug)]
pub struct AsExpr {
    pub span: Span,
    pub expr: Box<Expr>,
    pub new_type: TypeExpr
}

impl AsExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        match self.new_type.to_ir_storable_type(ctx.ir_unit)? {
            ir::StorableType::Value(v) => Ok(v),
            _ => Err(IrGenError::new(self.span.clone(), IrGenErrorKind::NonValueCast)),
        }
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let curr_type = self.expr.append_ir_value(ctx, target, None)?;
        let desired_type = match self.new_type.to_ir_storable_type(ctx.ir_unit)? {
            ir::StorableType::Value(v) => v,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::NonValueCast)),
        };

        if !curr_type.is_num() || !desired_type.is_num() {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::NonValueCast));
        }

        target.push(ir::Ins::Convert(curr_type, desired_type.clone()));

        Ok(desired_type)
    }
}