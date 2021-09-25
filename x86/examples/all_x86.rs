use std::collections::HashMap;
use x86;

fn main() {
    let mut raw = Vec::new();

    let ins: Vec<x86::Ins> = vec![
        x86::Ins::AddRegReg(x86::Reg::Rax, x86::Reg::R9),
        x86::Ins::AddRegMem(x86::Reg::Edx, x86::Mem::new().base(x86::RegClass::Ecx)),
        x86::Ins::AddMemReg(x86::Mem::new().base(x86::RegClass::R8).disp(0x10), x86::Reg::R15B),
        x86::Ins::AddRegImm(x86::Reg::R15D, 0x64),
        x86::Ins::AddMemImm(x86::Size::Quad, x86::Mem::new().base(x86::RegClass::R13).disp(0x10), 0x64),

        x86::Ins::MovRegReg(x86::Reg::Rax, x86::Reg::R9),
        x86::Ins::MovRegMem(x86::Reg::Edx, x86::Mem::new().base(x86::RegClass::Ecx)),
        x86::Ins::MovMemReg(x86::Mem::new().base(x86::RegClass::R8).disp(0x10), x86::Reg::R15B),
        x86::Ins::MovRegImm(x86::Reg::R15D, 0x64),
        x86::Ins::MovMemImm(x86::Size::Quad, x86::Mem::new().base(x86::RegClass::R13).disp(0x10), 0x64),

        x86::Ins::PushReg(x86::Reg::R8),
        x86::Ins::PushMem(x86::Mem::new().base(x86::RegClass::R8).disp(0x10)),
        x86::Ins::PushImm(0x12),

        x86::Ins::Ret,

        x86::Ins::SubRegReg(x86::Reg::Rax, x86::Reg::R9),
        x86::Ins::SubRegMem(x86::Reg::Edx, x86::Mem::new().base(x86::RegClass::Ecx)),
        x86::Ins::SubMemReg(x86::Mem::new().base(x86::RegClass::R8).disp(0x10), x86::Reg::R15B),
        x86::Ins::SubRegImm(x86::Reg::R15D, 0x64),
        x86::Ins::SubMemImm(x86::Size::Quad, x86::Mem::new().base(x86::RegClass::R13).disp(0x10), 0x64),
    ];

    let mut local_symbols = HashMap::new();
    let mut unfilled_local_symbols = Vec::new();

    for i in ins { i.encode(&mut raw, &mut local_symbols, &mut unfilled_local_symbols); }

    x86::Relocation::fill(&mut raw, &local_symbols, &unfilled_local_symbols);

    // View with `objdump -D x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("x86/examples/binary.bin", &raw).expect("Could not write output");
}