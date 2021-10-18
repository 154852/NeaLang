use std::collections::HashMap;
use std::str::FromStr;

use crate::ast;
use ir;
use syntax::Span;

#[derive(Debug)]
pub enum IrGenErrorKind {
	UnknownType,
	VariableDoesNotExist,
	FunctionDoesNotExist,
	InvalidInteger,
	BinaryOpTypeMismatch,
	AssignmentTypeMismatch,
	CannotInferType,
	CallArgParamCountMismatch,
	CallArgTypeMismatch,
	CallNotOneReturnInExpr,
	InvalidLHS,
	InvalidRHS,
	CompositeTypeOnStack,
	PropDoesNotExist,
	IllegalIndexObject,
	IllegalIndexValue,
	NonValueCast,
	StdLinkError
}

pub struct IrGenError {
	span: Span,
	kind: IrGenErrorKind
}

impl IrGenError {
	pub fn new(span: Span, kind: IrGenErrorKind) -> IrGenError {
		IrGenError {
			span, kind
		}
	}

	pub fn start(&self) -> usize {
		self.span.start
	}

	pub fn end(&self) -> usize {
		self.span.end
	}

	pub fn message(&self) -> String {
		format!("{:?}", self.kind)
	}
}

impl ast::TypeExpr {
	fn to_ir_base_storable_type(&self, ir_unit: &ir::TranslationUnit) -> Result<ir::StorableType, IrGenError> {
		// There must be a first item, or else this shouldn't have parsed
		match self.path.get(0).unwrap().as_str() {
			"i32" => return Ok(ir::StorableType::Value(ir::ValueType::I32)),
			"u32" => return Ok(ir::StorableType::Value(ir::ValueType::U32)),
			"i64" => return Ok(ir::StorableType::Value(ir::ValueType::I64)),
			"u64" => return Ok(ir::StorableType::Value(ir::ValueType::U64)),
			"uptr" => return Ok(ir::StorableType::Value(ir::ValueType::UPtr)),
			"u8" => return Ok(ir::StorableType::Value(ir::ValueType::U8)),
			_ => {}
		}

		if let Some(ct) = ir_unit.find_type(&self.path.get(0).unwrap()) {
			return Ok(ir::StorableType::Compound(ct));
		}

		Err(IrGenError::new(self.span.clone(), IrGenErrorKind::UnknownType))
	}

	fn to_ir_storable_type(&self, ir_unit: &ir::TranslationUnit) -> Result<ir::StorableType, IrGenError> {
		let mut st = self.to_ir_base_storable_type(ir_unit)?;

		for _ in 0..self.slice_depth {
			st = ir::StorableType::Slice(Box::new(st));
		}

		Ok(st)
	}

	fn to_ir_value_type(&self, ir_unit: &ir::TranslationUnit) -> Result<ir::ValueType, IrGenError> {
		match self.to_ir_storable_type(ir_unit)? {
			ir::StorableType::Compound(ct) => Ok(ir::ValueType::Ref(Box::new(ir::StorableType::Compound(ct)))),
			ir::StorableType::Slice(st) => Ok(ir::ValueType::Ref(Box::new(ir::StorableType::Slice(st)))),
			ir::StorableType::Value(v) => Ok(v),
			ir::StorableType::SliceData(_) => unreachable!()
		}
	}
}

impl PartialEq for ast::TypeExpr {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl ast::TranslationUnit {
	fn func(&self, name: &str) -> Option<(usize, &ast::Function)> {
		let mut id = 0;
		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					if func.path.len() == 0 && func.name == name { return Some((id, func)); }
					id += 1;
				},
				_ => {}
			}
		}

		None
	}

	fn method(&self, struct_name: &str, name: &str) -> Option<(usize, &ast::Function)> {
		let mut id = 0;
		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					if func.is_method() && func.path[0] == struct_name && func.name == name { return Some((id, func)); }
					id += 1;
				},
				_ => {}
			}
		}

		None
	}

	pub fn to_ir(&self) -> Result<ir::TranslationUnit, IrGenError> {
		let mut unit = ir::TranslationUnit::new();

		for node in &self.nodes {
			match node {
				ast::TopLevelNode::StructDeclaration(decl) => {
					let ct = decl.to_ir(&unit, self)?;
					unit.add_type(ct);
				},
				_ => {}
			}
		}

		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					let func = func.to_ir_base(&unit, self)?;
					unit.add_function(func);
				},
				_ => {}
			}
		}

		let mut id = 0;
		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					if func.code.is_some() {
						func.append_ir(self, &mut unit, id)?;
					}
					id += 1;
				},
				_ => {}
			}
		}

		Ok(unit)
	}
}

struct IrGenFunctionContext<'a> {
	unit: &'a ast::TranslationUnit,
	ir_unit: &'a mut ir::TranslationUnit,
	function_idx: ir::FunctionIndex,

	local_map: HashMap<&'a str, ir::LocalIndex>
}

impl<'a> IrGenFunctionContext<'a> {
	fn func(&self) -> &ir::Function {
		self.ir_unit.get_function(self.function_idx)
	}

	fn func_mut(&mut self) -> &mut ir::Function {
		self.ir_unit.get_function_mut(self.function_idx)
	}

	fn push_local(&mut self, name: &'a str, st: ir::StorableType) -> ir::LocalIndex {
		let idx = self.func_mut().push_local(ir::Local::new(st));
		self.local_map.insert(name, idx);

		idx
	}
}

struct IrGenCodeTarget {
	ins: Vec<ir::Ins>
}

impl IrGenCodeTarget {
	fn new() -> IrGenCodeTarget {
		IrGenCodeTarget {
			ins: Vec::new()
		}
	}

	fn push(&mut self, ins: ir::Ins) {
		self.ins.push(ins);
	}

	fn take(self) -> Vec<ir::Ins> {
		self.ins
	}
}

impl ast::StructDeclaration {
	fn to_ir(&self, ir_unit: &ir::TranslationUnit, _unit: &ast::TranslationUnit) -> Result<ir::CompoundTypeRef, IrGenError> {
		let mut ir_struct = ir::StructContent::new();
		for field in &self.fields {
			ir_struct.push_prop(ir::StructProperty::new(
				&field.name,
				ir::StorableType::Value(field.field_type.to_ir_value_type(ir_unit)?)
			));
		}

		Ok(ir::CompoundType::new(&self.name, ir::TypeContent::Struct(ir_struct)))
	}
}

impl ast::Function {
	fn to_ir_base(&self, ir_unit: &ir::TranslationUnit, _unit: &ast::TranslationUnit) -> Result<ir::Function, IrGenError> {
		let mut params = Vec::with_capacity(self.params.len());
		for param in &self.params {
			params.push(param.param_type.to_ir_value_type(ir_unit)?);
		}

		let mut returns = Vec::with_capacity(self.return_types.len());
		for return_type in &self.return_types {
			returns.push(return_type.to_ir_value_type(ir_unit)?);
		}

		if self.path.len() > 0 {
			assert_eq!(self.path.len(), 1);
			let ctr = match ir_unit.find_type(&self.path[0]) {
				Some(x) => x,
				None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::UnknownType))
			};

			Ok(if self.code.is_some() {
				ir::Function::new_method(&self.name, ir::Signature::new(params, returns), ctr)
			} else {
				ir::Function::new_extern_method(&self.name, ir::Signature::new(params, returns), ctr)
			})
		} else {
			Ok(if self.code.is_some() {
				ir::Function::new(&self.name, ir::Signature::new(params, returns))
			} else {
				ir::Function::new_extern(&self.name, ir::Signature::new(params, returns))
			})
		}
	}

	fn append_ir(&self, unit: &ast::TranslationUnit, ir_unit: &mut ir::TranslationUnit, idx: ir::FunctionIndex) -> Result<(), IrGenError> {
		let mut ctx = IrGenFunctionContext {
			unit,
			ir_unit,
			function_idx: idx,
			local_map: HashMap::new()
		};

		let mut target = IrGenCodeTarget::new();

		// Put params into locals
		for param in self.params.iter().rev() {
			let vt = param.param_type.to_ir_value_type(ctx.ir_unit)?;
			let local = ctx.push_local(&param.name, ir::StorableType::Value(vt.clone()));
			target.push(ir::Ins::PopLocalValue(vt, local));
		}

		for code in self.code.as_ref().unwrap() {
			code.append_ir(&mut ctx, &mut target)?;
		}

		if ctx.func().signature().returns().len() == 0 && !matches!(ctx.func().code().last(), Some(ir::Ins::Ret)) {
			target.push(ir::Ins::Ret);
		}

		ctx.func_mut().code_mut().extend(target.take());
		
		Ok(())
	}
}

impl ast::Code {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		match self {
			ast::Code::ReturnStmt(stmt) => stmt.append_ir(ctx, target),
			ast::Code::VarDeclaration(vardecl) => vardecl.append_ir(ctx, target),
			ast::Code::ExprStmt(expr) => {
				let drop_count = match expr {
					ast::Expr::Call(call_expr) => call_expr.append_ir_out_expr(ctx, target)?,
					_ => {
						expr.append_ir_value(ctx, target, None)?;
						1
					}
				};

				for _ in 0..drop_count {
					target.push(ir::Ins::Drop);
				}

				Ok(())
			},
			ast::Code::Assignment(assignment) => assignment.append_ir(ctx, target),
			ast::Code::IfStmt(if_stmt) => if_stmt.append_ir(ctx, target),
			ast::Code::ForStmt(for_stmt) => for_stmt.append_ir(ctx, target)
		}
	}
}

impl ast::ForStmt {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		if let Some(init) = &self.init {
			init.append_ir(ctx, target)?;
		}

		let mut body = IrGenCodeTarget::new();
		for code in &self.code {
			code.append_ir(ctx, &mut body)?;
		}

		let mut inc_body = IrGenCodeTarget::new();
		if let Some(inc) = &self.inc {
			inc.append_ir(ctx, &mut inc_body)?;
		}

		let mut condition_body = IrGenCodeTarget::new();
		if let Some(condition) = &self.condition {
			condition.append_ir_value(ctx, &mut condition_body, Some(&ir::ValueType::Bool))?;
		}

		target.push(ir::Ins::Loop(
			body.take(),
			condition_body.take(),
			inc_body.take(),
		));

		Ok(())
	}
}

impl ast::IfStmt {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		self.condition.append_ir_value(ctx, target, Some(&ir::ValueType::Bool))?;

		let mut true_then = IrGenCodeTarget::new();
		for code in &self.code {
			code.append_ir(ctx, &mut true_then)?;
		}

		if let Some(else_code) = &self.else_code {
			let mut false_then = IrGenCodeTarget::new();
			for code in else_code {
				code.append_ir(ctx, &mut false_then)?;
			}

			target.push(ir::Ins::IfElse(
				true_then.take(),
				false_then.take()
			));
		} else {
			target.push(ir::Ins::If(
				true_then.take()
			));
		}

		Ok(())
	}
}

impl ast::Assignment {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		match &self.left {
			ast::Expr::Name(name) => {
				if let Some(local_idx) = ctx.local_map.get(name.name.as_str()) {
					let local_idx = *local_idx;
					let local = ctx.func().get_local(local_idx).unwrap();

					let expected = match local.local_type() {
						ir::StorableType::Value(t) => t.clone(),
						_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
					};

					let vt = self.right.append_ir_value(ctx, target, Some(&expected))?;
					if vt != expected {
						return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
					}

					target.push(ir::Ins::PopLocalValue(vt, local_idx));
				} else {
					todo!() // Global?
				}
			},
			_ => {
				let t = self.left.append_ir_ref(ctx, target, None)?;
				let vt = self.right.append_ir_value(ctx, target, match &t {
					ir::ValueType::Ref(x) => match x.as_ref() {
						ir::StorableType::Value(v) => Some(v.clone()),
						_ => None
					},
					_ => panic!("Expected reference")
				}.as_ref())?;

				if t != ir::ValueType::Ref(Box::new(ir::StorableType::Value(vt.clone()))) {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
				}

				target.push(ir::Ins::PopRef(vt));
			}
		}

		Ok(())
	}
}

impl ast::ReturnStmt {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		if let Some(expr) = &self.expr {
			expr.append_ir_value(ctx, target, None)?;
		}

		target.push(ir::Ins::Ret);

		Ok(())
	}
}

impl ast::VarDeclaration {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		let expected_type = if let Some(var_type) = &self.var_type {
			let var_type = var_type.to_ir_storable_type(ctx.ir_unit)?;

			match var_type {
				ir::StorableType::Value(v) => Some(v),
				ir::StorableType::Compound(ct) => {
					ctx.push_local(&self.name, ir::StorableType::Value(ir::ValueType::Ref(Box::new(ir::StorableType::Compound(ct)))));
					return Ok(());
				},
				ir::StorableType::Slice(st) => {
					ctx.push_local(&self.name, ir::StorableType::Value(ir::ValueType::Ref(Box::new(ir::StorableType::Slice(st)))));
					return Ok(());
				}
				ir::StorableType::SliceData(_) => unreachable!()
			}
		} else {
			None
		};

		let mut expr_type = if let Some(expr) = &self.expr {
			Some(expr.append_ir_value(ctx, target, expected_type.as_ref())?)
		} else {
			None
		};

		if let Some(var_type) = expected_type {
			if let Some(expr_type) = &expr_type {
				if &var_type != expr_type {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
				}
			} else {
				expr_type = Some(var_type);
			}
		} else if expr_type.is_none() {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CannotInferType));
		}

		let idx = ctx.push_local(&self.name, ir::StorableType::Value(expr_type.as_ref().unwrap().clone()));

		if self.expr.is_some() {
			target.push(ir::Ins::PopLocalValue(expr_type.unwrap(), idx));
		}

		Ok(())
	}
}

impl ast::Expr {
	fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		match self {
			ast::Expr::BinaryExpr(binary_expr) => binary_expr.append_ir(ctx, target, prefered),
			ast::Expr::Name(name_expr) => name_expr.append_ir_value(ctx, target, prefered),
			ast::Expr::Closed(closed_expr) => closed_expr.expr.append_ir_value(ctx, target, prefered),
			ast::Expr::NumberLit(number_lit) => number_lit.append_ir(ctx, target, prefered),
			ast::Expr::Call(call_expr) => call_expr.append_ir_in_expr(ctx, target, prefered),
			ast::Expr::MemberAccess(member_access) => member_access.append_ir_value(ctx, target, prefered),
			ast::Expr::Index(index_expr) => index_expr.append_ir_value(ctx, target, prefered),
			ast::Expr::As(as_expr) => as_expr.append_ir(ctx, target, prefered),
			ast::Expr::StringLit(string_expr) => string_expr.append_ir_value(ctx, target, prefered),
		}
	}

	fn append_ir_ref<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		match self {
			ast::Expr::BinaryExpr(binary_expr) => return Err(IrGenError::new(binary_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
			ast::Expr::Name(name_expr) => name_expr.append_ir_ref(ctx, target, prefered),
			ast::Expr::Closed(closed_expr) => closed_expr.expr.append_ir_ref(ctx, target, prefered),
			ast::Expr::NumberLit(number_lit) => return Err(IrGenError::new(number_lit.span.clone(), IrGenErrorKind::InvalidLHS)),
			ast::Expr::Call(call_expr) => return Err(IrGenError::new(call_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
			ast::Expr::MemberAccess(member_access) => member_access.append_ir_ref(ctx, target, prefered),
			ast::Expr::Index(index_expr) => index_expr.append_ir_ref(ctx, target, prefered),
			ast::Expr::As(as_expr) => return Err(IrGenError::new(as_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
			ast::Expr::StringLit(string_expr) => return Err(IrGenError::new(string_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
		}
	}
}

impl ast::StringLitExpr {
	fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
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

		let id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
			None, 
			st.clone(),
			false,
			ir::Storable::Compound(ir::Compound::Struct(ir::Struct::new(vec![
				ir::StructProp::new(ir::Storable::Value(ir::Value::Ref(raw_slice)))
			])))
		));

		target.push(ir::Ins::PushGlobalRef(st.clone(), id));

		Ok(ir::ValueType::Ref(Box::new(st)))
	}
}

impl ast::AsExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
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

impl ast::IndexExpr {
	fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let el = match self.object.append_ir_value(ctx, target, None)? {
			ir::ValueType::Ref(st) => match st.as_ref() {
				ir::StorableType::Slice(t) => t.clone(),
				_ => {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
				}
			},
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
		};

		if self.arg.append_ir_value(ctx, target, Some(&ir::ValueType::UPtr))? != ir::ValueType::UPtr {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexValue));
		}

		target.push(ir::Ins::PushSliceElement(el.as_ref().clone()));

		Ok(match el.as_ref() {
			ir::StorableType::Value(val) => val.clone(),
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
		})
	}

	fn append_ir_ref<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let el = match self.object.append_ir_value(ctx, target, None)? {
			ir::ValueType::Ref(st) => match st.as_ref() {
				ir::StorableType::Slice(t) => t.clone(),
				_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
			},
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexObject))
		};

		if self.arg.append_ir_value(ctx, target, Some(&ir::ValueType::UPtr))? != ir::ValueType::UPtr {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::IllegalIndexValue));
		}

		target.push(ir::Ins::PushSliceElementRef(el.as_ref().clone()));

		Ok(ir::ValueType::Ref(el))
	}
}

impl ast::MemberAccessExpr {
	fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let object = self.object.append_ir_value(ctx, target, None)?;

		match object {
    		ir::ValueType::Ref(r) => match r.as_ref() {
				ir::StorableType::Compound(c) => {
					match c.content() {
						ir::TypeContent::Struct(s) => {
							let idx = match s.props().iter().position(|x| x.name() == self.prop) {
								Some(x) => x,
								None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist)),
							};
							let prop = s.prop(idx).unwrap();
							let t = match prop.prop_type() {
								ir::StorableType::Value(vt) => vt.clone(),
								_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
							};
							target.push(ir::Ins::PushProperty(c.clone(), t.clone(), idx));
							Ok(t)
						},
					}
				},
				ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
				ir::StorableType::Slice(st) => {
					if self.prop != "length" {
						return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist));
					}

					target.push(ir::Ins::PushSliceLen(st.as_ref().clone()));

					Ok(ir::ValueType::UPtr)
				},
				ir::StorableType::SliceData(_) => unreachable!(),
			},
			_ => unreachable!()
		}
	}

	fn append_ir_ref<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let object = self.object.append_ir_value(ctx, target, None)?;

		match object {
    		ir::ValueType::Ref(r) => match r.as_ref() {
				ir::StorableType::Compound(c) => {
					match c.content() {
						ir::TypeContent::Struct(s) => {
							let idx = match s.props().iter().position(|x| x.name() == self.prop) {
								Some(x) => x,
								None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist)),
							};
							let prop = s.prop(idx).unwrap();
							target.push(ir::Ins::PushPropertyRef(c.clone(), prop.prop_type().clone(), idx));
							Ok(ir::ValueType::Ref(Box::new(prop.prop_type().clone())))
						},
					}
				},
				ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
				ir::StorableType::Slice(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist)),
				ir::StorableType::SliceData(_) => unreachable!(),
			},
			_ => unreachable!()
		}
	}
}

impl ast::CallExpr {
	fn append_ir_in_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let (func_id, func) = match self.object.as_ref() {
			ast::Expr::Name(name) => {
				match ctx.unit.func(&name.name) {
					Some(x) => x,
					_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist))
				}
			},
			ast::Expr::MemberAccess(member_access) => {
				// Also acts as first argument:
				let v = member_access.object.append_ir_value(ctx, target, None)?;
				match v {
					ir::ValueType::Ref(r) => match r.as_ref() {
						ir::StorableType::Compound(c) => {
							match ctx.unit.method(c.name(), &member_access.prop) {
								Some(x) => x,
								_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist)),
							}
						},
						_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
					},
					_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
				}
			},
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist))
		};

		if func.return_types.len() != 1 {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallNotOneReturnInExpr));
		}
		
		if func.is_method() {
			if self.args.len() + 1 != func.params.len() {
				return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgParamCountMismatch));
			}
			
			for (a, arg) in self.args.iter().enumerate() {
				let expected = ctx.ir_unit.get_function(func_id).signature().params()[a + 1].clone();
				if arg.append_ir_value(ctx, target, Some(&expected))? != expected {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgTypeMismatch));
				}
			}
		} else {
			if self.args.len() != func.params.len() {
				return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgParamCountMismatch));
			}
			
			for (a, arg) in self.args.iter().enumerate() {
				let expected = ctx.ir_unit.get_function(func_id).signature().params()[a].clone();
				if arg.append_ir_value(ctx, target, Some(&expected))? != expected {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgTypeMismatch));
				}
			}
		}

		target.push(ir::Ins::Call(func_id));

		Ok(ctx.ir_unit.get_function(func_id).signature().returns()[0].clone())
	}

	fn append_ir_out_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<usize, IrGenError> {
		let (func_id, func) = match self.object.as_ref() {
			ast::Expr::Name(name) => {
				match ctx.unit.func(&name.name) {
					Some(x) => x,
					_ => todo!() // Possibly a local or global
				}
			},
			_ => todo!()
		};

		if self.args.len() != func.params.len() {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgParamCountMismatch));
		}

		for (a, arg) in self.args.iter().enumerate() {
			let expected = ctx.ir_unit.get_function(func_id).signature().params()[a].clone();
			if arg.append_ir_value(ctx, target, Some(&expected))? != expected {
				return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgTypeMismatch));
			}
		}

		target.push(ir::Ins::Call(func_id));

		Ok(ctx.ir_unit.get_function(func_id).signature().returns().len())
	}
}

impl ast::BinaryExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		let left = self.left.append_ir_value(ctx, target, if self.op.is_num() { prefered } else { None })?;
		let right = self.right.append_ir_value(ctx, target, if self.op.is_num() { prefered } else { None })?;

		if left != right { return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::BinaryOpTypeMismatch)) }

		target.push(match self.op {
			ast::BinaryOp::Add => ir::Ins::Add(left.clone()),
			ast::BinaryOp::Mul => ir::Ins::Mul(left.clone()),
			ast::BinaryOp::Div => ir::Ins::Div(left.clone()),
			ast::BinaryOp::Sub => ir::Ins::Sub(left.clone()),
			
			ast::BinaryOp::Eq => ir::Ins::Eq(left.clone()),
			ast::BinaryOp::Ne => ir::Ins::Ne(left.clone()),
			
			ast::BinaryOp::Lt => ir::Ins::Lt(left.clone()),
			ast::BinaryOp::Le => ir::Ins::Le(left.clone()),
			ast::BinaryOp::Gt => ir::Ins::Gt(left.clone()),
			ast::BinaryOp::Ge => ir::Ins::Ge(left.clone()),
		});

		Ok(left)
	}
}

impl ast::NameExpr {
	fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let idx = *idx;
			
			let st = ctx.func().get_local(idx).unwrap().local_type();

			match st {
				ir::StorableType::Value(vt) => {
					target.push(ir::Ins::PushLocalValue(vt.clone(), idx));
					Ok(vt.clone())
				},
				_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CompositeTypeOnStack)),
			}
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::VariableDoesNotExist))
		}
	}

	fn append_ir_ref<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let idx = *idx;
			
			let st = ctx.func().get_local(idx).unwrap().local_type();

			target.push(ir::Ins::PushLocalRef(st.clone(), idx));
			Ok(ir::ValueType::Ref(Box::new(st.clone())))
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::VariableDoesNotExist))
		}
	}
}

impl ast::NumberLitExpr {
	fn append_ir<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
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