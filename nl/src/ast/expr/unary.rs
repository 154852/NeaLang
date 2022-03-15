use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext};

use super::Expr;

#[derive(Debug)]
pub struct UnaryExpr {
    pub span: Span,
    pub op: UnaryOp,
    pub right: Box<Expr>
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg
}

impl UnaryOp {
    pub fn is_num(&self) -> bool {
        match self {
            UnaryOp::Neg => true,
        }
    }
}

impl UnaryExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        self.right.resultant_type(ctx, if self.op.is_num() { preferred } else { None })
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        // 1. Load RHS
        let right = self.right.append_ir_value(ctx, target, if self.op.is_num() { preferred } else { None })?;

        // 2. Do the operation
        target.push(match self.op {
            UnaryOp::Neg => ir::Ins::Neg(right.clone()),
        });

        match self.op {
            UnaryOp::Neg => Ok(right),
        }
    }
}