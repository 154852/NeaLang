use std::collections::HashMap;

use crate::Ins;

pub fn pass_zero(insns: &mut Vec<Ins>) {
	collapse_sequential_symbols(insns);
	remove_zero_jumps(insns);
	remove_redundant_movs(insns);
}

pub fn collapse_sequential_symbols(insns: &mut Vec<Ins>) {
	let mut symbol_translation = HashMap::new();

	let mut i = 0;
	while i < insns.len() {
		let ins = &insns[i];
		match ins {
			Ins::LocalSymbol(a) => {
				match insns.get(i + 1) {
					Some(Ins::LocalSymbol(b)) => {
						symbol_translation.insert(*b, *a);
						insns.remove(i + 1);
					},
					_ => {
						i += 1;
					}
				}
			},
			_ => {
				i += 1;
			}
		}
	}

	for ins in insns.iter_mut() {
		match ins {
			Ins::JumpLocalSymbol(jump_target) | Ins::JumpIfZeroLocalSymbol(jump_target) => {
				if let Some(new) = symbol_translation.get(jump_target) {
					*jump_target = *new;
				}
			},
			_ => {}
		}
	}
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