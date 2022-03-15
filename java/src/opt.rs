pub mod ins {
    use crate::{ClassFile, Constant, Descriptor, Ins, Integer};

	pub fn ldc(idx: usize, classfile: &ClassFile) -> Ins {
		if classfile.constant_pool_index_to_encodable_index(idx) <= 0xff {
			return Ins::Ldc { index: idx };
		}

		return Ins::LdcW { index: idx };
	}

	pub fn iconst(i: i32, classfile: &mut ClassFile) -> Ins {
		match i {
			-1 => Ins::IConstM1,
			0 => Ins::IConst0,
			1 => Ins::IConst1,
			2 => Ins::IConst2,
			3 => Ins::IConst3,
			4 => Ins::IConst4,
			5 => Ins::IConst5,
			_ if i <= std::i16::MAX as i32 && i >= std::i16::MIN as i32 => Ins::SIPush { value: i as i16 },
			_ => {
				let idx = classfile.add_constant(Constant::Integer(Integer::new(i as u32)));
				ldc(idx, classfile)
			}
		}
	}

	pub fn load(idx: usize, desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte => {
				Ins::ILoad { local: idx as u8 }
			},
			Descriptor::Char => {
				Ins::ILoad { local: idx as u8 }
			},
			Descriptor::Double => {
				Ins::DLoad { local: idx as u8 }
			},
			Descriptor::Float => {
				Ins::FLoad { local: idx as u8 }
			},
			Descriptor::Int => {
				Ins::ILoad { local: idx as u8 }
			},
			Descriptor::Long => {
				Ins::LLoad { local: idx as u8 }
			},
			Descriptor::Reference(_) => {
				Ins::ALoad { local: idx as u8 }
			},
			Descriptor::Short => {
				Ins::ILoad { local: idx as u8 }
			},
			Descriptor::Boolean => {
				Ins::ILoad { local: idx as u8 }
			},
			Descriptor::Array(_, _) => {
				Ins::ALoad { local: idx as u8 }
			},
			Descriptor::Void => panic!("Cannot load void"),
		}
	}

	pub fn aload(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte => {
				Ins::BALoad
			},
			Descriptor::Char => {
				Ins::CALoad
			},
			Descriptor::Double => {
				Ins::DALoad
			},
			Descriptor::Float => {
				Ins::FALoad
			},
			Descriptor::Int => {
				Ins::IALoad
			},
			Descriptor::Long => {
				Ins::LALoad
			},
			Descriptor::Reference(_) => {
				Ins::AALoad
			},
			Descriptor::Short => {
				Ins::SALoad
			},
			Descriptor::Boolean => {
				Ins::BALoad
			},
			Descriptor::Array(_, _) => {
				Ins::AALoad
			},
			Descriptor::Void => panic!("Cannot aload void"),
		}
	}

	pub fn store(idx: usize, desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte => {
				Ins::IStore { local: idx as u8 }
			},
			Descriptor::Char => {
				Ins::IStore { local: idx as u8 }
			},
			Descriptor::Double => {
				Ins::DStore { local: idx as u8 }
			},
			Descriptor::Float => {
				Ins::FStore { local: idx as u8 }
			},
			Descriptor::Int => {
				Ins::IStore { local: idx as u8 }
			},
			Descriptor::Long => {
				Ins::LStore { local: idx as u8 }
			},
			Descriptor::Reference(_) => {
				Ins::AStore { local: idx as u8 }
			},
			Descriptor::Short => {
				Ins::IStore { local: idx as u8 }
			},
			Descriptor::Boolean => {
				Ins::IStore { local: idx as u8 }
			},
			Descriptor::Array(_, _) => {
				Ins::AStore { local: idx as u8 }
			},
			Descriptor::Void => panic!("Cannot store void"),
		}
	}

	pub fn astore(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte => {
				Ins::BAStore
			},
			Descriptor::Char => {
				Ins::CAStore
			},
			Descriptor::Double => {
				Ins::DAStore
			},
			Descriptor::Float => {
				Ins::FAStore
			},
			Descriptor::Int => {
				Ins::IAStore
			},
			Descriptor::Long => {
				Ins::LAStore
			},
			Descriptor::Reference(_) => {
				Ins::AAStore
			},
			Descriptor::Short => {
				Ins::SAStore
			},
			Descriptor::Boolean => {
				Ins::BAStore
			},
			Descriptor::Array(_, _) => {
				Ins::AAStore
			},
			Descriptor::Void => panic!("Cannot astore void"),
		}
	}

	pub fn conv(from: &Descriptor, to: &Descriptor) -> Option<Ins> {
		match from {
			Descriptor::Byte | Descriptor::Boolean =>
				match to {
					Descriptor::Byte | Descriptor::Boolean => None,
					Descriptor::Char => Some(Ins::I2C),
					Descriptor::Float => Some(Ins::I2F),
					Descriptor::Double => Some(Ins::I2D),
					Descriptor::Short => Some(Ins::I2S),
					Descriptor::Long => Some(Ins::I2L),
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Int => None,
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Char =>
				match to {
					Descriptor::Byte | Descriptor::Boolean => Some(Ins::I2B),
					Descriptor::Char => None,
					Descriptor::Float => Some(Ins::I2F),
					Descriptor::Double => Some(Ins::I2D),
					Descriptor::Short => Some(Ins::I2S),
					Descriptor::Long => Some(Ins::I2L),
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Int => None,
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Double =>
				match to {
					Descriptor::Byte | Descriptor::Boolean | Descriptor::Char | Descriptor::Int | Descriptor::Short => Some(Ins::D2I),
					Descriptor::Float => Some(Ins::D2F),
					Descriptor::Double => None,
					Descriptor::Long => Some(Ins::D2L),
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Float =>
				match to {
					Descriptor::Byte | Descriptor::Boolean | Descriptor::Char | Descriptor::Int | Descriptor::Short => Some(Ins::F2I),
					Descriptor::Float => None,
					Descriptor::Double => Some(Ins::F2D),
					Descriptor::Long => Some(Ins::F2L),
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Int =>
				match to {
					Descriptor::Byte | Descriptor::Boolean | Descriptor::Char | Descriptor::Int | Descriptor::Short => None,
					Descriptor::Float => Some(Ins::I2F),
					Descriptor::Double => Some(Ins::I2D),
					Descriptor::Long => Some(Ins::I2L),
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Long =>
				match to {
					Descriptor::Byte | Descriptor::Boolean | Descriptor::Char | Descriptor::Int | Descriptor::Short => Some(Ins::L2I),
					Descriptor::Float => Some(Ins::L2F),
					Descriptor::Double => Some(Ins::L2D),
					Descriptor::Long => None,
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Reference(_) => panic!("Cannot convert from reference"),
			Descriptor::Short => 
				match to {
					Descriptor::Byte | Descriptor::Boolean | Descriptor::Char | Descriptor::Int => Some(Ins::I2S),
					Descriptor::Short => None,
					Descriptor::Float => Some(Ins::I2F),
					Descriptor::Double => Some(Ins::I2D),
					Descriptor::Long => Some(Ins::I2L),
					Descriptor::Array(_, _) => panic!("Cannot convert to array"),
					Descriptor::Reference(_) => panic!("Cannot convert to reference"),
					Descriptor::Void => panic!("Cannot convert to void"),
				},
			Descriptor::Array(_, _) => panic!("Cannot convert from array"),
			Descriptor::Void => panic!("Cannot convert from void"),
		}
	}

	pub fn ret(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte | Descriptor::Char | Descriptor::Int | Descriptor::Short | Descriptor::Boolean => Ins::IReturn,
			Descriptor::Double => Ins::DReturn,
			Descriptor::Float => Ins::FReturn,
			Descriptor::Long => Ins::LReturn,
			Descriptor::Reference(_) | Descriptor::Array(_, _) => Ins::AReturn,
			Descriptor::Void => Ins::Return,
		}
	}

	pub fn add(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte | Descriptor::Char | Descriptor::Int | Descriptor::Short | Descriptor::Boolean => Ins::IAdd,
			Descriptor::Double => Ins::DAdd,
			Descriptor::Float => Ins::FAdd,
			Descriptor::Long => Ins::LAdd,
			Descriptor::Reference(_) | Descriptor::Array(_, _) => panic!("Cannot add references"),
			Descriptor::Void => panic!("Cannot add void"),
		}
	}

	pub fn mul(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte | Descriptor::Char | Descriptor::Int | Descriptor::Short | Descriptor::Boolean => Ins::IMul,
			Descriptor::Double => Ins::DMul,
			Descriptor::Float => Ins::FMul,
			Descriptor::Long => Ins::LMul,
			Descriptor::Reference(_) | Descriptor::Array(_, _) => panic!("Cannot mul references"),
			Descriptor::Void => panic!("Cannot mul void"),
		}
	}

	pub fn sub(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte | Descriptor::Char | Descriptor::Int | Descriptor::Short | Descriptor::Boolean => Ins::ISub,
			Descriptor::Double => Ins::DSub,
			Descriptor::Float => Ins::FSub,
			Descriptor::Long => Ins::LSub,
			Descriptor::Reference(_) | Descriptor::Array(_, _) => panic!("Cannot sub references"),
			Descriptor::Void => panic!("Cannot sub void"),
		}
	}

	pub fn div(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte | Descriptor::Char | Descriptor::Int | Descriptor::Short | Descriptor::Boolean => Ins::IDiv,
			Descriptor::Double => Ins::DDiv,
			Descriptor::Float => Ins::FDiv,
			Descriptor::Long => Ins::LDiv,
			Descriptor::Reference(_) | Descriptor::Array(_, _) => panic!("Cannot div references"),
			Descriptor::Void => panic!("Cannot div void"),
		}
	}

	pub fn neg(desc: &Descriptor) -> Ins {
		match desc {
			Descriptor::Byte | Descriptor::Char | Descriptor::Int | Descriptor::Short | Descriptor::Boolean => Ins::INeg,
			Descriptor::Double => Ins::DNeg,
			Descriptor::Float => Ins::FNeg,
			Descriptor::Long => Ins::LNeg,
			Descriptor::Reference(_) | Descriptor::Array(_, _) => panic!("Cannot neg references"),
			Descriptor::Void => panic!("Cannot neg void"),
		}
	}
}