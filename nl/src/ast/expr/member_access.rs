use syntax::Span;

use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext, storable_type_to_string};

use super::Expr;

#[derive(Debug)]
pub struct MemberAccessExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub prop: String
}

impl MemberAccessExpr {
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let object = match self.object.resultant_type(ctx, None)? {
            ir::ValueType::Ref(ref_target) => ref_target,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
        };

        match object.as_ref() {
            ir::StorableType::Compound(compound) => {
                match compound.content() {
                    ir::CompoundContent::Struct(struc) => {
                        let prop_idx = match struc.find_prop(&self.prop) {
                            Some(idx) => idx,
                            None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), compound.name().to_string()))),
                        };
                        let prop = struc.prop(prop_idx).unwrap();
                        Ok(match prop.prop_type() {
                            ir::StorableType::Value(vt) => vt.clone(),
                            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
                        })
                    },
                }
            },
            ir::StorableType::Slice(_) => {
                if self.prop != "length" {
                    return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(object.as_ref()))));
                }

                Ok(ir::ValueType::UPtr)
            },
            ir::StorableType::Value(_) => Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
            ir::StorableType::SliceData(_) => unreachable!(),
        }
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        let object = match self.object.append_ir_value(ctx, target, None)? {
            ir::ValueType::Ref(ref_target) => ref_target,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
        };

        match object.as_ref() {
            ir::StorableType::Compound(compound) =>
                match compound.content() {
                    ir::CompoundContent::Struct(struc) => {
                        let prop_idx = match struc.find_prop(&self.prop) {
                            Some(idx) => idx,
                            None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), compound.name().to_string()))),
                        };
                        let prop = struc.prop(prop_idx).unwrap();
                        let t = match prop.prop_type() {
                            ir::StorableType::Value(vt) => vt.clone(),
                            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
                        };
                        
                        target.push(ir::Ins::PushPath(ir::ValuePath::new(
                            ir::ValuePathOrigin::Deref(object.as_ref().clone()),
                            vec![
                                ir::ValuePathComponent::Property(prop_idx, compound.clone(), prop.prop_type().clone())
                            ]
                        ), t.clone()));
                        target.push(ir::Ins::Push(t.clone()));
                        Ok(t)
                    },
                },
            ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
            ir::StorableType::Slice(_) => {
                if self.prop != "length" {
                    return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(&object))));
                }

                target.push(ir::Ins::PushPath(ir::ValuePath::new(
                    ir::ValuePathOrigin::Deref(object.as_ref().clone()),
                    vec![
                        ir::ValuePathComponent::Length
                    ]
                ), ir::ValueType::UPtr));
                target.push(ir::Ins::Push(ir::ValueType::UPtr));

                Ok(ir::ValueType::UPtr)
            },
            ir::StorableType::SliceData(_) => unreachable!(),
        }
    }

    pub fn construct_path_to<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _prefered: Option<&ir::ValueType>) -> Result<(ir::StorableType, ir::ValuePath), IrGenError> {
        let object = self.object.append_ir_value(ctx, target, None)?;

        let compound = match object {
            ir::ValueType::Ref(ref_target) =>
                match ref_target.as_ref() {
                    ir::StorableType::Compound(compound) => compound.clone(),
                    ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
                    ir::StorableType::Slice(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(&ref_target)))),
                    ir::StorableType::SliceData(_) => unreachable!(),
                },
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
        };

        match compound.content() {
            ir::CompoundContent::Struct(struc) => {
                let prop_idx = match struc.find_prop(&self.prop) {
                    Some(idx) => idx,
                    None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), compound.name().to_string()))),
                };
                let prop = struc.prop(prop_idx).unwrap();

                Ok((
                    prop.prop_type().clone(), ir::ValuePath::new(
                        ir::ValuePathOrigin::Deref(ir::StorableType::Compound(compound.clone())),
                        vec![
                            ir::ValuePathComponent::Property(prop_idx, compound.clone(), prop.prop_type().clone())
                        ]
                    )
                ))
            }
        }
    }
}