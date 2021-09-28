#[derive(Debug)]
pub struct TypeExpr {
	pub path: Vec<String>
}

#[derive(Debug)]
pub struct FunctionAnnotation {
	pub name: String
}

#[derive(Debug)]
pub struct Function {
	pub name: String,
	pub params: Vec<FunctionParam>,
	pub code: Vec<Code>,
	pub annotations: Vec<FunctionAnnotation>,
	pub return_types: Vec<TypeExpr>
}

#[derive(Debug)]
pub struct FunctionParam {
	pub name: String,
	pub param_type: TypeExpr
}

#[derive(Debug)]
pub enum Expr {
	BinaryExpr(BinaryExpr),
	Name(NameExpr),
	Closed(ClosedExpr),
	NumberLit(NumberLitExpr)
}

#[derive(Debug)]
pub struct ClosedExpr {
	pub expr: Box<Expr>
}

#[derive(Debug)]
pub struct NameExpr {
	pub name: String
}

#[derive(Debug)]
pub struct NumberLitExpr {
	pub number: String
}

#[derive(Debug)]
pub struct BinaryExpr {
	pub op: BinaryOp,
	pub left: Box<Expr>,
	pub right: Box<Expr>
}

#[derive(Debug)]
pub enum BinaryOp {
	Add, Mul, Div, Sub
}

#[derive(Debug)]
pub struct ReturnStmt {
	pub expr: Option<Expr>
}

#[derive(Debug)]
pub struct VarDeclaration {
	pub name: String,
	pub expr: Option<Expr>
}

#[derive(Debug)]
pub enum Code {
	Function(Function),
	ReturnStmt(ReturnStmt),
	VarDeclaration(VarDeclaration)
}