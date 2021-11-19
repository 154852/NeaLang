use syntax::Span;

use crate::{ast::Expr, irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext}};

#[derive(Debug)]
pub struct Assignment {
	pub span: Span,
	pub left: Expr,
	pub right: Expr
}

impl Assignment {
	pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		match &self.left {
			Expr::Name(name) => {
				if let Some(local_idx) = ctx.local_map.get(name.name.as_str()) {
					let local_idx = *local_idx;
					// Only valid local indices go in the local_map
					let local = ctx.func().get_local(local_idx).unwrap();

					let expected = match local.local_type() {
						ir::StorableType::Value(t) => t.clone(),
						_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
					};

					target.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(
						ir::ValuePathOrigin::Local(local_idx, ir::StorableType::Value(expected.clone())),
					), expected.clone()));

					let vt = self.right.append_ir_value(ctx, target, Some(&expected))?;
					if vt != expected {
						return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
					}

					target.push(ir::Ins::Pop(vt));
				} else {
					return Err(IrGenError::new(name.span.clone(), IrGenErrorKind::VariableDoesNotExist(name.name.clone())));
				}
			},
			_ => {				
				let (st, path) = self.left.construct_path_to(ctx, target, None)?;

				let st_v = match st {
					ir::StorableType::Value(x) => x.clone(),
					_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
				};

				target.push(ir::Ins::PushPath(
					path, st_v.clone()
				));

				let vt = self.right.append_ir_value(ctx, target, Some(&st_v))?;

				if st_v != vt {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
				}

				target.push(ir::Ins::Pop(vt));
			}
		}

		Ok(())
	}
}