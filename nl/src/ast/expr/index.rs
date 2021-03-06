use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};

use super::Expr;

#[derive(Debug)]
pub struct IndexExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub arg: Box<Expr>
}

impl IndexExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        match self.object.resultant_type(ctx, None)? {
            ir::ValueType::Ref(st) => match st.as_ref() {
                ir::StorableType::Slice(t) => match t.as_ref() {
                    ir::StorableType::Value(v) => Ok(v.clone()),
                    _ => {
                        Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
                    }
                },
                _ => {
                    Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
                }
            },
            _ => Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
        }
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {        
        // 1. Load the index as a uptr
        if self.arg.append_ir_value(ctx, target, Some(&ir::ValueType::UPtr))? != ir::ValueType::UPtr {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexValue));
        }

        // 2. Convert it into an index
        target.push(ir::Ins::Index(match self.object.resultant_type(ctx, None)? {
            ir::ValueType::Ref(st) => match st.as_ref() {
                ir::StorableType::Slice(t) => t.as_ref().clone(),
                _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
        }));

        // 3. Load the slice reference
        let el = match self.object.append_ir_value(ctx, target, None)? {
            ir::ValueType::Ref(st) => match st.as_ref() {
                ir::StorableType::Slice(t) => t.clone(),
                _ => {
                    return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
                }
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
        };

        let vt = match el.as_ref() {
            ir::StorableType::Value(val) => val,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
        };

        // 4. Create a path to access it
        target.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Deref(ir::StorableType::Slice(el.clone())),
            vec![
                ir::ValuePathComponent::Slice(ir::StorableType::Value(vt.clone()))
            ]
        ), vt.clone()));

        // 5. Derefence it
        target.push(ir::Ins::Push(vt.clone()));

        Ok(vt.clone())
    }

    pub fn construct_path_to<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<(ir::StorableType, ir::ValuePath), IrGenError> {
        // 1. Load the index as a uptr
        if self.arg.append_ir_value(ctx, target, Some(&ir::ValueType::UPtr))? != ir::ValueType::UPtr {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexValue));
        }
        
        // 2. Convert it into an index
        target.push(ir::Ins::Index(match self.object.resultant_type(ctx, None)? {
            ir::ValueType::Ref(st) => match st.as_ref() {
                ir::StorableType::Slice(t) => t.as_ref().clone(),
                _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
        }));
        
        // 3. Load the object (a reference to a slice)
        let el = match self.object.append_ir_value(ctx, target, None)? {
            ir::ValueType::Ref(st) => match st.as_ref() {
                ir::StorableType::Slice(t) => t.clone(),
                _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
        };

        // Return the path, which derefs the slice ref and indexes into it
        Ok((el.as_ref().clone(), ir::ValuePath::new(
            ir::ValuePathOrigin::Deref(ir::StorableType::Slice(el.clone())),
            vec![
                ir::ValuePathComponent::Slice(el.as_ref().clone())
            ]
        )))
    }
}