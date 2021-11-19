use std::collections::HashMap;

use syntax::Span;

use crate::{ast::{Code, TranslationUnit, TypeExpr}, irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext}, lexer::{TokenKind, TokenStream}};

#[derive(Debug)]
pub struct FunctionAnnotation {
	pub span: Span,
	pub name: String
}

#[derive(Debug)]
pub struct FunctionParam {
	pub span: Span,
	pub name: String,
	pub param_type: TypeExpr
}

#[derive(Debug)]
pub struct Function {
	pub span: Span,
	pub path: Vec<String>,
	pub name: String,
	pub params: Vec<FunctionParam>,
	pub code: Option<Vec<Code>>,
	pub annotations: Vec<FunctionAnnotation>,
	pub return_types: Vec<TypeExpr>
}

impl Function {
	pub fn to_ir_base(&self, ir_unit: &ir::TranslationUnit, _unit: &TranslationUnit) -> Result<ir::Function, IrGenError> {
		let mut params = Vec::with_capacity(self.params.len());
		for param in &self.params {
			params.push(param.param_type.to_ir_value_type(ir_unit)?);
		}

		let mut returns = Vec::with_capacity(self.return_types.len());
		for return_type in &self.return_types {
			returns.push(return_type.to_ir_value_type(ir_unit)?);
		}

		let mut func = if self.path.len() > 0 {
			assert_eq!(self.path.len(), 1);
			let ctr = match ir_unit.find_type(&self.path[0]) {
				Some(x) => x,
				None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::UnknownType))
			};

			if self.code.is_some() {
				ir::Function::new_method(&self.name, ir::Signature::new(params, returns), ctr)
			} else {
				ir::Function::new_extern_method(&self.name, ir::Signature::new(params, returns), ctr)
			}
		} else {
			if self.code.is_some() {
				ir::Function::new(&self.name, ir::Signature::new(params, returns))
			} else {
				ir::Function::new_extern(&self.name, ir::Signature::new(params, returns))
			}
		};

		for annotation in &self.annotations {
			match annotation.name.as_str() {
				"entry" => func.set_entry(),
				"alloc" => func.set_alloc(),
				"alloc_slice" => func.set_alloc_slice(),
				_ => return Err(IrGenError::new(annotation.span.clone(), IrGenErrorKind::UnknownAnnotation(annotation.name.clone())))
			}
		}

		Ok(func)
	}

	pub fn append_ir(&self, ir_unit: &mut ir::TranslationUnit, idx: ir::FunctionIndex) -> Result<(), IrGenError> {
		let mut ctx = IrGenFunctionContext {
			ir_unit,
			function_idx: idx,
			local_map: HashMap::new()
		};

		for param in &self.params {
			let vt = param.param_type.to_ir_value_type(ctx.ir_unit)?;
			ctx.push_local(&param.name, ir::StorableType::Value(vt.clone()));
		}

		let mut target = IrGenCodeTarget::new();

		// Safe to unwrap as we wouldn't be here otherwise
		for code in self.code.as_ref().unwrap() {
			code.append_ir(&mut ctx, &mut target)?;
		}

		if ctx.func().signature().return_count() == 0 && !matches!(ctx.func().code().last(), Some(ir::Ins::Ret)) {
			target.push(ir::Ins::Ret);
		}

		ctx.func_mut().code_mut().extend(target.take());
		
		Ok(())
	}

	pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Function> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::FuncKeyword));

        let mut annotations = Vec::new();
        if syntax::tk_iss!(stream, TokenKind::OpenBracket) {
            loop {
                annotations.push(match syntax::parse!(stream, FunctionAnnotation::parse) {
                    Some(x) => x,
                    None => break
                });
    
                if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
            }

            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseBracket), stream.error("Expected ']'"));
        }

        let mut name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

        let mut path = Vec::new();
        while syntax::tk_iss!(stream, TokenKind::Dot) {
            path.push(name);
            
            name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
            stream.step();
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenParen), stream.error("Expected '('"));

        let mut params = Vec::new();
        loop {
            params.push(match syntax::parse!(stream, FunctionParam::parse) {
                Some(x) => x,
                None => break
            });

            if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));

        let mut returns = Vec::new();
        if syntax::tk_iss!(stream, TokenKind::Colon) {
            if syntax::tk_iss!(stream, TokenKind::OpenParen) {
                loop {
                    returns.push(match syntax::parse!(stream, TypeExpr::parse) {
                        Some(x) => x,
                        None => break
                    });
        
                    if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
                }

                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
            } else {
                returns.push(syntax::ex!(syntax::parse!(stream, TypeExpr::parse), stream.error("Expected return type")));
            }
        }

        let end = stream.tell_start();

        let code = if syntax::tk_iss!(stream, TokenKind::ExternKeyword) {
            None
        } else {
            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenCurly), stream.error("Expected '{'"));

            let mut code = Vec::new();
            loop {
                code.push(match syntax::parse!(stream, Code::parse, true) {
                    Some(x) => x,
                    None => break
                });
            }
    
            syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

            Some(code)
        };
    
        syntax::MatchResult::Ok(Function {
            span: syntax::Span::new(start, end),
            path, name, params, code,
			return_types: returns,
			annotations
        })
    }
}

impl FunctionParam {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<FunctionParam> {
        let start = stream.tell_start();
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Colon), stream.error("Expected ':'"));

        let param_type = syntax::ex!(syntax::parse!(stream, TypeExpr::parse), stream.error("Expected type"));

        syntax::MatchResult::Ok(FunctionParam {
            span: syntax::Span::new(start, stream.tell_start()),
            name,
			param_type
        })
    }
}

impl FunctionAnnotation {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<FunctionAnnotation> {
        let start = stream.tell_start();
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::MatchResult::Ok(FunctionAnnotation {
            span: syntax::Span::new(start, stream.tell_start()),
            name,
        })
    }
}