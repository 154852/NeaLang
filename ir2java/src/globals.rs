use java::ClassFile;

use crate::{TranslationContext, storable_type_to_jtype};

impl<'a> TranslationContext<'a> {
	pub fn translate_storable(&self, storable: &ir::Storable, storable_type: &ir::StorableType, classfile: &mut ClassFile, insns: &mut Vec<java::Ins>) {
		match (storable, storable_type) {
			(ir::Storable::Compound(compound_value), ir::StorableType::Compound(compound_type)) => {
				let name = format!("{}${}", classfile.name(), compound_type.name());

				insns.push(java::Ins::New { index: classfile.const_class(&name) });
				insns.push(java::Ins::Dup);
				let method = classfile.const_method(&name, "<init>", "()V");
				insns.push(java::Ins::InvokeSpecial { index: method });

				match (compound_value, compound_type.content()) {
					(ir::Compound::Struct(struct_value), ir::TypeContent::Struct(struct_type)) => {
						for (property_value, property_type) in struct_value.props().iter().zip(struct_type.props()) {
							insns.push(java::Ins::Dup);
							self.translate_storable(property_value.value(), property_type.prop_type(), classfile, insns);
							insns.push(java::Ins::PutField {
								index: classfile.const_field(&name, property_type.name(), &storable_type_to_jtype(property_type.prop_type(), &classfile).to_string())
							});
						}
					},
				}
			},
			(ir::Storable::Value(value), ir::StorableType::Value(_)) => {
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
						let name = match global.name() {
							Some(x) => x.to_string(),
							None => format!("_global${}", global_index)
						};
						insns.push(java::Ins::GetStatic {
							index: classfile.const_field(&classfile.name().to_string(), &name, &storable_type_to_jtype(global.global_type(), &classfile).to_string())
						});
					},
				}
			},
			(ir::Storable::Slice(owned_index, index, length), ir::StorableType::Slice(_)) => {
				let owned = self.unit().get_global(*owned_index).unwrap();

				match (owned.global_type(), owned.default()) {
					(ir::StorableType::SliceData(_), Some(ir::Storable::SliceData(data))) => {
						assert!(*index == 0 && *length == data.len());
					},
					_ => panic!("Invalid slice reference")
				}

				let name = match owned.name() {
					Some(x) => x.to_string(),
					None => format!("_global${}", *owned_index)
				};
				insns.push(java::Ins::GetStatic {
					index: classfile.const_field(&classfile.name().to_string(), &name, &storable_type_to_jtype(owned.global_type(), &classfile).to_string())
				});
			},
			(ir::Storable::SliceData(elements), ir::StorableType::SliceData(slice_type)) => {
				insns.push(java::Ins::SIPush { value: elements.len() as i16 });

				let descriptor = storable_type_to_jtype(slice_type, &classfile);
				if let java::Descriptor::Reference(name) = descriptor {
					insns.push(java::Ins::ANewArray { index: classfile.const_class(&name) });
				} else {
					insns.push(java::Ins::NewArray { atype: descriptor });
				}

				for (e, element) in elements.iter().enumerate() {
					insns.push(java::Ins::Dup);
					insns.push(java::Ins::SIPush { value: e as i16 });
					self.translate_storable(element, slice_type, classfile, insns);
					
					match storable_type_to_jtype(slice_type, &classfile) {
						java::Descriptor::Byte => {
							insns.push(java::Ins::BAStore);
						},
						java::Descriptor::Char => {
							insns.push(java::Ins::CAStore);
						},
						java::Descriptor::Double => {
							insns.push(java::Ins::DAStore);
						},
						java::Descriptor::Float => {
							insns.push(java::Ins::FAStore);
						},
						java::Descriptor::Int => {
							insns.push(java::Ins::IAStore);
						},
						java::Descriptor::Long => {
							insns.push(java::Ins::LAStore);
						},
						java::Descriptor::Reference(_) => {
							insns.push(java::Ins::AAStore);
						},
						java::Descriptor::Short => {
							insns.push(java::Ins::SAStore);
						},
						java::Descriptor::Boolean => {
							insns.push(java::Ins::BAStore);
						},
						java::Descriptor::Array(_, _) => {
							insns.push(java::Ins::AAStore);
						},
						java::Descriptor::Void => panic!(),
					}
				}
			},
			_ => panic!("Invalid storable / storable type pair")
		}
	}
}