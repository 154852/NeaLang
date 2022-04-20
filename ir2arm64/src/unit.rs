use std::collections::HashMap;

use crate::registerify::{StackToReg};

pub(crate) enum LocalSymbol {
    If,
    /// Start, End
    Loop(arm64::LocalSymbolID, arm64::LocalSymbolID)
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
    function: &'a ir::Function,
    unit: &'a ir::TranslationUnit,
    stack: StackToReg,
    local_symbols: LocalSymbolStack,
    local_symbols_allocated: usize,
}

impl<'a> FunctionTranslationContext<'a> {
    fn new(function: &'a ir::Function, unit: &'a ir::TranslationUnit) -> FunctionTranslationContext<'a> {
        FunctionTranslationContext {
            function, unit,
            stack: StackToReg::new(),
            local_symbols: LocalSymbolStack::new(),
            local_symbols_allocated: 1, // root
        }
    }

    pub(crate) fn new_local_symbol(&mut self) -> arm64::LocalSymbolID {
        self.local_symbols_allocated += 1;
        arm64::LocalSymbolID::new(self.local_symbols_allocated - 1)
    }

    pub(crate) fn stack(&mut self) -> &mut StackToReg {
        &mut self.stack
    }

    pub(crate) fn stack_ref(&self) -> &StackToReg {
        &self.stack
    }

    pub(crate) fn local_symbols(&mut self) -> &mut LocalSymbolStack {
        &mut self.local_symbols
    }

    pub(crate) fn unit(&self) -> &ir::TranslationUnit {
        &self.unit
    }

    pub(crate) fn local_addr(&self, idx: ir::LocalIndex) -> u32 {
        let mut addr = 0;

        assert!(self.function.local_count() > idx.idx());

        for i in &self.function.locals()[0..idx.idx() + 1] {
            addr += crate::util::size_for_storable_type(i.local_type()) as u32;
        }

        addr
    }

    pub(crate) fn symbol_id_for_function(&self, idx: ir::FunctionIndex) -> arm64::GlobalSymbolID {
        arm64::GlobalSymbolID::new(idx.idx())
    }

    pub(crate) fn symbol_id_for_global(&self, idx: ir::GlobalIndex) -> arm64::GlobalSymbolID {
        arm64::GlobalSymbolID::new(self.unit.function_count() + idx.idx())
    }
}

pub enum GlobalObjectId {
    Function(ir::FunctionIndex),
    Global(ir::GlobalIndex)
}

pub struct GlobalIDAllocator<'a> {
    unit: &'a ir::TranslationUnit,
    symbol_ids: HashMap<arm64::GlobalSymbolID, (usize, i64)>
}

impl<'a> GlobalIDAllocator<'a> {
    pub fn new(unit: &'a ir::TranslationUnit) -> GlobalIDAllocator<'a> {
        GlobalIDAllocator {
            unit,
            symbol_ids: HashMap::new()
        }
    }

    pub fn global_id_of_function(&self, func: ir::FunctionIndex) -> arm64::GlobalSymbolID {
        arm64::GlobalSymbolID::new(func.idx())
    }

    pub fn global_id_of_global(&self, global: ir::GlobalIndex) -> arm64::GlobalSymbolID {
        arm64::GlobalSymbolID::new(self.unit.function_count() + global.idx())
    }

    pub fn push_global_symbol_mapping(&mut self, global: arm64::GlobalSymbolID, symbol: usize, addend: i64) {
        self.symbol_ids.insert(global, (symbol, addend));
    }

    pub fn object_id_of_global_id(&self, idx: arm64::GlobalSymbolID) -> GlobalObjectId {
        if idx.idx() >= self.unit.function_count() {
            GlobalObjectId::Global(ir::GlobalIndex::new(idx.idx() - self.unit.function_count()))
        } else {
            GlobalObjectId::Function(ir::FunctionIndex::new(idx.idx()))
        }
    }

    pub fn symbol_for_global_id(&self, global: arm64::GlobalSymbolID) -> Option<(usize, i64)> {
        match self.symbol_ids.get(&global) {
            Some(x) => Some(*x),
            _ => None
        }
    }
}

pub struct TranslationContext {
    
}

impl TranslationContext {
    pub fn new() -> TranslationContext {
        TranslationContext { }
    }

    pub fn translate_function(&self, func: &ir::Function, unit: &ir::TranslationUnit) -> Vec<arm64::Ins> {
        if func.is_extern() { panic!("Cannot translate extern function"); }

        let mut arm64_ins = Vec::new();

        let mut ftc = FunctionTranslationContext::new(func, unit);

        let mut frame_size = 16; // lr, fp
        if let Some(last) = func.last_local_index() {
            frame_size += ftc.local_addr(last) as i32;
        }

        if frame_size & 15 != 0 {
            frame_size += 16 - (frame_size & 15);
        }

        /*
        STACK:
        +
        prev fp + lr
        -> fp
        locals
        -> sp
        -
        */

        arm64_ins.push(arm64::Ins::SubImm {
            size: arm64::SizeFlag::Size64,
            src: arm64::Reg::sp(),
            dest: arm64::Reg::sp(),
            val: frame_size as u32,
            shift: arm64::ImmShift::Shift0
        });
        arm64_ins.push(arm64::Ins::Stp {
            size: arm64::SizeFlag::Size64,
            src1: arm64::Reg::fp(),
            src2: arm64::Reg::lr(),
            base: arm64::Reg::sp(),
            offset: frame_size - 16,
            mode: arm64::IndexMode::SignedOffset
        });
        arm64_ins.push(arm64::Ins::AddImm {
            size: arm64::SizeFlag::Size64,
            dest: arm64::Reg::fp(),
            src: arm64::Reg::sp(),
            shift: arm64::ImmShift::Shift0,
            val: frame_size as u32 - 16
        });

        // Put params into locals
        for (p, _) in func.signature().params().iter().enumerate() {
            arm64_ins.push(arm64::Ins::Stur {
                size: arm64::SizeFlag::Size64,
                base: arm64::Reg::fp(),
                offset: -(ftc.local_addr(ir::LocalIndex::new(p)) as i32),
                src: arm64::Reg(p as u32)
            });
        }
        
        for ins in func.code() {
            self.translate_instruction_to(ins, &mut ftc, &mut arm64_ins);
        }

        arm64_ins.push(arm64::Ins::LocalSymbol(arm64::LocalSymbolID::new(0)));

        arm64_ins.push(arm64::Ins::Ldp {
            size: arm64::SizeFlag::Size64,
            dest1: arm64::Reg::fp(),
            dest2: arm64::Reg::lr(),
            base: arm64::Reg::sp(),
            offset: frame_size - 16,
            mode: arm64::IndexMode::SignedOffset
        });
        arm64_ins.push(arm64::Ins::AddImm {
            size: arm64::SizeFlag::Size64,
            src: arm64::Reg::sp(),
            dest: arm64::Reg::sp(),
            val: frame_size as u32,
            shift: arm64::ImmShift::Shift0
        });

        arm64_ins.push(arm64::Ins::Ret(arm64::Reg::lr()));

        arm64_ins
    }

    fn translate_storable(&self, storable: &ir::StorableValue, unit: &ir::TranslationUnit, gid_allocator: &GlobalIDAllocator, relocs: &mut Vec<arm64::Relocation>, offset: usize, section_offset: usize, addend: i64) -> Vec<u8> {
        match storable {
            ir::StorableValue::Value(v) => match v {
                ir::Value::U8(i) => i.to_le_bytes().to_vec(),
                ir::Value::I8(i) => i.to_le_bytes().to_vec(),
                ir::Value::U16(i) => i.to_le_bytes().to_vec(),
                ir::Value::I16(i) => i.to_le_bytes().to_vec(),
                ir::Value::U32(i) => i.to_le_bytes().to_vec(),
                ir::Value::I32(i) => i.to_le_bytes().to_vec(),
                ir::Value::U64(i) => i.to_le_bytes().to_vec(),
                ir::Value::I64(i) => i.to_le_bytes().to_vec(),
                ir::Value::UPtr(i) => (*i as u64).to_le_bytes().to_vec(),
                ir::Value::IPtr(i) => (*i as i64).to_le_bytes().to_vec(),
                ir::Value::Bool(i) => (*i as u8).to_le_bytes().to_vec(),
                ir::Value::Ref(idx) => {
                    relocs.push(arm64::Relocation::new_global_absolute(gid_allocator.global_id_of_global(*idx), section_offset + offset, 0));
                    
                    (0 as u64).to_le_bytes().to_vec()
                },
            },
            ir::StorableValue::Compound(c) => {
                match c {
                    ir::CompoundValue::Struct(s) => {
                        let mut data = Vec::new();

                        for p in s.props() {
                            data.extend(self.translate_storable(p.value(), unit, gid_allocator, relocs, offset + data.len(), section_offset, addend));
                        }

                        data
                    }
                }
            },
            ir::StorableValue::Slice(data_global, start, length) => {
                let mut data = Vec::new();

                assert_eq!(*start, 0); // TODO: Values need to be type checked anyway

                data.extend(vec![0; 8]);
                relocs.push(arm64::Relocation::new_global_absolute(
                    gid_allocator.global_id_of_global(*data_global),
                    section_offset + offset,
                    addend
                )); 

                data.extend((*length as u64).to_le_bytes().to_vec());

                data
            },
            ir::StorableValue::SliceData(values) => {
                let mut data = Vec::new();

                for value in values {
                    data.extend(self.translate_storable(value, unit, gid_allocator, relocs, offset + data.len(), section_offset, addend));
                }

                data
            }
        }
    }

    pub fn translate_global(&self, global: &ir::Global, unit: &ir::TranslationUnit, gid_allocator: &GlobalIDAllocator, relocs: &mut Vec<arm64::Relocation>, section_offset: usize, addend: i64) -> Vec<u8> {
        if let Some(default) = global.default() {
            self.translate_storable(default, unit, gid_allocator, relocs, 0, section_offset, addend)
        } else {
            vec![0; crate::util::size_for_storable_type(global.global_type())]
        }
    }
}