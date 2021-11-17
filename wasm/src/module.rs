use crate::{Expr, encode::WasmEncodable};

#[derive(PartialEq, Eq)]
pub enum NumType {
	I32,
	I64,
	F32,
	F64,
}

impl WasmEncodable for NumType {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        data.push(match self {
            NumType::I32 => 0x7f,
            NumType::I64 => 0x7e,
            NumType::F32 => 0x7d,
            NumType::F64 => 0x7c,
        });
    }
}

pub enum RefType {
	FuncRef,
	ExternRef
}

impl WasmEncodable for RefType {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		data.push(match self {
			RefType::FuncRef => 0x70,
            RefType::ExternRef => 0x6f,
		});
	}
}

pub enum ValType {
	Num(NumType),
	Ref(RefType)
}

impl WasmEncodable for ValType {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
            ValType::Num(n) => n.wasm_encode(data),
            ValType::Ref(r) => r.wasm_encode(data),
        }
    }
}

pub type ResultType = Vec<ValType>;

pub struct FunctionType {
	parameters: ResultType,
	results: ResultType,
}

impl FunctionType {
	pub fn new(parameters: ResultType, results: ResultType) -> FunctionType {
		FunctionType {
			parameters, results
		}
	}
}

impl WasmEncodable for FunctionType {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        data.push(0x60);
		self.parameters.wasm_encode(data);
		self.results.wasm_encode(data);
    }
}

pub struct Limits {
	min: u32,
	max: Option<u32>
}

impl Limits {
	pub fn new(min: u32) -> Limits {
		Limits {
			min,
			max: None
		}
	}

	pub fn new_with_max(min: u32, max: u32) -> Limits {
		Limits {
			min,
			max: Some(max)
		}
	}
}

impl WasmEncodable for Limits {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        if let Some(max) = self.max {
			data.push(0x00);
			self.min.wasm_encode(data);
			max.wasm_encode(data)
		} else {
			data.push(0x00);
			self.min.wasm_encode(data);
		}
    }
}

pub struct MemType {
	limits: Limits
}

impl MemType {
	pub fn new(limits: Limits) -> MemType {
		MemType {
			limits
		}
	}
}

impl WasmEncodable for MemType {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		self.limits.wasm_encode(data);
	}
}

pub struct TableType {
	ref_type: RefType,
	limits: Limits
}

impl WasmEncodable for TableType {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		self.ref_type.wasm_encode(data);
		self.limits.wasm_encode(data);
	}
}

pub struct GlobalType {
	val_type: ValType,
	mutable: bool
}

impl GlobalType {
	pub fn new(val_type: ValType) -> GlobalType {
		GlobalType {
			val_type,
			mutable: false
		}
	}
}

impl WasmEncodable for GlobalType {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        self.val_type.wasm_encode(data);
		data.push(if self.mutable { 1 } else { 0 });
    }
}

pub type TypeIdx = usize;
pub type FuncIdx = usize;
pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
pub type ElemIdx = usize;
pub type DataIdx = usize;
pub type LocalIdx = usize;
pub type LabelIdx = usize;

pub enum ImportDescriptor {
	Type(TypeIdx),
	Table(TableType),
	Mem(MemType),
	Global(GlobalType),
}

impl WasmEncodable for ImportDescriptor {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
            ImportDescriptor::Type(idx) => {
				data.push(0x00);
				idx.wasm_encode(data);
			},
            ImportDescriptor::Table(idx) => {
				data.push(0x01);
				idx.wasm_encode(data);
			},
            ImportDescriptor::Mem(idx) => {
				data.push(0x02);
				idx.wasm_encode(data);
			},
            ImportDescriptor::Global(idx) => {
				data.push(0x03);
				idx.wasm_encode(data);
			},
        }
    }
}

pub struct Import {
	module_name: String,
	name: String,
	descriptor: ImportDescriptor
}

impl Import {
	pub fn new<T: Into<String>, U: Into<String>>(module_name: T, name: U, descriptor: ImportDescriptor) -> Import {
		Import {
			module_name: module_name.into(),
			name: name.into(),
			descriptor
		}
	}
}

impl WasmEncodable for Import {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		self.module_name.wasm_encode(data);
		self.name.wasm_encode(data);
		self.descriptor.wasm_encode(data);
	}
}

pub struct Global {
	global_type: GlobalType,
	expr: Expr
}

impl Global {
	pub fn new(global_type: GlobalType, expr: Expr) -> Global {
		Global {
			global_type, expr
		}
	}
}

impl WasmEncodable for Global {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		self.global_type.wasm_encode(data);
		self.expr.wasm_encode(data);
	}
}

pub enum ExportDescriptor {
	Func(FuncIdx),
	Table(TableIdx),
	Mem(MemIdx),
	Global(GlobalIdx),
}

impl WasmEncodable for ExportDescriptor {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
            ExportDescriptor::Func(idx) => {
				data.push(0x00);
				idx.wasm_encode(data);
			},
            ExportDescriptor::Table(idx) => {
				data.push(0x01);
				idx.wasm_encode(data);
			},
            ExportDescriptor::Mem(idx) => {
				data.push(0x02);
				idx.wasm_encode(data);
			},
            ExportDescriptor::Global(idx) => {
				data.push(0x03);
				idx.wasm_encode(data);
			},
        }
    }
}

pub struct Export {
	name: String,
	descriptor: ExportDescriptor
}

impl Export {
	pub fn new<T: Into<String>>(name: T, descriptor: ExportDescriptor) -> Export {
		Export {
			name: name.into(),
			descriptor
		}
	}
}

impl WasmEncodable for Export {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		self.name.wasm_encode(data);
		self.descriptor.wasm_encode(data);
	}
}

pub enum Elem {
	ActiveIndices(TableIdx, Expr, Vec<FuncIdx>),
	PassiveIndices(Vec<FuncIdx>),
	DeclarativeIndices(Vec<FuncIdx>),
	
	ActiveExprs(TableIdx, Expr, RefType, Vec<Expr>),
	PassiveExprs(RefType, Vec<Expr>),
	DeclarativeExprs(RefType, Vec<Expr>),
}

impl WasmEncodable for Elem {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
            Elem::ActiveIndices(tbl, offset, indices) => {
				if *tbl == 0 {
					data.push(0x00);
					offset.wasm_encode(data);
					indices.wasm_encode(data);
				} else {
					data.push(0x02);
					tbl.wasm_encode(data);
					offset.wasm_encode(data);
					data.push(0x00); // elemkind
					indices.wasm_encode(data);
				}
			},
            Elem::PassiveIndices(indices) => {
				data.extend([0x02, 0x00]);
				indices.wasm_encode(data);
			},
			Elem::DeclarativeIndices(indices) => {
				data.extend([0x03, 0x00]);
				indices.wasm_encode(data);
			},
            Elem::ActiveExprs(tbl, offset, rt, exprs) => {
				if *tbl == 0 {
					data.push(0x04);
					offset.wasm_encode(data);
					exprs.wasm_encode(data);
				} else {
					data.push(0x06);
					tbl.wasm_encode(data);
					offset.wasm_encode(data);
					rt.wasm_encode(data);
					exprs.wasm_encode(data);
				}
			},
            Elem::PassiveExprs(rt, exprs) => {
				data.push(0x05);
				rt.wasm_encode(data);
				exprs.wasm_encode(data);
			},
            Elem::DeclarativeExprs(rt, exprs) => {
				data.push(0x07);
				rt.wasm_encode(data);
				exprs.wasm_encode(data);
			},
			
        }
    }
}

pub struct Code {
	locals: Vec<ValType>,
	expr: Expr
}

impl Code {
	pub fn new(locals: Vec<ValType>, expr: Expr) -> Code {
		Code {
			locals, expr
		}
	}
}

impl WasmEncodable for Code {
	fn wasm_encode(&self, data: &mut Vec<u8>) {
		let mut tmp = Vec::new();

		self.locals.len().wasm_encode(&mut tmp);
		for local in &self.locals {
			(1u32).wasm_encode(&mut tmp);
			local.wasm_encode(&mut tmp);
		}

		self.expr.wasm_encode(&mut tmp);

		tmp.len().wasm_encode(data);
		data.extend(tmp);
	}
}

pub enum Data {
	Active(MemIdx, Expr, Vec<u8>),
	Passive(Vec<u8>)
}

impl WasmEncodable for Data {
    fn wasm_encode(&self, data: &mut Vec<u8>) {
        match self {
            Data::Active(mem, expr, bytes) => {
				if *mem == 0 {
					data.push(0x00);
				} else {
					data.push(0x02);
					mem.wasm_encode(data);
				}

				expr.wasm_encode(data);
				bytes.wasm_encode(data);
			},
            Data::Passive(bytes) => {
				data.push(0x01);
				bytes.wasm_encode(data);
			},
        }
    }
}

pub struct Module {
	types: Vec<FunctionType>,
	imports: Vec<Import>,
	functions: Vec<TypeIdx>,
	tables: Vec<TableType>,
	memories: Vec<MemType>,
	globals: Vec<Global>,
	exports: Vec<Export>,
	start: Option<FuncIdx>,
	elems: Vec<Elem>,
	code: Vec<Code>,
	data: Vec<Data>
}

impl Module {
	pub fn new() -> Module {
		Module {
			types: Vec::new(),
			imports: Vec::new(),
			functions: Vec::new(),
			tables: Vec::new(),
			memories: Vec::new(),
			globals: Vec::new(),
			exports: Vec::new(),
			start: None,
			elems: Vec::new(),
			code: Vec::new(),
			data: Vec::new()
		}
	}

	pub fn add_type(&mut self, wtype: FunctionType) -> TypeIdx {
		self.types.push(wtype);
		self.types.len() - 1
	}

	pub fn add_import(&mut self, import: Import) {
		self.imports.push(import);
	}

	pub fn add_function(&mut self, function: TypeIdx) {
		self.functions.push(function);
	}

	pub fn add_table(&mut self, table: TableType) {
		self.tables.push(table);
	}

	pub fn add_memory(&mut self, memory: MemType) {
		self.memories.push(memory);
	}

	pub fn add_global(&mut self, global: Global) -> GlobalIdx {
		self.globals.push(global);
		self.globals.len() - 1
	}

	pub fn add_export(&mut self, export: Export) {
		self.exports.push(export);
	}

	pub fn add_elem(&mut self, elem: Elem) {
		self.elems.push(elem);
	}

	pub fn add_code(&mut self, code: Code) {
		self.code.push(code);
	}

	pub fn add_data(&mut self, data: Data) {
		self.data.push(data);
	}

	fn encode_section(id: u8, sec: &impl WasmEncodable, data: &mut Vec<u8>) {
		data.push(id);
		
		let mut tmp = Vec::new();
		sec.wasm_encode(&mut tmp);

		tmp.len().wasm_encode(data);
		data.extend(tmp);
	}

	pub fn encode(&self) -> Vec<u8> {
		let mut data = vec![
			0x00, 0x61, 0x73, 0x6d,
			0x01, 0x00, 0x00, 0x00
		];

		if self.types.len() > 0 {
			Module::encode_section(1, &self.types, &mut data);
		}

		if self.imports.len() > 0 {
			Module::encode_section(2, &self.imports, &mut data);
		}

		if self.functions.len() > 0 {
			Module::encode_section(3, &self.functions, &mut data);
		}
		
		if self.tables.len() > 0 {
			Module::encode_section(4, &self.tables, &mut data);
		}

		if self.memories.len() > 0 {
			Module::encode_section(5, &self.memories, &mut data);
		}

		if self.globals.len() > 0 {
			Module::encode_section(6, &self.globals, &mut data);
		}

		if self.exports.len() > 0 {
			Module::encode_section(7, &self.exports, &mut data);
		}

		if let Some(idx) = self.start {
			Module::encode_section(8, &idx, &mut data);
		}

		if self.elems.len() > 0 {
			Module::encode_section(9, &self.elems, &mut data);
		}

		if self.code.len() > 0 {
			Module::encode_section(10, &self.code, &mut data);
		}

		if self.data.len() > 0 {
			Module::encode_section(11, &self.data, &mut data);
			// Module::encode_section(12, &self.data.len(), &mut data); // data count section
		}

		data
	}
}