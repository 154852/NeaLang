use syntax::Span;

use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenError, IrGenErrorKind};

use super::Expr;

#[derive(Debug)]
pub struct TypeExpr {
    pub span: Span,
    pub path: Vec<String>,
    pub slice_lengths: Vec<Option<Expr>>
}

impl TypeExpr {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<TypeExpr> {
        let start = stream.tell_start();

        // 1. Parse the type path
        let mut path = Vec::new();
        loop {
            path.push(syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected identifier")).to_owned());
            stream.step();

            if !syntax::tk_iss!(stream, TokenKind::Dot) { break }
        }

        // 2. Optionally followed by [] or [expr] to indicate slices or any dimension.
        let mut slice_lengths = Vec::new();
        while syntax::tk_iss!(stream, TokenKind::OpenBracket) {
            if syntax::tk_iss!(stream, TokenKind::CloseBracket) {
                slice_lengths.push(None);
            } else {
                slice_lengths.push(Some(syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected expression"))));
                syntax::req!(syntax::tk_iss!(stream, TokenKind::CloseBracket), stream.error("Expected ']'"));
            }
        }

        syntax::MatchResult::Ok(TypeExpr {
            span: syntax::Span::new(start, stream.tell_start()),
            path, slice_lengths
        })
    }

    pub fn to_ir_base_storable_type(&self, ir_unit: &ir::TranslationUnit) -> Result<ir::StorableType, IrGenError> {
        // 1. Attempt to match the name with an iternal type
        // There must be a first item, or else this shouldn't have parsed
        match self.path.get(0).unwrap().as_str() {
            "u8" => return Ok(ir::StorableType::Value(ir::ValueType::U8)),
            "i8" => return Ok(ir::StorableType::Value(ir::ValueType::I8)),
            "u16" => return Ok(ir::StorableType::Value(ir::ValueType::U16)),
            "i16" => return Ok(ir::StorableType::Value(ir::ValueType::I16)),
            "u32" => return Ok(ir::StorableType::Value(ir::ValueType::U32)),
            "i32" => return Ok(ir::StorableType::Value(ir::ValueType::I32)),
            "u64" => return Ok(ir::StorableType::Value(ir::ValueType::U64)),
            "i64" => return Ok(ir::StorableType::Value(ir::ValueType::I64)),
            "uptr" => return Ok(ir::StorableType::Value(ir::ValueType::UPtr)),
            "iptr" => return Ok(ir::StorableType::Value(ir::ValueType::IPtr)),
            _ => {}
        }

        // 2. If that fails, look for the type in the unit
        if let Some(ct) = ir_unit.find_type(&self.path.get(0).unwrap()) {
            return Ok(ir::StorableType::Compound(ct));
        }

        Err(IrGenError::new(self.span.clone(), IrGenErrorKind::UnknownType))
    }

    pub fn to_ir_storable_type(&self, ir_unit: &ir::TranslationUnit) -> Result<ir::StorableType, IrGenError> {
        let mut st = self.to_ir_base_storable_type(ir_unit)?;

        for _ in 0..self.slice_lengths.len() {
            st = ir::StorableType::Slice(Box::new(st));
        }

        Ok(st)
    }

    /// This is where NL feels more like java or python that C, in that objects are always treated as pointers.
    pub fn to_ir_value_type(&self, ir_unit: &ir::TranslationUnit) -> Result<ir::ValueType, IrGenError> {
        match self.to_ir_storable_type(ir_unit)? {
            ir::StorableType::Compound(ct) => Ok(ir::ValueType::Ref(Box::new(ir::StorableType::Compound(ct)))),
            ir::StorableType::Slice(st) => Ok(ir::ValueType::Ref(Box::new(ir::StorableType::Slice(st)))),
            ir::StorableType::Value(v) => Ok(v),
            ir::StorableType::SliceData(_) => unreachable!()
        }
    }
}

impl PartialEq for TypeExpr {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}