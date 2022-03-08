use std::collections::HashMap;

use syntax::Span;

use crate::ast::{Code, Expr, TranslationUnit, TypeExpr};
use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};

#[derive(Debug)]
pub struct FunctionAnnotation {
    pub span: Span,
    pub name: String,
    pub value: Option<Expr>
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
    pub return_types: Vec<TypeExpr>,
    pub is_static: bool
}

impl Function {
    /// Tests if this function should be compiled against the given arch - can be filtered with an arch annotation
    pub fn arch_matches(&self, target_arch: &str) -> Result<bool, IrGenError> {
        for annotation in &self.annotations {
            if annotation.name == "arch" {
                let archs = match &annotation.value {
                    Some(Expr::StringLit(string)) => string.value.split(','),
                    _ => return Err(IrGenError::new(annotation.span.clone(), IrGenErrorKind::InvalidAnnotationExpression("string".to_string())))
                };

                for arch in archs {
                    if arch == target_arch { return Ok(true) }
                }

                return Ok(false);
            }
        }

        return Ok(true);
    }

    /// Create the signature / method_of etc fields for a function - everything but the code, in effect.
    /// This means that an imported function will have an ir_base but not full ir.
    /// This function expects that arch_matches
    pub fn to_ir_base(&self, ir_unit: &ir::TranslationUnit, _unit: &TranslationUnit) -> Result<ir::Function, IrGenError> {
        let mut returns = Vec::with_capacity(self.return_types.len());
        for return_type in &self.return_types {
            returns.push(return_type.to_ir_value_type(ir_unit)?);
        }

        let mut func = if self.path.len() > 0 {
            assert_eq!(self.path.len(), 1); // Currently we only support associating a function with a type, but nothing more
            
            // Find the type
            let ctr = match ir_unit.find_type(&self.path[0]) {
                Some(x) => x,
                None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::UnknownType))
            };

            let mut params = Vec::with_capacity(self.params.len() + if self.is_static { 0 } else { 1 });
            
            // For virtual functions (methods), push self as an argument, before the others
            if !self.is_static {
                params.push(ir::ValueType::Ref(Box::new(ir::StorableType::Compound(ctr.clone()))));
            }

            for param in &self.params {
                params.push(param.param_type.to_ir_value_type(ir_unit)?);
            }

            let method_data = if self.is_static {
                ir::MethodData::new_static(ctr)
            } else {
                ir::MethodData::new_virtual(ctr)
            };

            if self.code.is_some() {
                ir::Function::new_method(&self.name, ir::Signature::new(params, returns), method_data)
            } else {
                ir::Function::new_extern_method(&self.name, ir::Signature::new(params, returns), method_data)
            }
        } else {
            let mut params = Vec::with_capacity(self.params.len());
            for param in &self.params {
                params.push(param.param_type.to_ir_value_type(ir_unit)?);
            }

            if self.code.is_some() {
                ir::Function::new(&self.name, ir::Signature::new(params, returns))
            } else {
                ir::Function::new_extern(&self.name, ir::Signature::new(params, returns))
            }
        };

        // Push the function annotations to the ir
        for annotation in &self.annotations {
            match annotation.name.as_str() {
                "entry" => func.push_attr(ir::FunctionAttr::Entry),
                "alloc" => func.push_attr(ir::FunctionAttr::Alloc),
                "alloc_slice" => func.push_attr(ir::FunctionAttr::AllocSlice),
                "free" => func.push_attr(ir::FunctionAttr::Free),
                "free_slice" => func.push_attr(ir::FunctionAttr::FreeSlice),
                "location" =>
                    match &annotation.value {
                        Some(Expr::StringLit(string)) => {
                            func.push_attr(ir::FunctionAttr::ExternLocation(string.value.clone()))
                        },
                        _ => return Err(IrGenError::new(annotation.span.clone(), IrGenErrorKind::InvalidAnnotationExpression("string".to_string())))
                    },
                "arch" => {},
                _ => return Err(IrGenError::new(annotation.span.clone(), IrGenErrorKind::UnknownAnnotation(annotation.name.clone())))
            }
        }

        Ok(func)
    }

    /// Push the actual code to this function.
    /// This assumes idx points to the result of append_ir_base for this function.
    pub fn append_ir(&self, ir_unit: &mut ir::TranslationUnit, idx: ir::FunctionIndex) -> Result<(), IrGenError> {
        let mut ctx = IrGenFunctionContext {
            ir_unit,
            function_idx: idx,
            local_map: HashMap::new()
        };

        if !self.is_static {
            // For virtual functions, define the name self as the first local
            ctx.push_local("self", ir::StorableType::Value(
                ir::ValueType::Ref(Box::new(ir::StorableType::Compound(ctx.func().method_of().unwrap())))
            ));
        }

        // Push the params and their names
        for param in &self.params {
            let vt = param.param_type.to_ir_value_type(ctx.ir_unit)?;
            ctx.push_local(&param.name, ir::StorableType::Value(vt.clone()));
        }

        // Push the code
        let mut target = IrGenCodeTarget::new();
        for code in self.code.as_ref().unwrap() {
            code.append_ir(&mut ctx, &mut target)?;
        }

        // Add a trailing ret if we return void
        if ctx.func().signature().return_count() == 0 && !matches!(ctx.func().code().last(), Some(ir::Ins::Ret)) {
            target.push(ir::Ins::Ret);
        }

        // Add the code to the function
        ctx.func_mut().code_mut().extend(target.take());
        
        Ok(())
    }

    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Function> {
        let start = stream.tell_start();
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::FuncKeyword));

        // 1. Parse function annotations
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

        // 2. Parse function name
        let mut name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

        // 3. Parse path / name
        let mut path = Vec::new();
        while syntax::tk_iss!(stream, TokenKind::Dot) {
            path.push(name);
            
            name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
            stream.step();
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenParen), stream.error("Expected '('"));

        let mut is_static = true;

        let mut params = Vec::new();
        loop {
            // If there is a non-empty path, and this is the first param, we are a virtual method
            if params.len() == 0 && is_static && path.len() > 0 && syntax::tk_iss!(stream, TokenKind::SelfKeyword) {
                is_static = false;
            } else {
                params.push(match syntax::parse!(stream, FunctionParam::parse) {
                    Some(x) => x,
                    None => break
                });
            }

            if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));

        // 4. Parse return values, either nothing, a single value or a collection of values
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

        // 5. Either we get code between { }, or this is extern
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
            annotations,
            is_static
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

        let value = if syntax::tk_iss!(stream, TokenKind::Eq) {
            Some(syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected expression")))
        } else {
            None
        };

        syntax::MatchResult::Ok(FunctionAnnotation {
            span: syntax::Span::new(start, stream.tell_start()),
            name,
            value
        })
    }
}