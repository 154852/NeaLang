use x86;

fn main() {
	let mut raw = Vec::new();

	let ins: Vec<x86::Ins> = vec![
		x86::Ins::MovRegReg(x86::Reg::Rax, x86::Reg::R9),
		x86::Ins::MovRegMem(x86::Reg::Edx, x86::Mem::new().base(x86::RegClass::Ecx)),
		x86::Ins::MovMemReg(x86::Mem::new().base(x86::RegClass::R8).disp(0x10), x86::Reg::R15B),
		x86::Ins::MovRegImm(x86::Reg::R15D, 0x64),
	];

	for i in ins { i.encode(&mut raw); }

	// View with `objdump -D x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
	std::fs::write("x86/examples/binary.bin", &raw).expect("Could not write output");
}