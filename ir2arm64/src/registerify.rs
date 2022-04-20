pub(crate) struct StackToReg {
    idx: usize,
}

impl StackToReg {
    pub fn new() -> StackToReg {
        StackToReg {
            idx: 0,
        }
    }

    pub fn push(&mut self) -> arm64::Reg {
        self.idx += 1;

        let reg = arm64::Reg(self.idx as u32 - 1);
        
        reg
    }

    pub fn pop(&mut self) -> arm64::Reg {
        self.idx -= 1;
        arm64::Reg(self.idx as u32)
    }

    pub fn peek(&self) -> arm64::Reg {
        arm64::Reg(self.idx as u32 - 1)
    }
    
    pub fn peek_at(&self, off: usize) -> arm64::Reg {
        arm64::Reg(self.idx as u32 - 1 - off as u32)
    }

    pub(crate) fn at(&self, idx: usize) -> arm64::Reg {
        arm64::Reg(idx as u32)
    }

    pub fn push_many(&mut self, count: usize) {
        // Can't just do self.idx += count; since we need to check for clobbered registers
        for _ in 0..count {
            self.push();
        }
    }

    pub fn pop_many(&mut self, count: usize) {
        self.idx -= count;
    }

    pub fn zero(&mut self) {
        self.idx = 0;
    }

    pub fn size(&self) -> usize {
        self.idx
    }
}