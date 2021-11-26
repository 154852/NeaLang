use crate::io::BinaryWriter;
use crate::classfile::ClassFile;
use crate::instructions::Ins;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Attribute {
    Code(Code),
    LocalVariableTable(LocalVariableTable),
}

impl Attribute {
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        let (name, data) = match self {
            Attribute::Code(code) => ("Code", code.encode(class)),
            Attribute::LocalVariableTable(table) => ("LocalVariableTable", table.encode(class)),
        };

        writer.u16(class.constant_pool_index_to_encodable_index(
            class.consant_pool_index_of_str(name).expect("Could not encode class file, missing attribute name")
        ));
        writer.u32(data.len() as u32);
        writer.bytes(&data);
    }
}

#[derive(Debug)]
pub struct UnDecodedAttribute {
    name_index: usize,
    raw: Vec<u8>
}

impl UnDecodedAttribute {
    pub fn new(name_index: usize, raw: Vec<u8>) -> UnDecodedAttribute {
        UnDecodedAttribute {
            name_index, raw
        }
    }

    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_index));
        writer.u32(self.raw.len() as u32);
        writer.bytes(&self.raw);
    }
}

#[derive(Debug)]
pub struct Exception {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16
}

impl Exception {
    pub fn encode(&self, writer: &mut BinaryWriter) {
        writer.u16(self.start_pc);
        writer.u16(self.end_pc);
        writer.u16(self.handler_pc);
        writer.u16(self.catch_type);
    }
}

#[derive(Debug)]
pub struct Code {
    max_stack: u16,
    max_locals: u16,
    code: Vec<Ins>,
    exception_table: Vec<Exception>,
    attributes: Vec<Attribute>
}

impl Code {
    pub fn new(max_stack: u16, max_locals: u16, code: Vec<Ins>) -> Code {
        Code {
            max_stack, max_locals,
            code,
            exception_table: Vec::new(),
            attributes: Vec::new()
        }
    }

    pub fn encode(&self, class: &ClassFile) -> Vec<u8> {
        let mut writer = BinaryWriter::new();

        writer.u16(self.max_stack);
        writer.u16(self.max_locals);

        let mut code_writer = BinaryWriter::new();
        for ins in &self.code {
            ins.encode(&mut code_writer, class);
        }
        let code = code_writer.take();
        writer.u32(code.len() as u32);
        writer.bytes(&code);

        writer.u16(self.exception_table.len() as u16);
        for exception in &self.exception_table {
            exception.encode(&mut writer);
        }

        writer.u16(self.attributes.len() as u16);
        for attr in &self.attributes {
            attr.encode(&mut writer, class);
        }

        writer.take()
    }

    pub fn stack_size(&self) -> u16 {
        self.max_stack
    }

    pub fn locals_size(&self) -> u16 {
        self.max_locals
    }

    pub fn instructions(&self) -> &Vec<Ins> {
        &self.code
    }
}

#[derive(Debug)]
struct LocalVariableEntry {
    start_pc: u16,
    length: u16,
    name_index: usize,
    descriptor_index: usize,
    index: u16
}

impl LocalVariableEntry {
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(self.start_pc);
        writer.u16(self.length);
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.descriptor_index));
        writer.u16(self.index);
    }
}

#[derive(Debug)]
pub struct LocalVariableTable {
    entries: Vec<LocalVariableEntry>
}

impl LocalVariableTable {
    pub fn encode(&self, class: &ClassFile) -> Vec<u8> {
        let mut writer = BinaryWriter::new();

        writer.u16(self.entries.len() as u16);

        for entry in &self.entries {
            entry.encode(&mut writer, class);
        }

        writer.take()
    }
}