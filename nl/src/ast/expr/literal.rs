use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};

use super::Expr;

#[derive(Debug)]
pub struct NumberLitExpr {
    pub span: Span,
    pub number: String
}

#[derive(Debug)]
pub struct StringLitExpr {
    pub span: Span,
    pub value: String
}

#[derive(Debug)]
pub struct SliceLitExpr {
    pub span: Span,
    pub values: Vec<Expr>
}

#[derive(Debug)]
pub struct BoolLitExpr {
    pub span: Span,
    pub value: bool
}

impl StringLitExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let st = ir::StorableType::Compound(match ctx.ir_unit.find_type("String") {
            Some(x) => x,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::StdLinkError))
        });

        Ok(ir::ValueType::Ref(Box::new(st)))
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let st = ir::StorableType::Compound(match ctx.ir_unit.find_type("String") {
            Some(x) => x,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::StdLinkError))
        });

        let raw_data = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::SliceData(Box::new(ir::StorableType::Value(ir::ValueType::U8))),
            false,
            ir::StorableValue::SliceData(self.value.as_bytes().iter().map(|x| ir::StorableValue::Value(ir::Value::U8(*x))).collect())
        ));

        let raw_slice = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::Slice(Box::new(ir::StorableType::Value(ir::ValueType::U8))),
            false,
            ir::StorableValue::Slice(raw_data, 0, self.value.as_bytes().len())
        ));

        let string_id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None, 
            st.clone(),
            false,
            ir::StorableValue::Compound(ir::CompoundValue::Struct(ir::StructValue::new(vec![
                ir::StructPropertyValue::new(ir::StorableValue::Value(ir::Value::Ref(raw_slice)))
            ])))
        ));

        let id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None, 
            ir::StorableType::Value(ir::ValueType::Ref(Box::new(st.clone()))),
            false,
            ir::StorableValue::Value(ir::Value::Ref(string_id))
        ));

        target.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Global(id, ir::StorableType::Value(ir::ValueType::Ref(Box::new(st.clone()))))),
            ir::ValueType::Ref(Box::new(st.clone()))
        ));

        target.push(ir::Ins::Push(ir::ValueType::Ref(Box::new(st.clone()))));

        Ok(ir::ValueType::Ref(Box::new(st)))
    }
}

impl NumberLitExpr {
    pub fn resultant_type<'a>(&'a self, _ctx: &IrGenFunctionContext<'a>, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        Ok(match prefered {
            Some(vt) if vt.is_num() => vt.clone(),
            _ => ir::ValueType::I32
        })
    }

    pub fn append_ir<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        use std::str::FromStr;
        
        if let Ok(num) = i32::from_str(&self.number) {
            let vt = match prefered {
                Some(vt) if vt.is_num() => vt,
                _ => &ir::ValueType::I32
            };
            target.push(ir::Ins::PushLiteral(vt.clone(), num as u64));
            Ok(vt.clone())
        } else {
            Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidInteger))
        }
    }
}

impl SliceLitExpr {
    fn slice_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        if self.values.len() == 0 {
            if let Some(ir::ValueType::Ref(ref_target)) = prefered {
                if let ir::StorableType::Slice(st) = ref_target.as_ref() {
                    if let ir::StorableType::Value(vt) = st.as_ref() {
                        return Ok(vt.clone());
                    }
                }
            }

            return Ok(ir::ValueType::I32);
        }

        if let Some(ir::ValueType::Ref(ref_target)) = prefered {
            if let ir::StorableType::Slice(st) = ref_target.as_ref() {
                if let ir::StorableType::Value(vt) = st.as_ref() {
                    return self.values[0].resultant_type(ctx, Some(vt));
                }
            }
        }

        self.values[0].resultant_type(ctx, None)
    }

    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        Ok(ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(ir::StorableType::Value(
            self.slice_type(ctx, prefered)?
        ))))))
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let resultant_type = self.slice_type(ctx, prefered)?;

        let mut values = Vec::with_capacity(self.values.len());
        for value in &self.values {
            values.push(ir::StorableValue::Value(value.as_value(ctx.ir_unit, &resultant_type)?));
        }

        let raw_data = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::SliceData(Box::new(ir::StorableType::Value(resultant_type.clone()))),
            false,
            ir::StorableValue::SliceData(values)
        ));

        let raw_slice = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::Slice(Box::new(ir::StorableType::Value(resultant_type.clone()))),
            false,
            ir::StorableValue::Slice(raw_data, 0, self.values.len())
        ));

        let slice = ir::StorableType::Slice(Box::new(ir::StorableType::Value(resultant_type.clone())));
        let id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None, 
            ir::StorableType::Value(ir::ValueType::Ref(Box::new(slice.clone()))),
            false,
            ir::StorableValue::Value(ir::Value::Ref(raw_slice))
        ));

        target.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Global(id, ir::StorableType::Value(ir::ValueType::Ref(Box::new(slice.clone()))))),
            ir::ValueType::Ref(Box::new(slice.clone()))
        ));

        target.push(ir::Ins::Push(ir::ValueType::Ref(Box::new(slice.clone()))));

        Ok(ir::ValueType::Ref(Box::new(slice)))
    }
}

impl BoolLitExpr {
    pub fn resultant_type<'a>(&'a self, _ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        Ok(ir::ValueType::Bool)
    }

    pub fn append_ir_value<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        target.push(ir::Ins::PushLiteral(ir::ValueType::Bool, if self.value { 1 } else { 0 }));
        Ok(ir::ValueType::Bool)
    }
}