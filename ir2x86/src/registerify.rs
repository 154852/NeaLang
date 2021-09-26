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

pub(crate) struct StackToReg {
	idx: usize,
	is_params: bool
}

impl StackToReg {
	pub fn new() -> StackToReg {
		StackToReg {
			idx: 0,
			is_params: true
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

	pub fn size(&self) -> usize {
		self.idx
	}
}