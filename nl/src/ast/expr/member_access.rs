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
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        // Get the type of the object we refer to
        let object = match self.object.resultant_type(ctx, None)? {
            ir::ValueType::Ref(ref_target) => ref_target,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
        };

        match object.as_ref() {
            ir::StorableType::Compound(compound) => {
                match compound.content() {
                    ir::CompoundContent::Struct(struc) => {
                        // We have a struct, so lookup the property by name...
                        let prop_idx = match struc.find_prop(&self.prop) {
                            Some(idx) => idx,
                            None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), compound.name().to_string()))),
                        };
                        
                        let prop = struc.prop(prop_idx).unwrap();
                        
                        // ...and get it's type
                        Ok(match prop.prop_type() {
                            ir::StorableType::Value(vt) => vt.clone(),
                            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
                        })
                    },
                }
            },
            ir::StorableType::Slice(_) => {
                // The only properties slices have is length...
                if self.prop != "length" {
                    return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(object.as_ref()))));
                }

                Ok(ir::ValueType::UPtr) // ...which is a uptr
            },
            ir::StorableType::Value(_) => Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
            ir::StorableType::SliceData(_) => unreachable!(),
        }
    }

    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        // 1. Load the object onto the stack, it should be a reference
        let object = match self.object.append_ir_value(ctx, target, None)? {
            ir::ValueType::Ref(ref_target) => ref_target,
            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS))
        };

        match object.as_ref() {
            ir::StorableType::Compound(compound) =>
                match compound.content() {
                    ir::CompoundContent::Struct(struc) => {
                        // 2. If it is a struct, find the property...
                        let prop_idx = match struc.find_prop(&self.prop) {
                            Some(idx) => idx,
                            None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), compound.name().to_string()))),
                        };
                        let prop = struc.prop(prop_idx).unwrap();
                        let t = match prop.prop_type() {
                            ir::StorableType::Value(vt) => vt.clone(),
                            _ => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidRHS)),
                        };
                        
                        // ...and push a path to it
                        target.push(ir::Ins::PushPath(ir::ValuePath::new(
                            ir::ValuePathOrigin::Deref(object.as_ref().clone()),
                            vec![
                                ir::ValuePathComponent::Property(prop_idx, compound.clone(), prop.prop_type().clone())
                            ]
                        ), t.clone()));

                        // 3. Derefence that path
                        target.push(ir::Ins::Push(t.clone()));
                        Ok(t)
                    },
                },
            ir::StorableType::Slice(_) => {
                // 2. Slices only have lengths
                if self.prop != "length" {
                    return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), storable_type_to_string(&object))));
                }

                // 3. Push a path to the length of the slice
                target.push(ir::Ins::PushPath(ir::ValuePath::new(
                    ir::ValuePathOrigin::Deref(object.as_ref().clone()),
                    vec![
                        ir::ValuePathComponent::Length
                    ]
                ), ir::ValueType::UPtr));

                // 4. Dereference it
                target.push(ir::Ins::Push(ir::ValueType::UPtr));

                Ok(ir::ValueType::UPtr)
            },
            ir::StorableType::Value(_) => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::InvalidLHS)),
            ir::StorableType::SliceData(_) => unreachable!(),
        }
    }

    pub fn construct_path_to<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, _preferred: Option<&ir::ValueType>) -> Result<(ir::StorableType, ir::ValuePath), IrGenError> {
        // 1. Load the object on the stack
        let object = self.object.append_ir_value(ctx, target, None)?;

        // The object must be a compound - you cannot write to the length of a slice
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
                // 2. Find the property index
                let prop_idx = match struc.find_prop(&self.prop) {
                    Some(idx) => idx,
                    None => return Err(IrGenError::new(self.span.clone(), IrGenErrorKind::PropDoesNotExist(self.prop.clone(), compound.name().to_string()))),
                };
                let prop = struc.prop(prop_idx).unwrap();

                // 3. Create a path to that property
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