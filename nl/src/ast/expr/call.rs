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
    pub fn find_function_index<'a>(&'a self, ctx: &IrGenFunctionContext<'a>) -> Result<ir::FunctionIndex, IrGenError> {
        let func_idx = match self.object.as_ref() {
            Expr::Name(name) => {
                match ctx.ir_unit.find_function_index(&name.name) {
                    Some(idx) => idx,
                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(name.name.clone())))
                }
            },
            Expr::MemberAccess(member_access) => {
                let static_result = match member_access.object.as_ref() {
                    Expr::Name(name) => {
                        if let Some(compound_type) = ctx.ir_unit.find_type(&name.name) {
                            match ctx.ir_unit.find_method_index(compound_type.clone(), &member_access.prop) {
                                Some(idx) => {
                                    let func = ctx.ir_unit.get_function(idx).unwrap();
                                    if func.is_virtual() {
                                        return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::MethodNotStatic))
                                    }
                                    Some(idx)
                                },
                                _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(member_access.prop.clone()))),
                            }
                        } else {
                            None
                        }
                    },
                    _ => None
                };


                if let Some(static_result) = static_result {
                    static_result
                } else {
                    let v = member_access.object.resultant_type(ctx, None)?;
                    match v {
                        ir::ValueType::Ref(r) => match r.as_ref() {
                            ir::StorableType::Compound(c) => {
                                match ctx.ir_unit.find_method_index(c.clone(), &member_access.prop) {
                                    Some(idx) => idx,
                                    _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::FunctionDoesNotExist(member_access.prop.clone()))),
                                }
                            },
                            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
                        },
                        _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
                    }
                }
            },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS))
        };

        Ok(func_idx)
    }

    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let func = ctx.ir_unit.get_function(self.find_function_index(ctx)?).unwrap();

        // Check return count, but does not check arguments since we are only trying to determine the type - nothing more
        if func.signature().return_count() != 1 {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallNotOneReturnInExpr));
        }

        Ok(func.signature().returns()[0].clone())
    }

    fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, in_expr: bool) -> Result<ir::FunctionIndex, IrGenError> {
        // Function calls can appear in one of two places:
        //  1. As a part of an expression - where the function called must have exactly one return value so they can be used (e.g. in a binary op)
        //  2. As it's own statement - where the function called can have any number of return values, as they are all ignored

        let func_id = self.find_function_index(ctx)?;
        let func = ctx.ir_unit.get_function(func_id).unwrap();

        // If we are in an expression, check we have exactly one return argument
        if in_expr && func.signature().return_count() != 1 {
            return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallNotOneReturnInExpr));
        }
        
        if func.is_virtual() {
            // Check we have the correct number of arguments (+ 1 due to implicit self argument)
            if self.args.len() + 1 != func.signature().param_count() {
                return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::CallArgParamCountMismatch(self.args.len() + 1, func.signature().param_count())));
            }

            // Push the argument, we know it's there because we're in a virtual function
            match self.object.as_ref() {
                Expr::MemberAccess(member_access) => {
                    member_access.object.append_ir_value(ctx, target, None)?;
                },
                _ => unreachable!()
            }

            // Push the arguments to the stack...
            for (a, arg) in self.args.iter().enumerate() {
                // Unfortunate repeated lookup, necessary since append_ir_value might mutate so the borrow checker gets mad
                let expected = ctx.ir_unit.get_function(func_id).unwrap().signature().params()[a + 1].clone();
                let found = arg.append_ir_value(ctx, target, Some(&expected))?;
                if found != expected { // ...checking their types as we go
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

        target.push(ir::Ins::Call(func_id)); // Do the call

        Ok(func_id)
    }

    pub fn append_ir_in_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let index = self.append_ir(ctx, target, true)?;
        Ok(ctx.ir_unit.get_function(index).unwrap().signature().returns()[0].clone())
    }

    // Returned usize is used in Code to drop the return values
    pub fn append_ir_out_expr<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        let index = self.append_ir(ctx, target, false)?;
        
        for _ in 0..ctx.ir_unit.get_function(index).unwrap().signature().return_count() {
            target.push(ir::Ins::Drop);
        }

        Ok(())
    }
}