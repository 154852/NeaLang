use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};

use super::Expr;

#[derive(Debug)]
pub struct BinaryExpr {
    pub span: Span,
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>
}

#[derive(Debug)]
pub enum BinaryOp {
    Add, Mul, Div, Sub,
    Eq, Ne, Lt, Le, Gt, Ge,
    BoolAnd, BoolOr
}

impl BinaryOp {
    pub fn is_num(&self) -> bool {
        match self {
            BinaryOp::Add | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Sub => true,
            _ => false
        }
    }
}

impl BinaryExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        self.left.resultant_type(ctx, if self.op.is_num() { prefered } else { None })
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let left = self.left.append_ir_value(ctx, target, if self.op.is_num() { prefered } else { None })?;
        let right = self.right.append_ir_value(ctx, target, if self.op.is_num() { prefered } else { Some(&left) })?;

        if left != right { return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::BinaryOpTypeMismatch)) }

        target.push(match self.op {
            BinaryOp::Add => ir::Ins::Add(left.clone()),
            BinaryOp::Mul => ir::Ins::Mul(left.clone()),
            BinaryOp::Div => ir::Ins::Div(left.clone()),
            BinaryOp::Sub => ir::Ins::Sub(left.clone()),
            
            BinaryOp::Eq => ir::Ins::Eq(left.clone()),
            BinaryOp::Ne => ir::Ins::Ne(left.clone()),
            
            BinaryOp::Lt => ir::Ins::Lt(left.clone()),
            BinaryOp::Le => ir::Ins::Le(left.clone()),
            BinaryOp::Gt => ir::Ins::Gt(left.clone()),
            BinaryOp::Ge => ir::Ins::Ge(left.clone()),

            BinaryOp::BoolAnd => ir::Ins::BoolAnd,
            BinaryOp::BoolOr => ir::Ins::BoolOr,
        });

        Ok(left)
    }
}