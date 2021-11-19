use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};

#[derive(Debug)]
pub struct NameExpr {
	pub span: Span,
	pub name: String
}

impl NameExpr {
	pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let st = ctx.func().get_local(*idx).unwrap().local_type();

			match st {
				ir::StorableType::Value(vt) => {
					Ok(vt.clone())
				},
				_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CompositeTypeOnStack)),
			}
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::VariableDoesNotExist(self.name.clone())))
		}
	}

	pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let idx = *idx;
			
			let st = ctx.func().get_local(idx).unwrap().local_type();

			match st {
				ir::StorableType::Value(vt) => {
					target.push(ir::Ins::PushPath(
						ir::ValuePath::new_origin_only(
							ir::ValuePathOrigin::Local(idx, st.clone()),
						),
						vt.clone()
					));
					target.push(ir::Ins::Push(vt.clone()));
					Ok(vt.clone())
				},
				_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CompositeTypeOnStack)),
			}
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::VariableDoesNotExist(self.name.clone())))
		}
	}

	pub fn construct_path_to<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, _target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<(ir::StorableType, ir::ValuePath), IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let idx = *idx;
			
			let st = ctx.func().get_local(idx).unwrap().local_type();

			Ok((st.clone(), ir::ValuePath::new_origin_only(
				ir::ValuePathOrigin::Local(idx, st.clone()),
			)))
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::VariableDoesNotExist(self.name.clone())))
		}
	}
}