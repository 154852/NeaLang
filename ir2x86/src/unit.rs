use std::collections::HashMap;

use x86::GlobalSymbolID;

use crate::{registerify::{SYS_V_ABI, StackToReg, reg_for_value_type}};

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

        assert!(self.function.local_count() > idx);

        for i in self.function.locals().iter().take(idx + 1) {
            addr += crate::registerify::size_for_storable_type(i.local_type(), self.mode) as u64;
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
        self.unit.function_count() + idx
    }
}

pub struct GlobalIDAllocator<'a> {
	unit: &'a ir::TranslationUnit,
	symbol_ids: HashMap<x86::GlobalSymbolID, (usize, i64)>
}

impl<'a> GlobalIDAllocator<'a> {
    pub fn new(unit: &'a ir::TranslationUnit) -> GlobalIDAllocator<'a> {
        GlobalIDAllocator {
            unit,
            symbol_ids: HashMap::new()
        }
    }

	pub fn global_id_of_function(&self, func: usize) -> x86::GlobalSymbolID {
		func
	}

	pub fn global_id_of_global(&self, global: usize) -> x86::GlobalSymbolID {
		self.unit.function_count() + global
	}

	pub fn push_global_symbol_mapping(&mut self, global: x86::GlobalSymbolID, symbol: usize, addend: i64) {
		self.symbol_ids.insert(global, (symbol, addend));
	}

	pub fn symbol_for_global_id(&self, global: x86::GlobalSymbolID) -> Option<(usize, i64)> {
		match self.symbol_ids.get(&global) {
            Some(x) => Some(*x),
            _ => None
        }
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

        if func.local_count() > 0 {
            x86_ins.push(x86::Ins::PushReg(self.mode.base_ptr()));
            x86_ins.push(x86::Ins::MovRegReg(self.mode.base_ptr(), self.mode.stack_ptr()));
            x86_ins.push(x86::Ins::SubRegImm(self.mode.stack_ptr(), ftc.local_addr(func.local_count() - 1)));

            // Put params into locals
            for (p, param) in func.signature().params().iter().enumerate() {
                x86_ins.push(x86::Ins::MovMemReg(
                    ftc.local_mem(p),
                    reg_for_value_type(param, self.mode, SYS_V_ABI[p])
                ));
            }
        }

        for ins in func.code() {
            self.translate_instruction_to(ins, &mut ftc, &mut x86_ins);
        }

        // Root is always 0
        x86_ins.push(x86::Ins::LocalSymbol(0));
        if func.local_count() > 0 {
            x86_ins.push(x86::Ins::MovRegReg(self.mode.stack_ptr(), self.mode.base_ptr()));
            x86_ins.push(x86::Ins::PopReg(self.mode.base_ptr()));
        }
        x86_ins.push(x86::Ins::Ret);

        x86_ins
    }

    fn translate_storable(&self, storable: &ir::Storable, unit: &ir::TranslationUnit, gid_allocator: &GlobalIDAllocator, relocs: &mut Vec<x86::Relocation>, global: GlobalSymbolID, offset: usize, section_offset: usize, addend: i64) -> Vec<u8> {
        match storable {
            ir::Storable::Value(v) => match v {
                ir::Value::U8(i) => i.to_le_bytes().to_vec(),
                ir::Value::I8(i) => i.to_le_bytes().to_vec(),
                ir::Value::U16(i) => i.to_le_bytes().to_vec(),
                ir::Value::I16(i) => i.to_le_bytes().to_vec(),
                ir::Value::U32(i) => i.to_le_bytes().to_vec(),
                ir::Value::I32(i) => i.to_le_bytes().to_vec(),
                ir::Value::U64(i) => i.to_le_bytes().to_vec(),
                ir::Value::I64(i) => i.to_le_bytes().to_vec(),
                ir::Value::UPtr(i) => match self.mode {
                    x86::Mode::X86 => (*i as u32).to_le_bytes().to_vec(),
                    x86::Mode::X8664 => (*i as u64).to_le_bytes().to_vec(),
                },
                ir::Value::IPtr(i) => match self.mode {
                    x86::Mode::X86 => (*i as i32).to_le_bytes().to_vec(),
                    x86::Mode::X8664 => (*i as u64).to_le_bytes().to_vec(),
                },
                ir::Value::Bool(i) => (*i as u8).to_le_bytes().to_vec(),
                ir::Value::Ref(idx) => {
                    relocs.push(x86::Relocation::new_global_absolute(gid_allocator.global_id_of_global(*idx), section_offset + offset, 0));
                    
                    match self.mode {
                        x86::Mode::X86 => (0 as u32).to_le_bytes().to_vec(),
                        x86::Mode::X8664 => (0 as u64).to_le_bytes().to_vec(),
                    }
                },
            },
            ir::Storable::Compound(c) => {
                match c {
                    ir::Compound::Struct(s) => {
                        let mut data = Vec::new();

                        for p in s.props() {
                            data.extend(self.translate_storable(p.value(), unit, gid_allocator, relocs, global, offset + data.len(), section_offset, addend));
                        }

                        data
                    }
                }
            },
            ir::Storable::Slice(data_global, start, length) => {
                let mut data = Vec::new();

                assert_eq!(*start, 0); // TODO: Values need to be type checked anyway

                data.extend(vec![0; self.mode.ptr_size()]);
                relocs.push(x86::Relocation::new_global_absolute(
                    gid_allocator.global_id_of_global(*data_global),
                    section_offset + offset,
                    addend
                )); 

                data.extend(match self.mode {
                    x86::Mode::X86 => (*length as u32).to_le_bytes().to_vec(),
                    x86::Mode::X8664 => (*length as u64).to_le_bytes().to_vec(),
                });

                data
            },
            ir::Storable::SliceData(values) => {
                let mut data = Vec::new();

                for value in values {
                    data.extend(self.translate_storable(value, unit, gid_allocator, relocs, global, offset + data.len(), section_offset, addend));
                }

                data
            }
        }
    }

    pub fn translate_global(&self, global: &ir::Global, unit: &ir::TranslationUnit, gid_allocator: &GlobalIDAllocator, relocs: &mut Vec<x86::Relocation>, symbol: GlobalSymbolID, section_offset: usize, addend: i64) -> Vec<u8> {
        if let Some(default) = global.default() {
            self.translate_storable(default, unit, gid_allocator, relocs, symbol, 0, section_offset, addend)
        } else {
            vec![0; crate::registerify::size_for_storable_type(global.global_type(), self.mode)]
        }
    }
}