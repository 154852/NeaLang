use syntax::Span;

use crate::ast::Expr;
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext, value_type_to_string};

#[derive(Debug)]
pub struct Assignment {
    pub span: Span,
    pub left: Expr,
    pub right: Expr
}

// NOTE: Parsing is handled in Code, since Assignment is tightly bound with expression parsing.
impl Assignment {
    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        match &self.left {
            Expr::Name(name) => {
                if let Some(local_idx) = ctx.local_map.get(name.name.as_str()) {
                    // Only valid local indices go in the local_map, so safe to unwrap
                    let local = ctx.func().get_local(*local_idx).unwrap();

                    // Check that the local type is a ValueType
                    let expected = match local.local_type() {
                        ir::StorableType::Value(t) => t.clone(),
                        _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
                    };

                    // 1. Push a path to the target, in this case a local
                    target.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(
                        ir::ValuePathOrigin::Local(*local_idx, ir::StorableType::Value(expected.clone())),
                    ), expected.clone()));

                    // 2. Push the value...
                    let vt = self.right.append_ir_value(ctx, target, Some(&expected))?;
                    if vt != expected { // ... and check it is the right type
                        return Err(IrGenError::new(self.span.clone(), 
                            IrGenErrorKind::AssignmentTypeMismatch(value_type_to_string(&vt), value_type_to_string(&expected))
                        ));
                    }

                    // 3. Pop
                    target.push(ir::Ins::Pop(vt));
                } else {
                    return Err(IrGenError::new(name.span.clone(), IrGenErrorKind::VariableDoesNotExist(name.name.clone())));
                }
            },
            _ => {
                // 1. Construct a path to the target
                let (st, path) = self.left.construct_path_to(ctx, target, None)?;

                // Check that the type that the path references is storable
                let st_v = match st {
                    ir::StorableType::Value(x) => x.clone(),
                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
                };

                // 2. Push the path
                target.push(ir::Ins::PushPath(
                    path, st_v.clone()
                ));

                // 3. Append the value...
                let vt = self.right.append_ir_value(ctx, target, Some(&st_v))?;
                if st_v != vt { // ... and check it is the right type
                    return Err(IrGenError::new(self.span.clone(),
                        IrGenErrorKind::AssignmentTypeMismatch(value_type_to_string(&vt), value_type_to_string(&st_v))
                    ));
                }

                // 4. Pop
                target.push(ir::Ins::Pop(vt));
            }
        }

        Ok(())
    }
}