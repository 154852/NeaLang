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

pub(crate) fn reg_for_vt(vt: &ir::ValueType, mode: x86::Mode, class: x86::RegClass) -> x86::Reg {
	match vt {
		ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool => class.u8(),
		ir::ValueType::U16 | ir::ValueType::I16 => class.u16(),
		ir::ValueType::U32 | ir::ValueType::I32 => class.u32(),
		ir::ValueType::U64 | ir::ValueType::I64 => class.u64(),
		ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) => match mode {
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
				offset += size_for_st(s.prop(i).expect("Invalid property ID").prop_type(), mode);
			}

			offset
		},
	}
}

pub(crate) fn size_for_compound(ct: &ir::CompoundType, mode: x86::Mode) -> usize {
	match ct.content() {
		ir::TypeContent::Struct(s) => {
			let mut size = 0;

			for s in s.props() {
				size += size_for_st(s.prop_type(), mode);
			}

			size
		},
	}
}

pub(crate) fn size_for_vt(vt: &ir::ValueType, mode: x86::Mode) -> usize {
	match vt {
		ir::ValueType::U8 | ir::ValueType::I8 | ir::ValueType::Bool => 1,
		ir::ValueType::U16 | ir::ValueType::I16 => 2,
		ir::ValueType::U32 | ir::ValueType::I32 => 4,
		ir::ValueType::U64 | ir::ValueType::I64 => 8,
		ir::ValueType::UPtr | ir::ValueType::IPtr | ir::ValueType::Ref(_) => mode.ptr_size(),
	}
}

pub(crate) fn size_for_st(st: &ir::StorableType, mode: x86::Mode) -> usize {
	match st {
		ir::StorableType::Compound(ct) => size_for_compound(ct, mode),
		ir::StorableType::Value(vt) => size_for_vt(vt, mode),
		ir::StorableType::Slice(_) => mode.ptr_size() * 2
	}
}

pub(crate) struct StackToReg {
	idx: usize,
	is_params: bool,
	mode: x86::Mode
}

impl StackToReg {
	pub fn new(mode: x86::Mode) -> StackToReg {
		StackToReg {
			idx: 0,
			is_params: true,
			mode
		}
	}

	pub fn push(&mut self) -> x86::RegClass {
		self.idx += 1;
		
		if self.is_params {
			SYS_V_ABI[self.idx - 1]
		} else {
			SYS_V_ABI_RET[self.idx - 1]
		}
	}

	pub fn push_many(&mut self, count: usize) {
		self.idx += count;
	}

	pub fn pop_many(&mut self, count: usize) {
		self.idx -= count;
		if self.idx == 0 { self.is_params = false; }
	}

	pub fn zero(&mut self) {
		self.idx = 0;
		self.is_params = false;
	}

	pub fn set_no_params(&mut self) {
		self.is_params = false;
	}

	pub fn pop(&mut self) -> x86::RegClass {
		self.idx -= 1;
		let ret = if self.is_params {
			SYS_V_ABI[self.idx]
		} else {
			SYS_V_ABI_RET[self.idx]
		};

		if self.idx == 0 { self.is_params = false; }
		ret
	}

	pub fn peek(&self) -> x86::RegClass {
		if self.is_params { SYS_V_ABI[self.idx - 1] }
		else { SYS_V_ABI_RET[self.idx - 1] }
	}
	
	pub fn peek_at(&self, off: usize) -> x86::RegClass {
		if self.is_params { SYS_V_ABI[self.idx - 1 - off] }
		else { SYS_V_ABI_RET[self.idx - 1 - off] }
	}

	pub(crate) fn at(&self, idx: usize) -> x86::RegClass {
		if self.is_params { SYS_V_ABI[idx] }
		else { SYS_V_ABI_RET[idx] }
	}

	pub fn size(&self) -> usize {
		self.idx
	}

	pub fn pop_vt(&mut self, vt: &ir::ValueType) -> x86::Reg {
		reg_for_vt(vt, self.mode, self.pop())
	}

	pub fn push_vt(&mut self, vt: &ir::ValueType) -> x86::Reg {
		reg_for_vt(vt, self.mode, self.push())
	}

	pub fn peek_vt(&self, vt: &ir::ValueType) -> x86::Reg {
		reg_for_vt(vt, self.mode, self.peek())
	}

	pub fn peek_at_vt(&self, off: usize, vt: &ir::ValueType) -> x86::Reg {
		reg_for_vt(vt, self.mode, self.peek_at(off))
	}

	pub fn at_vt(&self, off: usize, vt: &ir::ValueType) -> x86::Reg {
		reg_for_vt(vt, self.mode, self.at(off))
	}
}