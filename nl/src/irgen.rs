use std::collections::HashMap;
use std::str::FromStr;

use crate::ast;
use ir;
use syntax::Span;

pub enum IrGenErrorKind {
	UnknownType,
	VariableDoesNotExist,
	InvalidInteger,
	BinaryOpTypeMismatch,
	AssignmentTypeMismatch,
	CannotInferType,
	CallArgParamCountMismatch,
	CallArgTypeMismatch,
	CallNotOneReturnInExpr,
	InvalidLHS
}

impl IrGenErrorKind {
	pub fn message(&self) -> String {
		match self {
			IrGenErrorKind::UnknownType => "Unknown type".to_string(),
			IrGenErrorKind::VariableDoesNotExist => "Variable does not exist".to_string(),
			IrGenErrorKind::InvalidInteger => "Invalid integer".to_string(),
			IrGenErrorKind::BinaryOpTypeMismatch => "Binary operation type mismatch".to_string(),
			IrGenErrorKind::AssignmentTypeMismatch => "Assignment type mismatch".to_string(),
			IrGenErrorKind::CannotInferType => "Cannot infer type".to_string(),
			IrGenErrorKind::CallArgParamCountMismatch => "Incorrect number of arguments to function".to_string(),
			IrGenErrorKind::CallArgTypeMismatch => "Incorrect argument type".to_string(),
			IrGenErrorKind::CallNotOneReturnInExpr => "Can only call functions with one return value in an expression".to_string(),
			IrGenErrorKind::InvalidLHS => "Invalid left hand side".to_string(),
		}
	}
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
		self.kind.message()
	}
}

impl ast::TypeExpr {
	fn to_ir_valuetype(&self, _unit: &ast::TranslationUnit) -> Result<ir::ValueType, IrGenError> {
		// There must be a first item, or else this shouldn't have parsed
		match self.path.get(0).unwrap().as_str() {
			"i32" => Ok(ir::ValueType::I32),
			"u32" => Ok(ir::ValueType::U32),
			"i64" => Ok(ir::ValueType::I64),
			"u64" => Ok(ir::ValueType::U64),
			_ => Err(IrGenError::new(self.span.clone(), IrGenErrorKind::UnknownType))
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
					if func.name == name { return Some((id, func)); }
					id += 1;
				},
			}
		}

		None
	}

	pub fn to_ir(&self) -> Result<ir::TranslationUnit, IrGenError> {
		let mut unit = ir::TranslationUnit::new();

		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					unit.add_function(func.to_ir_base(self)?);
				},
			}
		}

		let mut id = 0;
		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					func.append_ir(self, &mut unit, id)?;
					id += 1;
				},
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

	fn push_local(&mut self, name: &'a str, vt: ir::ValueType) -> ir::LocalIndex {
		let idx = self.func_mut().push_local(ir::Local::new(vt));
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

impl ast::Function {
	fn to_ir_base(&self, unit: &ast::TranslationUnit) -> Result<ir::Function, IrGenError> {
		let mut params = Vec::with_capacity(self.params.len());
		for param in &self.params {
			params.push(param.param_type.to_ir_valuetype(unit)?);
		}

		let mut returns = Vec::with_capacity(self.return_types.len());
		for return_type in &self.return_types {
			returns.push(return_type.to_ir_valuetype(unit)?);
		}

		Ok(ir::Function::new(&self.name, ir::Signature::new(params, returns)))
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
			let vt = param.param_type.to_ir_valuetype(unit)?;
			let local = ctx.push_local(&param.name, vt);
			target.push(ir::Ins::PopLocal(vt, local));
		}

		for code in &self.code {
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
						expr.append_ir(ctx, target)?;
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
		}
	}
}

impl ast::IfStmt {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		self.condition.append_ir(ctx, target)?;

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
		let vt = self.right.append_ir(ctx, target)?;

		match &self.left {
			ast::Expr::Name(name) => {
				if let Some(local_idx) = ctx.local_map.get(name.name.as_str()) {
					let local_idx = *local_idx;
					let local = ctx.func().get_local(local_idx).unwrap();
					if local.value_type() != vt {
						return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
					}

					target.push(ir::Ins::PopLocal(vt, local_idx));
				} else {
					todo!() // Global?
				}
			},
			_ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
		}

		Ok(())
	}
}

impl ast::ReturnStmt {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		if let Some(expr) = &self.expr {
			expr.append_ir(ctx, target)?;
		}

		target.push(ir::Ins::Ret);

		Ok(())
	}
}

impl ast::VarDeclaration {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
		let mut expr_type = if let Some(expr) = &self.expr {
			Some(expr.append_ir(ctx, target)?)
		} else {
			None
		};

		if let Some(var_type) = &self.var_type {
			let var_type = var_type.to_ir_valuetype(ctx.unit)?;
			if let Some(expr_type) = expr_type {
				if var_type != expr_type {
					return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
				}
			} else {
				expr_type = Some(var_type);
			}
		} else if expr_type.is_none() {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CannotInferType));
		}

		let idx = ctx.push_local(&self.name, expr_type.unwrap());

		if self.expr.is_some() {
			target.push(ir::Ins::PopLocal(expr_type.unwrap(), idx));
		}

		Ok(())
	}
}

impl ast::Expr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<ir::ValueType, IrGenError> {
		match self {
			ast::Expr::BinaryExpr(binary_expr) => binary_expr.append_ir(ctx, target),
			ast::Expr::Name(name_expr) => name_expr.append_ir(ctx, target),
			ast::Expr::Closed(closed_expr) => closed_expr.append_ir(ctx, target),
			ast::Expr::NumberLit(number_lit) => number_lit.append_ir(ctx, target),
			ast::Expr::Call(call_expr) => call_expr.append_ir_in_expr(ctx, target),
		}
	}
}

impl ast::CallExpr {
	fn append_ir_in_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<ir::ValueType, IrGenError> {
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

		if func.return_types.len() != 1 {
			return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallNotOneReturnInExpr));
		}

		for (a, arg) in self.args.iter().enumerate() {
			if arg.append_ir(ctx, target)? != ctx.ir_unit.get_function(func_id).signature().params()[a] {
				return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgTypeMismatch));
			}
		}

		target.push(ir::Ins::Call(func_id));

		Ok(ctx.ir_unit.get_function(func_id).signature().returns()[0])
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
			if arg.append_ir(ctx, target)? != ctx.ir_unit.get_function(func_id).signature().params()[a] {
				return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgTypeMismatch));
			}
		}

		target.push(ir::Ins::Call(func_id));

		Ok(ctx.ir_unit.get_function(func_id).signature().returns().len())
	}
}

impl ast::BinaryExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<ir::ValueType, IrGenError> {
		let left = self.left.append_ir(ctx, target)?;
		let right = self.right.append_ir(ctx, target)?;

		if left != right { return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::BinaryOpTypeMismatch)) }

		target.push(match self.op {
			ast::BinaryOp::Add => ir::Ins::Add(left),
			ast::BinaryOp::Mul => ir::Ins::Mul(left),
			ast::BinaryOp::Div => ir::Ins::Div(left),
			ast::BinaryOp::Sub => ir::Ins::Sub(left),
			
			ast::BinaryOp::Eq => ir::Ins::Eq(left),
			ast::BinaryOp::Ne => ir::Ins::Ne(left),
			
			ast::BinaryOp::Lt => ir::Ins::Lt(left),
			ast::BinaryOp::Le => ir::Ins::Le(left),
			ast::BinaryOp::Gt => ir::Ins::Gt(left),
			ast::BinaryOp::Ge => ir::Ins::Ge(left),
		});

		Ok(left)
	}
}

impl ast::NameExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<ir::ValueType, IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let idx = *idx;
			
			let vt = ctx.func().get_local(idx).unwrap().value_type();
			target.push(ir::Ins::PushLocal(vt, idx));
			Ok(vt)
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::VariableDoesNotExist))
		}
	}
}

impl ast::ClosedExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<ir::ValueType, IrGenError> {
		self.expr.append_ir(ctx, target)
	}
}

impl ast::NumberLitExpr {
	fn append_ir<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<ir::ValueType, IrGenError> {
		if let Ok(num) = i32::from_str(&self.number) {
			target.push(ir::Ins::PushLiteral(ir::ValueType::I32, num as u64));
			Ok(ir::ValueType::I32)
		} else {
			Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidInteger))
		}
	}
}