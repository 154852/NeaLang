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
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let st = ir::StorableType::Compound(match ctx.ir_unit.find_type("String") {
            Some(x) => x,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::StdLinkError))
        });

        Ok(ir::ValueType::Ref(Box::new(st)))
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let st = ir::StorableType::Compound(match ctx.ir_unit.find_type("String") {
            Some(x) => x,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::StdLinkError))
        });

        // 1. Store the raw bytes of the string
        let raw_data = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::SliceData(Box::new(ir::StorableType::Value(ir::ValueType::U8))),
            ir::StorableValue::SliceData(self.value.as_bytes().iter().map(|x| ir::StorableValue::Value(ir::Value::U8(*x))).collect())
        ));

        // 2. Store a slice which refers to the raw bytes
        let raw_slice = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::Slice(Box::new(ir::StorableType::Value(ir::ValueType::U8))),
            ir::StorableValue::Slice(raw_data, 0, self.value.as_bytes().len())
        ));

        // 3. Create the struct value, which refers to the slice
        let string_id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None, 
            st.clone(),
            ir::StorableValue::Compound(ir::CompoundValue::Struct(ir::StructValue::new(vec![
                ir::StructPropertyValue::new(ir::StorableValue::Value(ir::Value::Ref(raw_slice)))
            ])))
        ));

        // 4. Create a reference to the string struct
        // FIXME: Is this correct?
        let id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None, 
            ir::StorableType::Value(ir::ValueType::Ref(Box::new(st.clone()))),
            ir::StorableValue::Value(ir::Value::Ref(string_id))
        ));

        // 5. Push a path to the global
        target.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Global(id, ir::StorableType::Value(ir::ValueType::Ref(Box::new(st.clone()))))),
            ir::ValueType::Ref(Box::new(st.clone()))
        ));

        // 6. Derefence to it
        target.push(ir::Ins::Push(ir::ValueType::Ref(Box::new(st.clone()))));

        Ok(ir::ValueType::Ref(Box::new(st)))
    }
}

impl NumberLitExpr {
    pub fn resultant_type<'a>(&'a self, _ctx: &IrGenFunctionContext<'a>, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        Ok(match preferred {
            Some(vt) if vt.is_num() => vt.clone(),
            _ => ir::ValueType::I32
        })
    }

    pub fn append_ir<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        use std::str::FromStr;

        let (vt, val) = match preferred {
            // Only 0 and 1 are valid in casting to a boolean
            Some(ir::ValueType::Bool) if self.number == "0" => {
                (ir::ValueType::Bool, Ok(0))
            },
            Some(ir::ValueType::Bool) if self.number == "1" => {
                (ir::ValueType::Bool, Ok(1))
            },

            Some(ir::ValueType::U8) => (ir::ValueType::U8, u8::from_str(&self.number).map(|x| x as u64)),
            Some(ir::ValueType::I8) => (ir::ValueType::I8, i8::from_str(&self.number).map(|x| x as u64)),
            Some(ir::ValueType::U16) => (ir::ValueType::U16, u16::from_str(&self.number).map(|x| x as u64)),
            Some(ir::ValueType::I16) => (ir::ValueType::I16, i16::from_str(&self.number).map(|x| x as u64)),
            Some(ir::ValueType::U32) => (ir::ValueType::U32, u32::from_str(&self.number).map(|x| x as u64)),
            // I32 falls through to end
            Some(ir::ValueType::U64) => (ir::ValueType::U64, u64::from_str(&self.number).map(|x| x as u64)),
            Some(ir::ValueType::I64) => (ir::ValueType::I64, i64::from_str(&self.number).map(|x| x as u64)),

            // FIXME: Should these bu u/i64
            Some(ir::ValueType::UPtr) => (ir::ValueType::UPtr, u64::from_str(&self.number).map(|x| x as u64)),
            Some(ir::ValueType::IPtr) => (ir::ValueType::IPtr, i64::from_str(&self.number).map(|x| x as u64)),
            
            // Fall back to signed 32 bit
            _ => (ir::ValueType::I32, i32::from_str(&self.number).map(|x| x as u64))
        };

        target.push(ir::Ins::PushLiteral(vt.clone(), match val {
            Ok(val) => val,
            Err(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidInteger))
        }));
        
        Ok(vt)
    }
}

impl SliceLitExpr {
    fn slice_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        // If we have no elements, either use the preferred type, or fall back to i32
        if self.values.len() == 0 {
            if let Some(ir::ValueType::Ref(ref_target)) = preferred {
                if let ir::StorableType::Slice(st) = ref_target.as_ref() {
                    if let ir::StorableType::Value(vt) = st.as_ref() {
                        return Ok(vt.clone());
                    }
                }
            }

            return Ok(ir::ValueType::I32);
        }

        // Get the resultant type of the first element, using the existing preferred type
        if let Some(ir::ValueType::Ref(ref_target)) = preferred {
            if let ir::StorableType::Slice(st) = ref_target.as_ref() {
                if let ir::StorableType::Value(vt) = st.as_ref() {
                    return self.values[0].resultant_type(ctx, Some(vt));
                }
            }
        }

        // If we have no (or an invalid) preferred type, don't use one
        self.values[0].resultant_type(ctx, None)
    }

    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        Ok(ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(ir::StorableType::Value(
            self.slice_type(ctx, preferred)?
        ))))))
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let resultant_type = self.slice_type(ctx, preferred)?;

        // 1. Load all the values as compile time constants
        let mut values = Vec::with_capacity(self.values.len());
        for value in &self.values {
            values.push(ir::StorableValue::Value(value.as_value(ctx.ir_unit, &resultant_type)?));
        }

        // 2. Store the raw values of the array
        let raw_data = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::SliceData(Box::new(ir::StorableType::Value(resultant_type.clone()))),
            ir::StorableValue::SliceData(values)
        ));

        // 3. Create a slice which regers to the array
        let raw_slice = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None,
            ir::StorableType::Slice(Box::new(ir::StorableType::Value(resultant_type.clone()))),
            ir::StorableValue::Slice(raw_data, 0, self.values.len())
        ));

        // 4. Create a reference to that slice
        let slice = ir::StorableType::Slice(Box::new(ir::StorableType::Value(resultant_type.clone())));
        let id = ctx.ir_unit.add_global(ir::Global::new_default::<String>(
            None, 
            ir::StorableType::Value(ir::ValueType::Ref(Box::new(slice.clone()))),
            ir::StorableValue::Value(ir::Value::Ref(raw_slice))
        ));

        // 5. Push the path to it
        target.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Global(id, ir::StorableType::Value(ir::ValueType::Ref(Box::new(slice.clone()))))),
            ir::ValueType::Ref(Box::new(slice.clone()))
        ));

        // 6. Dereference it
        target.push(ir::Ins::Push(ir::ValueType::Ref(Box::new(slice.clone()))));

        Ok(ir::ValueType::Ref(Box::new(slice)))
    }
}

impl BoolLitExpr {
    pub fn resultant_type<'a>(&'a self, _ctx: &IrGenFunctionContext<'a>, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        Ok(ir::ValueType::Bool)
    }

    pub fn append_ir_value<'a>(&'a self, _ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        target.push(ir::Ins::PushLiteral(ir::ValueType::Bool, if self.value { 1 } else { 0 }));
        Ok(ir::ValueType::Bool)
    }
}