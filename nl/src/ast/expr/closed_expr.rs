use syntax::Span;

use super::Expr;

#[derive(Debug)]
pub struct ClosedExpr {
	pub span: Span,
	pub expr: Box<Expr>
}