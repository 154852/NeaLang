use crate::{LocalSymbolStack, registerify::StackToReg};

pub struct FunctionTranslationContext<'a> {
    mode: x86::Mode,
    function: &'a ir::Function,
    stack: StackToReg,
    local_symbols: LocalSymbolStack
}

impl<'a> FunctionTranslationContext<'a> {
    fn new(mode: x86::Mode, function: &'a ir::Function) -> FunctionTranslationContext<'a> {
        FunctionTranslationContext {
            mode, function,
            stack: StackToReg::new(mode),
            local_symbols: LocalSymbolStack::new()
        }
    }

    pub(crate) fn stack(&mut self) -> &mut StackToReg {
        &mut self.stack
    }

    pub(crate) fn stack_ref(&self) -> &StackToReg {
        &self.stack
    }

    pub(crate) fn mode(&self) -> x86::Mode {
        self.mode
    }

    pub(crate) fn local_symbols(&mut self) -> &mut LocalSymbolStack {
        &mut self.local_symbols
    }

    pub(crate) fn func(&self) -> &ir::Function {
        &self.function
    }

    pub(crate) fn local_addr(&self, idx: ir::LocalIndex) -> u64 {
        let mut addr = 0;

        assert!(self.function.locals().len() > idx);

        for i in self.function.locals().iter().take(idx + 1) {
            addr += i.value_type().bytes_size(self.mode.ptr_size() as u64);
        }

        addr
    }

    pub(crate) fn local_mem(&self, idx: ir::LocalIndex) -> x86::Mem {
        x86::Mem::new().base(x86::RegClass::Ebp).disp(-(self.local_addr(idx) as i64))
    }
}

pub struct TranslationContext {
    pub(crate) mode: x86::Mode
}

impl TranslationContext {
    pub fn new(mode: x86::Mode) -> TranslationContext {
        TranslationContext {
            mode
        }
    }

    pub fn translate_function(&self, func: &ir::Function, _unit: &ir::TranslationUnit) -> Vec<x86::Ins> {
        let mut x86_ins = Vec::new();

        let mut ftc = FunctionTranslationContext::new(self.mode, func);

        if func.signature().params().len() == 0 {
            ftc.stack().set_no_params();
        } else {
            ftc.stack().push_many(func.signature().params().len());
        }

        if func.locals().len() > 0 {
            x86_ins.push(x86::Ins::PushReg(self.mode.base_ptr()));
            x86_ins.push(x86::Ins::MovRegReg(self.mode.base_ptr(), self.mode.stack_ptr()));
            x86_ins.push(x86::Ins::SubRegImm(self.mode.stack_ptr(), ftc.local_addr(func.locals().len() - 1)));
        }

        for ins in func.code() {
            self.translate_instruction_to(ins, &mut ftc, &mut x86_ins);
        }

        x86_ins.push(x86::Ins::LocalSymbol(ftc.local_symbols().root()));
        if func.locals().len() > 0 {
            x86_ins.push(x86::Ins::MovRegReg(self.mode.stack_ptr(), self.mode.base_ptr()));
            x86_ins.push(x86::Ins::PopReg(self.mode.base_ptr()));
        }
        x86_ins.push(x86::Ins::Ret);

        x86_ins
    }
}