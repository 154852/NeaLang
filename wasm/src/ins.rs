use crate::{DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, NumType, RefType, TableIdx, TypeIdx, ValType, encode::WasmEncodable};

pub enum BlockType {
	Empty,
	Value(ValType),
	Type(TypeIdx)
}

impl WasmEncodable for BlockType {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
			BlockType::Empty => data.push(0x40),
			BlockType::Value(v) => v.wasm_encode(data),
			BlockType::Type(t) => t.wasm_encode(data),
		}
    }
}

pub struct MemArg {
	align: u32,
	offset: u32
}

impl MemArg {
	pub fn new(align: u32, offset: u32) -> MemArg {
		MemArg {
			align, offset
		}
	}
}

impl WasmEncodable for MemArg {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        self.align.wasm_encode(data);
		self.offset.wasm_encode(data);
    }
}

pub enum NumSize {
	Bits8,
	Bits16,
	Bits32,
	Bits64
}

pub enum Ins {
	Unreachable, Nop,
	Block(BlockType, Vec<Ins>),
	Loop(BlockType, Vec<Ins>),
	If(BlockType, Vec<Ins>),
	IfElse(BlockType, Vec<Ins>, Vec<Ins>),
	Br(LabelIdx),
	BrIf(LabelIdx),
	BrTable(Vec<LabelIdx>, LabelIdx),
	Return,
	Call(FuncIdx),
	CallIndirect(TypeIdx, TableIdx),

	RefNull(RefType),
	RefIsNull,
	RefFunc(FuncIdx),

	Drop,
	Select,
	SelectTypes(Vec<ValType>),

	LocalGet(LocalIdx),
	LocalSet(LocalIdx),
	LocalTee(LocalIdx),
	GlobalGet(GlobalIdx),
	GlobalSet(GlobalIdx),

	TableGet(TableIdx),
	TableSet(TableIdx),
	TableInit(ElemIdx, TableIdx),
	ElemDrop(ElemIdx),
	TableCopy(TableIdx, TableIdx),
	TableGrow(TableIdx),
	TableSize(TableIdx),
	TableFill(TableIdx),

	Load(NumType, MemArg),
	LoadSX(NumType, NumSize, MemArg),
	LoadZX(NumType, NumSize, MemArg),
	Store(NumType, MemArg),
	StoreTrunc(NumType, NumSize, MemArg),
	MemorySize,
	MemoryGrow,
	MemoryInit(DataIdx),
	DataDrop(DataIdx),
	MemoryCopy,
	MemoryFill,

	ConstI32(i32),
	ConstI64(i64),
	ConstF32(f32),
	ConstF64(f64),

	Eqz(NumType),
	Eq(NumType),
	Ne(NumType),
	Lt(NumType, bool),
	Le(NumType, bool),
	Gt(NumType, bool),
	Ge(NumType, bool),

	Add(NumType),
	Sub(NumType),
	Mul(NumType),
	Div(NumType, bool),
	Rem(NumType, bool),
	And(NumType),
	Or(NumType),
	Xor(NumType)
}

impl WasmEncodable for Ins {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
            Ins::Unreachable => data.push(0x00),
            Ins::Nop => data.push(0x01),
            Ins::Block(bt, ins) => {
				data.push(0x02);
				bt.wasm_encode(data);
				for ins in ins { ins.wasm_encode(data); }
				data.push(0x0b);
			},
            Ins::Loop(bt, ins) => {
				data.push(0x03);
				bt.wasm_encode(data);
				for ins in ins { ins.wasm_encode(data); }
				data.push(0x0b);
			},
            Ins::If(bt, ins) => {
				data.push(0x04);
				bt.wasm_encode(data);
				for ins in ins { ins.wasm_encode(data); }
				data.push(0x0b);
			},
            Ins::IfElse(bt, true_then, false_then) => {
				data.push(0x04);
				bt.wasm_encode(data);
				for ins in true_then { ins.wasm_encode(data); }
				data.push(0x05);
				for ins in false_then { ins.wasm_encode(data); }
				data.push(0x0b);
			},
            Ins::Br(idx) => {
				data.push(0x0c);
				idx.wasm_encode(data);
			},
            Ins::BrIf(idx) => {
				data.push(0x0d);
				idx.wasm_encode(data);
			},
            Ins::BrTable(labels, default) => {
				data.push(0x0e);
				labels.wasm_encode(data);
				default.wasm_encode(data);
			},
            Ins::Return => data.push(0x0f),
            Ins::Call(idx) => {
				data.push(0x10);
				idx.wasm_encode(data);
			},
            Ins::CallIndirect(type_idx, table) => {
				data.push(0x11);
				type_idx.wasm_encode(data);
				table.wasm_encode(data);
			},

			Ins::RefNull(rt) => {
				data.push(0xd0);
				rt.wasm_encode(data);
			},
			Ins::RefIsNull => data.push(0xd1),
			Ins::RefFunc(idx) => {
				data.push(0xd2);
				idx.wasm_encode(data);
			},

			Ins::Drop => data.push(0x1a),
			Ins::Select => data.push(0x1b),
			Ins::SelectTypes(types) => {
				data.push(0x1c);
				types.wasm_encode(data);
			},

			Ins::LocalGet(idx) => {
				data.push(0x20);
				idx.wasm_encode(data);
			},
			Ins::LocalSet(idx) => {
				data.push(0x21);
				idx.wasm_encode(data);
			},
			Ins::LocalTee(idx) => {
				data.push(0x22);
				idx.wasm_encode(data);
			},
			Ins::GlobalGet(idx) => {
				data.push(0x23);
				idx.wasm_encode(data);
			},
			Ins::GlobalSet(idx) => {
				data.push(0x24);
				idx.wasm_encode(data);
			},

			Ins::TableGet(idx) => {
				data.push(0x25);
				idx.wasm_encode(data);
			},
			Ins::TableSet(idx) => {
				data.push(0x26);
				idx.wasm_encode(data);
			},
			Ins::TableInit(elem, table) => {
				data.extend([0xfc, 12]);
				elem.wasm_encode(data);
				table.wasm_encode(data);
			},
			Ins::ElemDrop(idx) => {
				data.extend([0xfc, 13]);
				idx.wasm_encode(data);
			},
			Ins::TableCopy(a, b) => {
				data.extend([0xfc, 14]);
				a.wasm_encode(data);
				b.wasm_encode(data);
			},
			Ins::TableGrow(idx) => {
				data.extend([0xfc, 15]);
				idx.wasm_encode(data);
			},
			Ins::TableSize(idx) => {
				data.extend([0xfc, 16]);
				idx.wasm_encode(data);
			},
			Ins::TableFill(idx) => {
				data.extend([0xfc, 17]);
				idx.wasm_encode(data);
			},

			Ins::Load(nt, arg) => {
				data.push(match nt {
					NumType::I32 => 0x28,
					NumType::I64 => 0x29,
					NumType::F32 => 0x2a,
					NumType::F64 => 0x2b,
				});
				arg.wasm_encode(data);
			},
			Ins::LoadSX(nt, from, arg) => {
				data.push(match nt {
					NumType::I32 => match from {
						NumSize::Bits8 => 0x2c,
						NumSize::Bits16 => 0x2e,
						_ => panic!("Invalid load SX source size")
					},
					NumType::I64 => match from {
						NumSize::Bits8 => 0x30,
						NumSize::Bits16 => 0x32,
						NumSize::Bits32 => 0x34,
						_ => panic!("Invalid load SX source size")
					},
					_ => panic!("Invalid load SX destination")
				});
				arg.wasm_encode(data);
			},
			Ins::LoadZX(nt, from, arg) => {
				data.push(match nt {
					NumType::I32 => match from {
						NumSize::Bits8 => 0x2d,
						NumSize::Bits16 => 0x2f,
						_ => panic!("Invalid load ZX source size")
					},
					NumType::I64 => match from {
						NumSize::Bits8 => 0x31,
						NumSize::Bits16 => 0x33,
						NumSize::Bits32 => 0x35,
						_ => panic!("Invalid load ZX source size")
					},
					_ => panic!("Invalid load ZX destination")
				});
				arg.wasm_encode(data);
			},
			Ins::Store(nt, arg) => {
				data.push(match nt {
					NumType::I32 => 0x36,
					NumType::I64 => 0x37,
					NumType::F32 => 0x38,
					NumType::F64 => 0x39,
				});
				arg.wasm_encode(data);
			},
			Ins::StoreTrunc(nt, to, arg) => {
				data.push(match nt {
					NumType::I32 => match to {
						NumSize::Bits8 => 0x3a,
						NumSize::Bits16 => 0x3b,
						_ => panic!("Invalid store trunc dest size")
					},
					NumType::I64 => match to {
						NumSize::Bits8 => 0x3c,
						NumSize::Bits16 => 0x3d,
						NumSize::Bits32 => 0x3e,
						_ => panic!("Invalid store trunc dest size")
					},
					_ => panic!("Invalid store trunc source size")
				});
				arg.wasm_encode(data);
			},
			Ins::MemorySize => data.extend([0x3f, 0x00]),
			Ins::MemoryGrow => data.extend([0x40, 0x00]),
			Ins::MemoryInit(idx) => {
				data.extend([0xfc, 8]);
				idx.wasm_encode(data);
				data.push(0x00);
			},
			Ins::DataDrop(idx) => {
				data.extend([0xfc, 9]);
				idx.wasm_encode(data);
			},
			Ins::MemoryCopy => data.extend([0xfc, 10, 0x00, 0x00]),
			Ins::MemoryFill => data.extend([0xfc, 11, 0x00]),

			Ins::ConstI32(i) => {
				data.push(0x41);
				i.wasm_encode(data);
			},
			Ins::ConstI64(i) => {
				data.push(0x42);
				i.wasm_encode(data);
			},
			Ins::ConstF32(i) => {
				data.push(0x43);
				i.wasm_encode(data);
			},
			Ins::ConstF64(i) => {
				data.push(0x44);
				i.wasm_encode(data);
			},

			Ins::Eqz(t) => data.push(match t {
				NumType::I32 => 0x45,
				NumType::I64 => 0x50,
				_ => panic!("Eqz only exists on i32 and i64")
			}),
			Ins::Eq(t) => data.push(match t {
				NumType::I32 => 0x46,
				NumType::I64 => 0x51,
				NumType::F32 => 0x5b,
				NumType::F64 => 0x61
			}),
			Ins::Ne(t) => data.push(match t {
				NumType::I32 => 0x47,
				NumType::I64 => 0x52,
				NumType::F32 => 0x5c,
				NumType::F64 => 0x62
			}),
			Ins::Lt(t, signed) => data.push(match t {
				NumType::I32 => if *signed { 0x48 } else { 0x49 },
				NumType::I64 => if *signed { 0x53 } else { 0x54 },
				NumType::F32 => 0x5d,
				NumType::F64 => 0x63
			}),
			Ins::Le(t, signed) => data.push(match t {
				NumType::I32 => if *signed { 0x4c } else { 0x4d },
				NumType::I64 => if *signed { 0x57 } else { 0x58 },
				NumType::F32 => 0x5f,
				NumType::F64 => 0x65
			}),
			Ins::Gt(t, signed) => data.push(match t {
				NumType::I32 => if *signed { 0x4a } else { 0x4b },
				NumType::I64 => if *signed { 0x55 } else { 0x56 },
				NumType::F32 => 0x5e,
				NumType::F64 => 0x64
			}),
			Ins::Ge(t, signed) => data.push(match t {
				NumType::I32 => if *signed { 0x4e } else { 0x4f },
				NumType::I64 => if *signed { 0x59 } else { 0x5a },
				NumType::F32 => 0x60,
				NumType::F64 => 0x66
			}),
			Ins::Add(t) => data.push(match t {
				NumType::I32 => 0x6a,
				NumType::I64 => 0x7c,
				NumType::F32 => 0x92,
				NumType::F64 => 0xa0
			}),
			Ins::Sub(t) => data.push(match t {
				NumType::I32 => 0x6b,
				NumType::I64 => 0x7d,
				NumType::F32 => 0x93,
				NumType::F64 => 0xa1
			}),
			Ins::Mul(t) => data.push(match t {
				NumType::I32 => 0x6c,
				NumType::I64 => 0x7e,
				NumType::F32 => 0x94,
				NumType::F64 => 0xa2
			}),
			Ins::Div(t, signed) => data.push(match t {
				NumType::I32 => if *signed { 0x6d } else { 0x6e },
				NumType::I64 => if *signed { 0x7f } else { 0x80 },
				NumType::F32 => 0x94,
				NumType::F64 => 0xa3
			}),
			Ins::Rem(t, signed) => data.push(match t {
				NumType::I32 => if *signed { 0x6f } else { 0x70 },
				NumType::I64 => if *signed { 0x81 } else { 0x82 },
				_ => panic!("Rem only exists on i32 and i64")
			}),
			Ins::Or(t) => data.push(match t {
				NumType::I32 => 0x72,
				NumType::I64 => 0x84,
				_ => panic!("Or only exists on i32 and i64")
			}),
			Ins::And(t) => data.push(match t {
				NumType::I32 => 0x71,
				NumType::I64 => 0x85,
				_ => panic!("And only exists on i32 and i64")
			}),
			Ins::Xor(t) => data.push(match t {
				NumType::I32 => 0x73,
				NumType::I64 => 0x85,
				_ => panic!("Xor only exists on i32 and i64")
			}),
        }
    }
}

pub struct Expr {
	ins: Vec<Ins>
}

impl Expr {
	pub fn new() -> Expr {
		Expr {
			ins: Vec::new()
		}
	}

	pub fn with(ins: Vec<Ins>) -> Expr {
		Expr {
			ins
		}
	}	

	pub fn push(&mut self, ins: Ins) {
		self.ins.push(ins);
	}
}

impl WasmEncodable for Expr {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		for ins in &self.ins {
			ins.wasm_encode(data);
		}
		data.push(0x0b);
	}
}