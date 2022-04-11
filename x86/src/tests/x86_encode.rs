use std::collections::HashMap;

use crate::*;

#[test]
fn instruction_opcodes() {
    let mut local_symbols = HashMap::new();
    let mut unfilled_local_symbols = Vec::new();
    
    let insns = vec![
        Ins::AddRegReg(Reg::Rax, Reg::Rcx),
        Ins::AddMemReg(Mem::new().base(RegClass::Edi), Reg::Rcx),

        Ins::AndRegReg(Reg::Rax, Reg::Rcx),
        Ins::AndMemReg(Mem::new().base(RegClass::Edi), Reg::Rcx),

        Ins::Cdq(Size::Double),

        Ins::CMovRegReg(Condition::Zero, Reg::Rax, Reg::Rcx),
        Ins::CMovRegMem(Condition::Less, Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::CmpRegReg(Reg::Rax, Reg::Rcx),
        Ins::CmpRegMem(Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::ConditionalSet(Condition::GreaterOrEqual, RegClass::Eax),

        Ins::DivReg(Reg::Rcx),
        Ins::DivMem(Size::Quad, Mem::new().base(RegClass::Edi)),

        Ins::IDivReg(Reg::Rcx),
        Ins::IDivMem(Size::Quad, Mem::new().base(RegClass::Edi)),

        Ins::IMulRegReg(Reg::Rax, Reg::Rcx),
        Ins::IMulRegMem(Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::LeaRegMem(Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::MovRegReg(Reg::Rax, Reg::Rcx),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::Edi)),
        Ins::MovMemReg(Mem::new().base(RegClass::Eax), Reg::Rdi),
        Ins::MovRegImm(Reg::Rax, 42),
        Ins::MovMemImm(Size::Quad, Mem::new().base(RegClass::Edi), 42),

        Ins::MovsxRegReg(Reg::Rax, Reg::Ecx),
        Ins::MovsxRegMem(Size::Double, Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::MovzxRegReg(Reg::Eax, Reg::Cx),
        Ins::MovzxRegMem(Size::Word, Reg::Eax, Mem::new().base(RegClass::Edi)),

        Ins::NegReg(Reg::Rax),
        Ins::NegMem(Size::Quad, Mem::new().base(RegClass::Eax)),

        Ins::OrRegReg(Reg::Rax, Reg::Rcx),
        Ins::OrRegMem(Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::PopReg(Reg::Rax),
        Ins::PopMem(Mem::new().base(RegClass::Edi)),

        Ins::PushReg(Reg::Rax),
        Ins::PushMem(Mem::new().base(RegClass::Edi)),
        Ins::PushImm(32),

        Ins::Ret,

        Ins::SubRegReg(Reg::Rax, Reg::Rcx),
        Ins::SubRegMem(Reg::Rax, Mem::new().base(RegClass::Edi)),

        Ins::TestRegReg(Reg::Rax, Reg::Rcx),
        Ins::TestMemReg(Mem::new().base(RegClass::Edi), Reg::Rax),

        Ins::AddRegImm(Reg::Rax, 3),
        Ins::AddRegImm(Reg::Rax, 356),
        Ins::AddRegImm(Reg::Rcx, 7),
        Ins::AddRegImm(Reg::Rcx, 456),
        Ins::AddMemImm(Size::Quad, Mem::new().base(RegClass::Ecx), 34),
    ];

    let mut data = Vec::new();
    for ins in insns {
        ins.encode(&mut data, &mut local_symbols, &mut unfilled_local_symbols);
    }

    assert_eq!(data, std::fs::read("src/tests/instruction_opcodes.bin").expect("Could not read instruction_opcodes.bin"));
}

#[test]
fn registers() {
    let mut local_symbols = HashMap::new();
    let mut unfilled_local_symbols = Vec::new();

    let insns = vec![
        Ins::MovRegReg(Reg::Rax, Reg::Rcx),
        Ins::MovRegReg(Reg::Rax, Reg::Rdx),
        Ins::MovRegReg(Reg::Rax, Reg::Rbx),
        Ins::MovRegReg(Reg::Rax, Reg::Rdi),
        Ins::MovRegReg(Reg::Rax, Reg::Rsi),
        Ins::MovRegReg(Reg::Rax, Reg::Rbp),
        Ins::MovRegReg(Reg::Rax, Reg::Rsp),
        Ins::MovRegReg(Reg::Rax, Reg::R8),
        Ins::MovRegReg(Reg::Rax, Reg::R9),
        Ins::MovRegReg(Reg::Rax, Reg::R10),
        Ins::MovRegReg(Reg::Rax, Reg::R11),
        Ins::MovRegReg(Reg::Rax, Reg::R12),
        Ins::MovRegReg(Reg::Rax, Reg::R13),
        Ins::MovRegReg(Reg::Rax, Reg::R14),
        Ins::MovRegReg(Reg::Rax, Reg::R15),

        Ins::MovRegReg(Reg::R8, Reg::Rax),
        Ins::MovRegReg(Reg::R9, Reg::Rax),
        Ins::MovRegReg(Reg::R10, Reg::Rax),
        Ins::MovRegReg(Reg::R11, Reg::Rax),
        Ins::MovRegReg(Reg::R12, Reg::Rax),
        Ins::MovRegReg(Reg::R13, Reg::Rax),
        Ins::MovRegReg(Reg::R14, Reg::Rax),
        Ins::MovRegReg(Reg::R15, Reg::Rax),

        Ins::MovRegReg(Reg::R8, Reg::R8),
        Ins::MovRegReg(Reg::R8, Reg::R9),
        Ins::MovRegReg(Reg::R8, Reg::R10),
        Ins::MovRegReg(Reg::R8, Reg::R11),
        Ins::MovRegReg(Reg::R8, Reg::R12),
        Ins::MovRegReg(Reg::R8, Reg::R13),
        Ins::MovRegReg(Reg::R8, Reg::R14),
        Ins::MovRegReg(Reg::R8, Reg::R15),

        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::R8)),
        Ins::MovRegMem(Reg::R8, Mem::new().base(RegClass::Eax)),
        Ins::MovMemReg(Mem::new().base(RegClass::R8), Reg::Rax),
        Ins::MovMemReg(Mem::new().base(RegClass::Eax), Reg::R8),

        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::R8).index(RegClass::Ebx)),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::Ebx).index(RegClass::R8)),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::R9).index(RegClass::R8)),
        Ins::MovRegMem(Reg::R10, Mem::new().base(RegClass::R9).index(RegClass::R8)),
    ];

    let mut data = Vec::new();
    for ins in insns {
        ins.encode(&mut data, &mut local_symbols, &mut unfilled_local_symbols);
    }

    assert_eq!(data, std::fs::read("src/tests/registers.bin").expect("Could not read registers.bin"));
}

#[test]
fn memory() {
    let mut local_symbols = HashMap::new();
    let mut unfilled_local_symbols = Vec::new();

    let insns = vec![
        Ins::MovRegMem(Reg::Rax, Mem::new().disp(123)),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::Ecx).disp(2)),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::Ecx).index(RegClass::Edx)),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::Ecx).index(RegClass::Edx).scale(2 /* 2<<2 = 4 */)),
        Ins::MovRegMem(Reg::Rax, Mem::new().base(RegClass::Ecx).index(RegClass::Edx).scale(2 /* 2<<2 = 4 */).disp(10)),
    ];

    let mut data = Vec::new();
    for ins in insns {
        ins.encode(&mut data, &mut local_symbols, &mut unfilled_local_symbols);
    }

    assert_eq!(data, std::fs::read("src/tests/memory.bin").expect("Could not read memory.bin"));
}

#[test]
fn relocations() {
    let mut ctx = EncodeContext::new();

    let a = LocalSymbolID::new(0);
    ctx.append_function(&vec![
        Ins::JumpLocalSymbol(a),
        Ins::AddRegImm(Reg::Rax, 3),
        Ins::LocalSymbol(a),
        Ins::AddRegImm(Reg::Rcx, 4),
        Ins::JumpConditionalLocalSymbol(Condition::NotZero, a),
    ]);

    let (data, relocs) = ctx.take();

    assert_eq!(relocs.len(), 0);
    // Note that we don't use nasm here because it does this more efficiently and provides alternative but shorter assembly using 8 bit offsets instead of the full 32 I use
    // We can check the below hex is the correct assembly by feeding it to a disassembler:
    // echo e9040000004883c0034883c1040f85f6ffffff | xxd -r -p | ndisasm -b 64 -
    // For comparison, nasm produces
    // echo eb044883c0034883c10475fa | xxd -r -p | ndisasm -b 64 -
    // which while shorter, is functionally identical
    assert_eq!(data, &[
        0xe9, 0x04, 0x00, 0x00, 0x00, 0x48, 0x83, 0xc0, 0x03, 0x48, 0x83, 0xc1, 0x04, 0x0f, 0x85, 0xf6, 0xff, 0xff, 0xff
    ]);
}