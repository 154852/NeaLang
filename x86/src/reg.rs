use crate::{Mode, Size};

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RegClass {
    Eax, Ebx, Ecx, Edx,
    Ebp, Esp, Eip, Esi, Edi,
    R8, R9, R10, R11, R12, R13, R14, R15
}

impl RegClass {
    pub fn id(&self) -> u8 {
        use RegClass::*;

        match self {
            Eax => 0,
            Ebx => 3,
            Ecx => 1,
            Edx => 2,
            Ebp => 5,
            Esp => 4,
            Eip => panic!(),
            Esi => 6,
            Edi => 7,
            R8 => 0,
            R9 => 1,
            R10 => 2,
            R11 => 3,
            R12 => 4,
            R13 => 5,
            R14 => 6,
            R15 => 7
        }
    }

    pub fn u8(&self) -> Reg {
        use RegClass::*;

        match self {
            Eax => Reg::Al,
            Ebx => Reg::Bl,
            Ecx => Reg::Cl,
            Edx => Reg::Dl,
            Ebp => Reg::Bpl,
            Esp => Reg::Spl,
            Eip => Reg::Ipl,
            Esi => Reg::Sil,
            Edi => Reg::Dil,
            R8 => Reg::R8B,
            R9 => Reg::R9B,
            R10 => Reg::R10B,
            R11 => Reg::R11B,
            R12 => Reg::R12B,
            R13 => Reg::R13B,
            R14 => Reg::R14B,
            R15 => Reg::R15B
        }
    }

    pub fn u16(&self) -> Reg {
        use RegClass::*;

        match self {
            Eax => Reg::Ax,
            Ebx => Reg::Bx,
            Ecx => Reg::Cx,
            Edx => Reg::Dx,
            Ebp => Reg::Bp,
            Esp => Reg::Sp,
            Eip => Reg::Ip,
            Esi => Reg::Si,
            Edi => Reg::Di,
            R8 => Reg::R8W,
            R9 => Reg::R9W,
            R10 => Reg::R10W,
            R11 => Reg::R11W,
            R12 => Reg::R12W,
            R13 => Reg::R13W,
            R14 => Reg::R14W,
            R15 => Reg::R15W
        }
    }

    pub fn u32(&self) -> Reg {
        use RegClass::*;

        match self {
            Eax => Reg::Eax,
            Ebx => Reg::Ebx,
            Ecx => Reg::Ecx,
            Edx => Reg::Edx,
            Ebp => Reg::Ebp,
            Esp => Reg::Esp,
            Eip => Reg::Eip,
            Esi => Reg::Esi,
            Edi => Reg::Edi,
            R8 => Reg::R8D,
            R9 => Reg::R9D,
            R10 => Reg::R10D,
            R11 => Reg::R11D,
            R12 => Reg::R12D,
            R13 => Reg::R13D,
            R14 => Reg::R14D,
            R15 => Reg::R15D
        }
    }

    pub fn u64(&self) -> Reg {
        use RegClass::*;

        match self {
            Eax => Reg::Rax,
            Ebx => Reg::Rbx,
            Ecx => Reg::Rcx,
            Edx => Reg::Rdx,
            Ebp => Reg::Rbp,
            Esp => Reg::Rsp,
            Eip => Reg::Rip,
            Esi => Reg::Rsi,
            Edi => Reg::Rdi,
            R8 => Reg::R8,
            R9 => Reg::R9,
            R10 => Reg::R10,
            R11 => Reg::R11,
            R12 => Reg::R12,
            R13 => Reg::R13,
            R14 => Reg::R14,
            R15 => Reg::R15
        }
    }

    pub fn is_rn(&self) -> bool {
        use RegClass::*;

        match self {
            R8 | R9 | R10 | R11 | R12 | R13 | R14 | R15 => true,
            _ => false
        }
    }

    pub fn uptr(&self, mode: &Mode) -> Reg {
        match mode {
            Mode::X86 => self.u32(),
            Mode::X8664 => self.u64(),
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Reg {
    Rax, Eax, Ax, Al, Ah,
    Rbx, Ebx, Bx, Bl, Bh,
    Rcx, Ecx, Cx, Cl, Ch,
    Rdx, Edx, Dx, Dl, Dh,
    Rbp, Ebp, Bp, Bpl,
    Rsp, Esp, Sp, Spl,
    Rip, Eip, Ip, Ipl,
    Rsi, Esi, Si, Sil,
    Rdi, Edi, Di, Dil,
    R8, R8D, R8W, R8B,
    R9, R9D, R9W, R9B,
    R10, R10D, R10W, R10B,
    R11, R11D, R11W, R11B,
    R12, R12D, R12W, R12B,
    R13, R13D, R13W, R13B,
    R14, R14D, R14W, R14B,
    R15, R15D, R15W, R15B
}

impl Reg {
    pub fn size(&self) -> Size {
        use Reg::*;

        match self {
            Rax | Rbx | Rcx | Rdx | Rbp | Rsp | Rip | Rsi | Rdi | R8 | R9 | R10 | R11 | R12 | R13 | R14 | R15 => Size::Quad,
            Eax | Ebx | Ecx | Edx | Ebp | Esp | Eip | Esi | Edi | R8D | R9D | R10D | R11D | R12D | R13D | R14D | R15D=> Size::Double,
            Ax | Bx | Cx | Dx | Bp | Sp | Ip | Si | Di | R8W | R9W | R10W | R11W | R12W | R13W | R14W | R15W => Size::Word,
            _ => Size::Byte
        }
    }

    pub fn class(&self) -> RegClass {
        use Reg::*;

        match self {
            Rax | Eax | Ax | Al | Ah => RegClass::Eax,
            Rbx | Ebx | Bx | Bl | Bh => RegClass::Ebx,
            Rcx | Ecx | Cx | Cl | Ch => RegClass::Ecx,
            Rdx | Edx | Dx | Dl | Dh => RegClass::Edx,
            Rbp | Ebp | Bp | Bpl => RegClass::Ebp,
            Rsp | Esp | Sp | Spl => RegClass::Esp,
            Rip | Eip | Ip | Ipl => RegClass::Eip,
            Rsi | Esi | Si | Sil => RegClass::Esi,
            Rdi | Edi | Di | Dil => RegClass::Edi,
            R8 | R8D | R8W | R8B => RegClass::R8,
            R9 | R9D | R9W | R9B => RegClass::R9,
            R10 | R10D | R10W | R10B => RegClass::R10,
            R11 | R11D | R11W | R11B => RegClass::R11,
            R12 | R12D | R12W | R12B => RegClass::R12,
            R13 | R13D | R13W | R13B => RegClass::R13,
            R14 | R14D | R14W | R14B => RegClass::R14,
            R15 | R15D | R15W | R15B => RegClass::R15
        }
    }

    pub fn modrm_reg_addressing(a: Reg, b: Reg) -> u8 {
        0b11000000 | (a.class().id() << 3) | b.class().id()
    }

    pub fn modrm_reg_addressing_single(&self, with: u8) -> u8 {
        0b11000000 | (with << 3) | self.class().id()
    }
}