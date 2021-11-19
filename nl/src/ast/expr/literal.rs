use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};

#[derive(Debug)]
pub struct NumberLitExpr {
	pub span: Span,
	pub number: String
}

#[derive(Debug)]
pub struct StringLitExpr {
	pub span: Span,
	pub value: String
}

impl StringLitExpr {
	pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let st = ir::StorableType::Compound(match ctx.ir_unit.find_type("String") {
			Some(x) => x,
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::StdLinkError))
		});

		Ok(ir::ValueType::Ref(Box::new(st)))
	}

	pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let st = ir::StorableType::Compound(match ctx.ir_unit.find_type("String") {
			Some(x) => x,
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::StdLinkError))
		});

		let raw_data = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
			None,
			ir::StorableType::SliceData(Box::new(ir::StorableType::Value(ir::ValueType::U8))),
			false,
			ir::Storable::SliceData(self.value.as_bytes().iter().map(|x| ir::Storable::Value(ir::Value::U8(*x))).collect())
		));

		let raw_slice = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
			None,
			ir::StorableType::Slice(Box::new(ir::StorableType::Value(ir::ValueType::U8))),
			false,
			ir::Storable::Slice(raw_data, 0, self.value.as_bytes().len())
		));

		let string_id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
			None, 
			st.clone(),
			false,
			ir::Storable::Compound(ir::Compound::Struct(ir::Struct::new(vec![
				ir::StructProp::new(ir::Storable::Value(ir::Value::Ref(raw_slice)))
			])))
		));

		let id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
			None, 
			ir::StorableType::Value(ir::ValueType::Ref(Box::new(st.clone()))),
			false,
			ir::Storable::Value(ir::Value::Ref(string_id))
		));

		target.push(ir::Ins::PushPath(
			ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Global(id, ir::StorableType::Value(ir::ValueType::Ref(Box::new(st.clone()))))),
			ir::ValueType::Ref(Box::new(st.clone()))
		));

		target.push(ir::Ins::Push(ir::ValueType::Ref(Box::new(st.clone()))));

		Ok(ir::ValueType::Ref(Box::new(st)))
	}
}

impl NumberLitExpr {
	pub fn resultant_type<'a>(&'a self, _ctx: &IrGenFunctionContext<'a>, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		Ok(match prefered {
			Some(vt) if vt.is_num() => vt.clone(),
			_ => ir::ValueType::I32
		})
	}

	pub fn append_ir<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		use std::str::FromStr;
		
		if let Ok(num) = i32::from_str(&self.number) {
			let vt = match prefered {
				Some(vt) if vt.is_num() => vt,
				_ => &ir::ValueType::I32
			};
			target.push(ir::Ins::PushLiteral(vt.clone(), num as u64));
			Ok(vt.clone())
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidInteger))
		}
	}
}