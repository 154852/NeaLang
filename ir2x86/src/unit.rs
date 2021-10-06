use x86::GlobalSymbolID;

use crate::{registerify::StackToReg};

pub(crate) enum LocalSymbol {
    If,
    /// Start, End
    Loop(x86::LocalSymbolID, x86::LocalSymbolID)
}

pub(crate) struct LocalSymbolStack {
    symbols: Vec<LocalSymbol>
}

impl LocalSymbolStack {
    fn new() -> LocalSymbolStack {
        LocalSymbolStack {
            symbols: Vec::new()
        }
    }

    pub(crate) fn push(&mut self, symbol: LocalSymbol) {
        self.symbols.push(symbol);
    }

    pub(crate) fn pop(&mut self) {
        self.symbols.pop();
    }
}

pub struct FunctionTranslationContext<'a> {
    mode: x86::Mode,
    function: &'a ir::Function,
    unit: &'a ir::TranslationUnit,
    stack: StackToReg,
    local_symbols: LocalSymbolStack,
    local_symbols_allocated: x86::LocalSymbolID
}

impl<'a> FunctionTranslationContext<'a> {
    fn new(mode: x86::Mode, function: &'a ir::Function, unit: &'a ir::TranslationUnit) -> FunctionTranslationContext<'a> {
        FunctionTranslationContext {
            mode, function, unit,
            stack: StackToReg::new(mode),
            local_symbols: LocalSymbolStack::new(),
            local_symbols_allocated: 1 // root
        }
    }

    pub(crate) fn new_local_symbol(&mut self) -> x86::LocalSymbolID {
        self.local_symbols_allocated += 1;
        self.local_symbols_allocated - 1
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

    pub(crate) fn unit(&self) -> &ir::TranslationUnit {
        &self.unit
    }

    pub(crate) fn local_addr(&self, idx: ir::LocalIndex) -> u64 {
        let mut addr = 0;

        assert!(self.function.locals().len() > idx);

        for i in self.function.locals().iter().take(idx + 1) {
            addr += crate::registerify::size_for_st(i.local_type(), self.mode) as u64;
        }

        addr
    }

    pub(crate) fn local_mem(&self, idx: ir::LocalIndex) -> x86::Mem {
        x86::Mem::new().base(x86::RegClass::Ebp).disp(-(self.local_addr(idx) as i64))
    }

    pub(crate) fn symbol_id_for_function(&self, idx: ir::FunctionIndex) -> GlobalSymbolID {
        idx
    }

    pub(crate) fn symbol_id_for_global(&self, idx: ir::GlobalIndex) -> GlobalSymbolID {
        self.unit.functions().len() + idx
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

    pub fn translate_function(&self, func: &ir::Function, unit: &ir::TranslationUnit) -> Vec<x86::Ins> {
        if func.is_extern() { panic!("Cannot translate extern function"); }

        let mut x86_ins = Vec::new();

        let mut ftc = FunctionTranslationContext::new(self.mode, func, unit);

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

        // Root is always 0
        x86_ins.push(x86::Ins::LocalSymbol(0));
        if func.locals().len() > 0 {
            x86_ins.push(x86::Ins::MovRegReg(self.mode.stack_ptr(), self.mode.base_ptr()));
            x86_ins.push(x86::Ins::PopReg(self.mode.base_ptr()));
        }
        x86_ins.push(x86::Ins::Ret);

        x86_ins
    }

    pub fn translate_global(&self, global: &ir::Global, _unit: &ir::TranslationUnit) -> Vec<u8> {
        vec![0; crate::registerify::size_for_st(global.global_type(), self.mode)]
    }
}