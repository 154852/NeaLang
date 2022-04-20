fn main() {
    let insns = vec![
        arm64::Ins::SubImm { size: arm64::SizeFlag::Size64, src: arm64::Reg::sp(), dest: arm64::Reg::sp(), val: 0x10, shift: arm64::ImmShift::Shift0 },
        arm64::Ins::Stp { size: arm64::SizeFlag::Size64, src1: arm64::Reg::fp(), src2: arm64::Reg::lr(), base: arm64::Reg::sp(), offset: 0, mode: arm64::IndexMode::SignedOffset },
        arm64::Ins::Ldp { size: arm64::SizeFlag::Size64, dest1: arm64::Reg::fp(), dest2: arm64::Reg::lr(), base: arm64::Reg::sp(), offset: 0, mode: arm64::IndexMode::SignedOffset },
        arm64::Ins::AddImm { size: arm64::SizeFlag::Size64, src: arm64::Reg::sp(), dest: arm64::Reg::sp(), val: 0x10, shift: arm64::ImmShift::Shift0 },
        arm64::Ins::Ret(arm64::Reg::lr())
    ];

    let mut data = Vec::new();
    for ins in insns {
        data.extend(ins.encode().get().to_le_bytes());
    }

    std::fs::write("arm64/examples/tiny", &data).expect("Could not write");
}