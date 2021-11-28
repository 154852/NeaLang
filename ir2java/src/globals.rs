use crate::{TranslationContext};

impl<'a> TranslationContext<'a> {
	pub(crate) fn translate_storable(&self, storable: &ir::StorableValue, storable_type: &ir::StorableType, classfile: &mut java::ClassFile, insns: &mut Vec<java::Ins>) {
		match (storable, storable_type) {
			(ir::StorableValue::Compound(compound_value), ir::StorableType::Compound(compound_type)) => {
				let name = crate::util::class_name_for_compound(classfile, compound_type);

				insns.push(java::Ins::New { index: classfile.const_class(&name) });
				insns.push(java::Ins::Dup);
				let method = classfile.const_method(&name, "<init>", "()V");
				insns.push(java::Ins::InvokeSpecial { index: method });

				match (compound_value, compound_type.content()) {
					(ir::CompoundValue::Struct(struct_value), ir::CompoundContent::Struct(struct_type)) => {
						for (property_value, property_type) in struct_value.props().iter().zip(struct_type.props()) {
							insns.push(java::Ins::Dup);
							self.translate_storable(property_value.value(), property_type.prop_type(), classfile, insns);
							insns.push(java::Ins::PutField {
								index: classfile.const_field(&name, property_type.name(), &crate::util::storable_type_to_descriptor(property_type.prop_type(), classfile).to_string())
							});
						}
					},
				}
			},
			(ir::StorableValue::Value(value), ir::StorableType::Value(_)) => {
				match value {
					ir::Value::U8(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::I8(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::Bool(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::U16(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::I16(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::U32(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::I32(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::UPtr(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::IPtr(value) => insns.push(java::Ins::SIPush { value: *value as i16 }),
					ir::Value::U64(_) => todo!(),
					ir::Value::I64(_) => todo!(),
					ir::Value::Ref(global_index) => {
						let global = self.unit().get_global(*global_index).unwrap();
						let name = crate::util::field_name_for_global(global, *global_index);
						insns.push(java::Ins::GetStatic {
							index: classfile.const_field(&classfile.name().to_string(), &name, &crate::util::storable_type_to_descriptor(global.global_type(), classfile).to_string())
						});
					},
				}
			},
			(ir::StorableValue::Slice(owned_index, index, length), ir::StorableType::Slice(_)) => {
				let owned = self.unit().get_global(*owned_index).unwrap();

				match (owned.global_type(), owned.default()) {
					(ir::StorableType::SliceData(_), Some(ir::StorableValue::SliceData(data))) => {
						assert!(*index == 0 && *length == data.len());
					},
					_ => panic!("Invalid slice reference")
				}

				let name = crate::util::field_name_for_global(owned, *owned_index);
				insns.push(java::Ins::GetStatic {
					index: classfile.const_field(&classfile.name().to_string(), &name, &crate::util::storable_type_to_descriptor(owned.global_type(), classfile).to_string())
				});
			},
			(ir::StorableValue::SliceData(elements), ir::StorableType::SliceData(slice_type)) => {
				insns.push(java::Ins::SIPush { value: elements.len() as i16 });

				let descriptor = crate::util::storable_type_to_descriptor(slice_type, &classfile);
				if let java::Descriptor::Reference(name) = descriptor {
					insns.push(java::Ins::ANewArray { index: classfile.const_class(&name) });
				} else {
					insns.push(java::Ins::NewArray { atype: descriptor });
				}

				for (e, element) in elements.iter().enumerate() {
					insns.push(java::Ins::Dup);
					insns.push(java::opt::ins::iconst(e as i32, classfile));
					self.translate_storable(element, slice_type, classfile, insns);
					
					insns.push(java::opt::ins::astore(&crate::util::storable_type_to_descriptor(slice_type, classfile)));
				}
			},
			_ => panic!("Invalid storable / storable type pair")
		}
	}
}