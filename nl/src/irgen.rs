use std::collections::HashMap;
use std::str::FromStr;

use crate::ast;
use ir;
use syntax::Span;

pub enum IrGenErrorKind {
	UnknownType,
	FunctionDoesNotExist,
	VariableDoesNotExist,
	InvalidInteger,
	BinaryOpTypeMismatch
}

impl IrGenErrorKind {
	pub fn message(&self) -> String {
		match self {
			IrGenErrorKind::UnknownType => "Unknown type".to_string(),
			IrGenErrorKind::FunctionDoesNotExist => "Function does not exist".to_string(),
			IrGenErrorKind::VariableDoesNotExist => "Variable does not exist".to_string(),
			IrGenErrorKind::InvalidInteger => "Invalid integer".to_string(),
			IrGenErrorKind::BinaryOpTypeMismatch => "Binary operation type mismatch".to_string()
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
	fn to_ir_valuetype(&self, unit: &ast::TranslationUnit) -> Result<ir::ValueType, IrGenError> {
		// There must be a first item, or else this shouldn't have parsed
		match self.path.get(0).unwrap().as_str() {
			"i32" => Ok(ir::ValueType::I32),
			"u32" => Ok(ir::ValueType::U32),
			"i64" => Ok(ir::ValueType::I64),
			"u64" => Ok(ir::ValueType::U64),
			_ => Err(IrGenError::new(self.span, IrGenErrorKind::UnknownType))
		}
	}
}

impl ast::TranslationUnit {
	fn id_of_func(&self, name: &str) -> Option<usize> {
		let mut id = 0;
		for node in &self.nodes {
			match node {
    			ast::TopLevelNode::Function(func) => {
					if func.name == name { return Some(id); }
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

		// Put params into locals
		for param in self.params.iter().rev() {
			let vt = param.param_type.to_ir_valuetype(unit)?;
			let local = ctx.push_local(&param.name, vt);
			ctx.func_mut().push(ir::Ins::PopLocal(vt, local));
		}

		for code in &self.code {
			code.append_ir(&mut ctx)?;
		}

		if ctx.func().signature().returns().len() == 0 && !matches!(ctx.func().code().last(), Some(ir::Ins::Ret)) {
			ctx.func_mut().push(ir::Ins::Ret);
		}
		
		Ok(())
	}
}

impl ast::Code {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<(), IrGenError> {
		match self {
			ast::Code::ReturnStmt(stmt) => stmt.append_ir(ctx),
			ast::Code::VarDeclaration(vardecl) => vardecl.append_ir(ctx),
		}
	}
}

impl ast::ReturnStmt {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<(), IrGenError> {
		if let Some(expr) = &self.expr {
			expr.append_ir(ctx)?;
		}

		ctx.func_mut().push(ir::Ins::Ret);

		Ok(())
	}
}

impl ast::VarDeclaration {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<(), IrGenError> {
		// TODO: Make this either inferred or explicitly given
		let idx = ctx.push_local(&self.name, ir::ValueType::I32);

		if let Some(expr) = &self.expr {
			// TODO: Type check, once types actually exist
			expr.append_ir(ctx)?;
			ctx.func_mut().push(ir::Ins::PopLocal(ir::ValueType::I32, idx));
		}

		Ok(())
	}
}

impl ast::Expr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<ir::ValueType, IrGenError> {
		match self {
			ast::Expr::BinaryExpr(binary_expr) => binary_expr.append_ir(ctx),
			ast::Expr::Name(name_expr) => name_expr.append_ir(ctx),
			ast::Expr::Closed(closed_expr) => closed_expr.append_ir(ctx),
			ast::Expr::NumberLit(number_lit) => number_lit.append_ir(ctx),
		}
	}
}

impl ast::BinaryExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<ir::ValueType, IrGenError> {
		let left = self.left.append_ir(ctx)?;
		let right = self.right.append_ir(ctx)?;

		if left != right { return Err(IrGenError::new(self.span, IrGenErrorKind::BinaryOpTypeMismatch)) }

		ctx.func_mut().push(match self.op {
			ast::BinaryOp::Add => ir::Ins::Add(left),
			ast::BinaryOp::Mul => ir::Ins::Mul(left),
			ast::BinaryOp::Div => ir::Ins::Div(left),
			ast::BinaryOp::Sub => ir::Ins::Sub(left),
		});

		Ok(left)
	}
}

impl ast::NameExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<ir::ValueType, IrGenError> {
		if let Some(idx) = ctx.local_map.get(self.name.as_str()) {
			let idx = *idx;
			
			let vt = ctx.func().get_local(idx).unwrap().value_type();
			ctx.func_mut().push(ir::Ins::PushLocal(vt, idx));
			Ok(vt)
		} else {
			Err(IrGenError::new(self.span, IrGenErrorKind::VariableDoesNotExist))
		}
	}
}

impl ast::ClosedExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<ir::ValueType, IrGenError> {
		self.expr.append_ir(ctx)
	}
}

impl ast::NumberLitExpr {
	fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>) -> Result<ir::ValueType, IrGenError> {
		if let Ok(num) = i32::from_str(&self.number) {
			ctx.func_mut().push(ir::Ins::PushLiteral(ir::ValueType::I32, num as u64));
			Ok(ir::ValueType::I32)
		} else {
			Err(IrGenError::new(self.span, IrGenErrorKind::InvalidInteger))
		}
	}
}