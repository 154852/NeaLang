use crate::Ins;

pub fn pass_zero(insns: &mut Vec<Ins>) {
	remove_zero_jumps(insns);
	remove_redundant_movs(insns);
}

pub fn remove_zero_jumps(insns: &mut Vec<Ins>) {
	let mut i = 0;
	while i < insns.len() {
		let ins = &insns[i];
		match ins {
			Ins::JumpLocalSymbol(jump_target) => {
				if matches!(insns.get(i + 1), Some(Ins::LocalSymbol(symbol)) if symbol == jump_target) {
					insns.remove(i);
				} else {
					i += 1;
				}
			},
			_ => {
				i += 1;
			}
		}
	}
}

pub fn remove_redundant_movs(insns: &mut Vec<Ins>) {
	let mut i = 0;
	while i < insns.len() {
		let ins = &insns[i];
		match ins {
			Ins::MovRegReg(a, b) => {
				if a == b {
					insns.remove(i);
				} else {
					i += 1;
				}
			},
			_ => {
				i += 1;
			}
		}
	}
}