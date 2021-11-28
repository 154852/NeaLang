use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext, value_type_to_string};

use super::Expr;

#[derive(Debug)]
pub struct CallExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub args: Vec<Expr>
}

impl CallExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let func = match self.object.as_ref() {
            Expr::Name(name) => {
                match ctx.ir_unit.find_function_index(&name.name) {
                    Some(idx) => ctx.ir_unit.get_function(idx).unwrap(),
                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(name.name.clone())))
                }
            },
            Expr::MemberAccess(member_access) => {
                let v = member_access.object.resultant_type(ctx, None)?;
                match v {
                    ir::ValueType::Ref(r) => match r.as_ref() {
                        ir::StorableType::Compound(c) => {
                            match ctx.ir_unit.find_method_index(c.clone(), &member_access.prop) {
                                Some(idx) => ctx.ir_unit.get_function(idx).unwrap(),
                                _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(member_access.prop.clone()))),
                            }
                        },
                        _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
                    },
                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
                }
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
        };

        if func.signature().return_count() != 1 {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallNotOneReturnInExpr));
        }

        Ok(func.signature().returns()[0].clone())
    }

    fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, in_expr: bool) -> Result<ir::FunctionIndex, IrGenError> {
        let (func_id, func) = match self.object.as_ref() {
            Expr::Name(name) => {
                match ctx.ir_unit.find_function_index(&name.name) {
                    Some(x) => (x, ctx.ir_unit.get_function(x).unwrap()),
                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(name.name.clone())))
                }
            },
            Expr::MemberAccess(member_access) => {
                // Also acts as first argument
                let v = member_access.object.append_ir_value(ctx, target, None)?;
                match v {
                    ir::ValueType::Ref(r) => match r.as_ref() {
                        ir::StorableType::Compound(c) => {
                            match ctx.ir_unit.find_method_index(c.clone(), &member_access.prop) {
                                Some(x) => (x, ctx.ir_unit.get_function(x).unwrap()),
                                _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(member_access.prop.clone()))),
                            }
                        },
                        _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
                    },
                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
                }
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
        };

        if in_expr && func.signature().return_count() != 1 {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallNotOneReturnInExpr));
        }
        
        if func.method_of().is_some() {
            if self.args.len() + 1 != func.signature().param_count() {
                return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgParamCountMismatch(self.args.len() + 1, func.signature().param_count())));
            }
            
            for (a, arg) in self.args.iter().enumerate() {
                let expected = ctx.ir_unit.get_function(func_id).unwrap().signature().params()[a + 1].clone();
                let found = arg.append_ir_value(ctx, target, Some(&expected))?;
                if found != expected {
                    return Err(IrGenError::new(arg.span().clone(), IrGenErrorKind::CallArgTypeMismatch(value_type_to_string(&found), value_type_to_string(&expected))));
                }
            }
        } else {
            if self.args.len() != func.signature().param_count() {
                return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgParamCountMismatch(self.args.len(), func.signature().param_count())));
            }
            
            for (a, arg) in self.args.iter().enumerate() {
                let expected = ctx.ir_unit.get_function(func_id).unwrap().signature().params()[a].clone();
                let found = arg.append_ir_value(ctx, target, Some(&expected))?;
                if found != expected {
                    return Err(IrGenError::new(arg.span().clone(), IrGenErrorKind::CallArgTypeMismatch(value_type_to_string(&found), value_type_to_string(&expected))));
                }
            }
        }

        target.push(ir::Ins::Call(func_id));

        Ok(func_id)
    }

    pub fn append_ir_in_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let index = self.append_ir(ctx, target, true)?;
        Ok(ctx.ir_unit.get_function(index).unwrap().signature().returns()[0].clone())
    }

    pub fn append_ir_out_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<usize, IrGenError> {
        let index = self.append_ir(ctx, target, false)?;
        Ok(ctx.ir_unit.get_function(index).unwrap().signature().return_count())
    }
}