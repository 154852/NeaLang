use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext, storable_type_to_string, value_type_to_string};

use super::Expr;

#[derive(Debug)]
pub struct MemberAccessExpr {
	pub span: Span,
	pub object: Box<Expr>,
	pub prop: String
}

impl MemberAccessExpr {
	pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let object = self.object.resultant_type(ctx, None)?;

		match object {
    		ir::ValueType::Ref(r) =>
				match r.as_ref() {
					ir::StorableType::Compound(c) => {
						match c.content() {
							ir::TypeContent::Struct(s) => {
								let prop_idx = match s.props().iter().position(|x| x.name() == self.prop) {
									Some(x) => x,
									None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), c.name().to_string()))),
								};
								let prop = s.prop(prop_idx).unwrap();
								let t = match prop.prop_type() {
									ir::StorableType::Value(vt) => vt.clone(),
									_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
								};
								
								Ok(t)
							},
						}
					},
					ir::StorableType::Value(_) => Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
					ir::StorableType::Slice(_st) => {
						if self.prop != "length" {
							return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(&r))));
						}

						Ok(ir::ValueType::UPtr)
					},
					ir::StorableType::SliceData(_) => unreachable!(),
				},
			_ => unreachable!()
		}
	}

	pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let object = self.object.append_ir_value(ctx, target, None)?;

		match object {
    		ir::ValueType::Ref(r) =>
				match r.as_ref() {
					ir::StorableType::Compound(c) =>
						match c.content() {
							ir::TypeContent::Struct(s) => {
								let idx = match s.props().iter().position(|x| x.name() == self.prop) {
									Some(x) => x,
									None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), c.name().to_string()))),
								};
								let prop = s.prop(idx).unwrap();
								let t = match prop.prop_type() {
									ir::StorableType::Value(vt) => vt.clone(),
									_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
								};
								
								target.push(ir::Ins::PushPath(ir::ValuePath::new(
									ir::ValuePathOrigin::Deref(r.as_ref().clone()),
									vec![
										ir::ValuePathComponent::Property(idx, c.clone(), prop.prop_type().clone())
									]
								), t.clone()));
								target.push(ir::Ins::Push(t.clone()));
								Ok(t)
							},
						},
					ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
					ir::StorableType::Slice(_st) => {
						if self.prop != "length" {
							return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(&r))));
						}

						target.push(ir::Ins::PushPath(ir::ValuePath::new(
							ir::ValuePathOrigin::Deref(r.as_ref().clone()),
							vec![
								ir::ValuePathComponent::Length
							]
						), ir::ValueType::UPtr));
						target.push(ir::Ins::Push(ir::ValueType::UPtr));

						Ok(ir::ValueType::UPtr)
					},
					ir::StorableType::SliceData(_) => unreachable!(),
				},
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), value_type_to_string(&object))))
		}
	}

	pub fn construct_path_to<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<(ir::StorableType, ir::ValuePath), IrGenError> {
		let object = self.object.append_ir_value(ctx, target, None)?;

		match object {
    		ir::ValueType::Ref(r) => match r.as_ref() {
				ir::StorableType::Compound(c) =>
					match c.content() {
						ir::TypeContent::Struct(s) => {
							let idx = match s.props().iter().position(|x| x.name() == self.prop) {
								Some(x) => x,
								None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), c.name().to_string()))),
							};
							let prop = s.prop(idx).unwrap();

							Ok((
								prop.prop_type().clone(), ir::ValuePath::new(
									ir::ValuePathOrigin::Deref(r.as_ref().clone()),
									vec![
										ir::ValuePathComponent::Property(idx, c.clone(), prop.prop_type().clone())
									]
								)
							))
						},
					},
				ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
				ir::StorableType::Slice(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(&r)))),
				ir::StorableType::SliceData(_) => unreachable!(),
			},
			_ => unreachable!()
		}
	}
}