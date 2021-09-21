use crate::{Encoder, Mem, Reg, Size};

pub enum Ins {
	/// A <- B
	MovRegReg(Reg, Reg),

	/// A <- B
	MovRegMem(Reg, Mem),

	/// A <- B
	MovMemReg(Mem, Reg),

	/// A <- B
	MovRegImm(Reg, u64),

	/// A <- B
	MovMemImm(Size, Mem, u64),
}

impl Ins {
	pub fn encode(&self, data: &mut Vec<u8>) {
		match *self {
			// https://www.felixcloutier.com/x86/mov
			Ins::MovRegReg(a, b) => Encoder::new(if a.size() == Size::Byte { 0x88 } else { 0x89 }).rr(b, a).to(data),
			Ins::MovRegMem(r, ref m) => Encoder::new(if r.size() == Size::Byte { 0x8a } else { 0x8b }).rm(r, m).to(data),
			Ins::MovMemReg(ref m, r) => Encoder::new(if r.size() == Size::Byte { 0x88 } else { 0x89 }).rm(r, m).to(data),
			Ins::MovRegImm(r, i) => Encoder::new(if r.size() == Size::Byte { 0xb0 } else { 0xb8 }).offset(r).immnq(i, r.size()).to(data),
			Ins::MovMemImm(s, ref m, i) => Encoder::new(if s == Size::Byte { 0xc6 } else { 0xc7 }).mn(s, m, 0).immn(i as u32, s).to(data),
		}
	}
}