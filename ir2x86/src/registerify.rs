use ir::PropertyIndex;
use x86;

pub(crate) const SYS_V_ABI: &[x86::RegClass] = &[
	x86::RegClass::Edi,
	x86::RegClass::Esi,
	x86::RegClass::Edx,
	x86::RegClass::Ecx,
	x86::RegClass::R8,
	x86::RegClass::R9,

	// TODO: Handle items on the stack

	x86::RegClass::R10,
	x86::RegClass::R11,
	x86::RegClass::R12,
	x86::RegClass::R13,
	x86::RegClass::R14,
	x86::RegClass::R15,
	x86::RegClass::Ebx,
	x86::RegClass::Eax,
];

pub(crate) const SYS_V_ABI_RET: &[x86::RegClass] = &[
	x86::RegClass::Eax,
	x86::RegClass::Edx,
	x86::RegClass::Ebx,

	x86::RegClass::R15,
	x86::RegClass::R14,
	x86::RegClass::R13,
	x86::RegClass::R12,
	x86::RegClass::R11,
	x86::RegClass::R10,
];

pub(crate) fn reg_for_value_type(vt: &ir::ValueType, mode: x86::Mode, class: x86::RegClass) -> x86::Reg {
	match vt {
		ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool => class.u8(),
		ir::ValueType::U16 | ir::ValueType::I16 => class.u16(),
		ir::ValueType::U32 | ir::ValueType::I32 => class.u32(),
		ir::ValueType::U64 | ir::ValueType::I64 => class.u64(),
		ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => match mode {
			x86::Mode::X86 => class.u32(),
			x86::Mode::X8664 => class.u64(),
		},
	}
}

pub(crate) fn offset_of_prop(ct: &ir::CompoundTypeRef, idx: PropertyIndex, mode: x86::Mode) -> usize {
	match ct.content() {
		ir::TypeContent::Struct(s) => {
			let mut offset = 0;

			for i in 0..idx {
				offset += size_for_storable_type(s.prop(i).expect("Invalid property ID").prop_type(), mode);
			}

			offset
		},
	}
}

pub(crate) fn size_for_compound_type(ct: &ir::CompoundType, mode: x86::Mode) -> usize {
	match ct.content() {
		ir::TypeContent::Struct(s) => {
			let mut size = 0;

			for s in s.props() {
				size += size_for_storable_type(s.prop_type(), mode);
			}

			size
		},
	}
}

pub(crate) fn size_for_value_type(vt: &ir::ValueType, mode: x86::Mode) -> usize {
	match vt {
		ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool => 1,
		ir::ValueType::U16 | ir::ValueType::I16 => 2,
		ir::ValueType::U32 | ir::ValueType::I32 => 4,
		ir::ValueType::U64 | ir::ValueType::I64 => 8,
		ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) | ir::ValueType::Index(_) => mode.ptr_size(),
	}
}

pub(crate) fn size_for_storable_type(st: &ir::StorableType, mode: x86::Mode) -> usize {
	match st {
		ir::StorableType::Compound(ct) => size_for_compound_type(ct, mode),
		ir::StorableType::Value(vt) => size_for_value_type(vt, mode),
		ir::StorableType::Slice(_) => mode.ptr_size() * 2,
		ir::StorableType::SliceData(_) => panic!("Cannot compute raw size of SliceData type"),
	}
}

pub(crate) struct StackToReg {
	idx: usize,
	mode: x86::Mode
}

impl StackToReg {
	pub fn new(mode: x86::Mode) -> StackToReg {
		StackToReg {
			idx: 0,
			mode
		}
	}

	pub fn push(&mut self) -> x86::RegClass {
		self.idx += 1;
		
		SYS_V_ABI_RET[self.idx - 1]
	}

	pub fn pop(&mut self) -> x86::RegClass {
		self.idx -= 1;
		SYS_V_ABI_RET[self.idx]
	}

	pub fn peek(&self) -> x86::RegClass {
		SYS_V_ABI_RET[self.idx - 1]
	}
	
	pub fn peek_at(&self, off: usize) -> x86::RegClass {
		SYS_V_ABI_RET[self.idx - 1 - off]
	}

	pub(crate) fn at(&self, idx: usize) -> x86::RegClass {
		SYS_V_ABI_RET[idx]
	}

	pub fn push_many(&mut self, count: usize) {
		self.idx += count;
	}

	pub fn pop_many(&mut self, count: usize) {
		self.idx -= count;
	}

	pub fn zero(&mut self) {
		self.idx = 0;
	}

	pub fn size(&self) -> usize {
		self.idx
	}

	pub fn pop_vt(&mut self, vt: &ir::ValueType) -> x86::Reg {
		reg_for_value_type(vt, self.mode, self.pop())
	}

	pub fn push_vt(&mut self, vt: &ir::ValueType) -> x86::Reg {
		reg_for_value_type(vt, self.mode, self.push())
	}

	pub fn push_ptr(&mut self) -> x86::Reg {
		self.push().uptr(&self.mode)
	}

	pub fn pop_ptr(&mut self) -> x86::Reg {
		self.pop().uptr(&self.mode)
	}

	pub fn peek_ptr(&self) -> x86::Reg {
		self.peek().uptr(&self.mode)
	}

	pub fn peek_vt(&self, vt: &ir::ValueType) -> x86::Reg {
		reg_for_value_type(vt, self.mode, self.peek())
	}

	pub fn peek_at_vt(&self, off: usize, vt: &ir::ValueType) -> x86::Reg {
		reg_for_value_type(vt, self.mode, self.peek_at(off))
	}

	pub fn at_vt(&self, off: usize, vt: &ir::ValueType) -> x86::Reg {
		reg_for_value_type(vt, self.mode, self.at(off))
	}
}