use syntax::Span;

use crate::{ast::TypeExpr, irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext}};

#[derive(Debug)]
pub struct NewExpr {
    pub span: Span,
    pub new_type: TypeExpr
}

impl NewExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let st = self.new_type.to_ir_storable_type(ctx.ir_unit)?;
        Ok(ir::ValueType::Ref(Box::new(st)))
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let st = self.new_type.to_ir_storable_type(ctx.ir_unit)?;
        match &st {
            ir::StorableType::Slice(slice_st) => {
                // 1. Push the length (could be calculated at runtime)
                // .last because we are not created an N dimensional array, we are only creating an array of references
                if let Some(Some(expr)) = self.new_type.slice_lengths.last() {
                    if expr.append_ir_value(ctx, target, Some(&ir::ValueType::UPtr))? != ir::ValueType::UPtr {
                        return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexValue));
                    }
                } else {
                    target.push(ir::Ins::PushLiteral(ir::ValueType::UPtr, 0));
                }

                // 2. Allocate the slice
                target.push(ir::Ins::NewSlice(slice_st.as_ref().clone()));
            },
            ir::StorableType::SliceData(_) => unreachable!(),
            _ => {
                // 1. Simply allocate the object - size is not controlled by the programmer
                target.push(ir::Ins::New(st.clone()));
            },
        }

        Ok(ir::ValueType::Ref(Box::new(st)))
    }
}