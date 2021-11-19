use syntax::Span;

use crate::{ast::{TranslationUnit, TypeExpr}, irgen::IrGenError, lexer::{TokenKind, TokenStream}};

#[derive(Debug)]
pub struct StructDeclaration {
	pub span: Span,
	pub name: String,
	pub fields: Vec<StructFieldDeclaration>,
}

#[derive(Debug)]
pub struct StructFieldDeclaration {
	pub span: Span,
	pub name: String,
	pub field_type: TypeExpr,
}

impl StructFieldDeclaration {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<StructFieldDeclaration> {
        let start = stream.tell_start();
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Colon), stream.error("Expected ':'"));

        let field_type = syntax::ex!(syntax::parse!(stream, TypeExpr::parse), stream.error("Expected type"));

        syntax::MatchResult::Ok(StructFieldDeclaration {
            span: syntax::Span::new(start, stream.tell_start()),
            name,
			field_type
        })
    }
}

impl StructDeclaration {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<StructDeclaration> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::StructKeyword));

        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenCurly), stream.error("Expected '{'"));

        let mut fields = Vec::new();
        loop {
            fields.push(match syntax::parse!(stream, StructFieldDeclaration::parse) {
                Some(x) => x,
                None => break
            });

            if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

        syntax::MatchResult::Ok(StructDeclaration {
            span: syntax::Span::new(start, stream.tell_start()),
            name, fields
        })
    }

    pub fn to_ir(&self, ir_unit: &ir::TranslationUnit, _unit: &TranslationUnit) -> Result<ir::CompoundTypeRef, IrGenError> {
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