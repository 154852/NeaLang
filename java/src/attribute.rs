use crate::{ClassAccessFlags, Descriptor};
use crate::io::BinaryWriter;
use crate::classfile::ClassFile;
use crate::instructions::Ins;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Attribute {
    Code(Code),
    LocalVariableTable(LocalVariableTable),
    InnerClasses(InnerClasses),
    StackMapTable(StackMapTable)
}

impl Attribute {
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        let (name, data) = match self {
            Attribute::Code(code) => ("Code", code.encode(class)),
            Attribute::LocalVariableTable(table) => ("LocalVariableTable", table.encode(class)),
            Attribute::InnerClasses(classes) => ("InnerClasses", classes.encode(class)),
            Attribute::StackMapTable(table) => ("StackMapTable", table.encode(class)),
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

    pub fn add_map(&mut self, map: StackMapTable) {
        self.attributes.push(Attribute::StackMapTable(map));
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

#[derive(Debug)]
pub struct InnerClass {
    inner_class_index: usize,
    outer_class_index: usize,
    inner_name_index: usize,
    access_flags: ClassAccessFlags
}

impl InnerClass {
    pub fn new(inner_class_index: usize, outer_class_index: usize, inner_name_index: usize) -> InnerClass {
        InnerClass {
            inner_class_index,
            outer_class_index,
            inner_name_index,
            access_flags: ClassAccessFlags::from_bits(ClassAccessFlags::ACC_PUBLIC | ClassAccessFlags::ACC_STATIC)
        }
    }
    
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.inner_class_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.outer_class_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.inner_name_index));
        writer.u16(self.access_flags.bits());
    }
}

#[derive(Debug)]
pub struct InnerClasses {
    entries: Vec<InnerClass>
}

impl InnerClasses {
    pub fn new() -> InnerClasses {
        InnerClasses {
            entries: Vec::new()
        }
    }

    pub fn encode(&self, class: &ClassFile) -> Vec<u8> {
        let mut writer = BinaryWriter::new();

        writer.u16(self.entries.len() as u16);

        for entry in &self.entries {
            entry.encode(&mut writer, class);
        }

        writer.take()
    }

    pub fn add_entry(&mut self, class: InnerClass) {
        self.entries.push(class);
    }
}

#[derive(Debug)]
pub struct StackMapTable {
    frames: Vec<StackMapFrame>
}

impl StackMapTable {
    pub fn new() -> StackMapTable {
        StackMapTable {
            frames: Vec::new()
        }
    }

    pub fn encode(&self, class: &ClassFile) -> Vec<u8> {
        let mut writer = BinaryWriter::new();

        writer.u16(self.frames.len() as u16);

        for entry in &self.frames {
            entry.encode(&mut writer, class);
        }

        writer.take()
    }

    pub fn add_entry(&mut self, class: StackMapFrame) {
        self.frames.push(class);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    Object(usize)
}

impl VerificationTypeInfo {
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        match self {
            VerificationTypeInfo::Top => writer.u8(0),
            VerificationTypeInfo::Integer => writer.u8(1),
            VerificationTypeInfo::Float => writer.u8(2),
            VerificationTypeInfo::Long => writer.u8(4),
            VerificationTypeInfo::Double => writer.u8(3),
            VerificationTypeInfo::Null => writer.u8(5),
            VerificationTypeInfo::Object(idx) => {
                writer.u8(7);
                writer.u16(class.constant_pool_index_to_encodable_index(*idx));
            },
        }
    }

    pub fn from_descriptor(desc: &Descriptor, class: &mut ClassFile) -> VerificationTypeInfo {
        match desc {
            Descriptor::Byte => VerificationTypeInfo::Integer,
            Descriptor::Char => VerificationTypeInfo::Integer,
            Descriptor::Double => VerificationTypeInfo::Double,
            Descriptor::Float => VerificationTypeInfo::Float,
            Descriptor::Int => VerificationTypeInfo::Integer,
            Descriptor::Long => VerificationTypeInfo::Long,
            Descriptor::Reference(name) => VerificationTypeInfo::Object(class.const_class(name)),
            Descriptor::Short => VerificationTypeInfo::Integer,
            Descriptor::Boolean => VerificationTypeInfo::Integer,
            Descriptor::Array(_, _) => VerificationTypeInfo::Object(class.const_class(&desc.to_string())),
            Descriptor::Void => panic!("void can not be a verification type"),
        }
    }
}

#[derive(Debug)]
pub enum StackMapFrame {
    SameFrameExtended { offset: u16 },
    SameLocalsOneStackExtended { offset: u16, stack: VerificationTypeInfo },
    AppendFrame { offset: u16, locals: Vec<VerificationTypeInfo> },
    FullFrame { offset: u16, locals: Vec<VerificationTypeInfo>, stack: Vec<VerificationTypeInfo> }
}

impl StackMapFrame {
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        match self {
            StackMapFrame::SameFrameExtended { offset } => {
                writer.u8(251);
                writer.u16(*offset);
            },
            StackMapFrame::SameLocalsOneStackExtended { offset, stack } => {
                writer.u8(247);
                writer.u16(*offset);
                stack.encode(writer, class);
            },
            StackMapFrame::AppendFrame { offset, locals } => {
                assert!(locals.len() <= 3);
                writer.u8(251 + locals.len() as u8);
                writer.u16(*offset);
                for local in locals { local.encode(writer, class); }
            },
            StackMapFrame::FullFrame { offset, locals, stack } => {
                writer.u8(255);
                writer.u16(*offset);
                writer.u16(locals.len() as u16);
                for local in locals { local.encode(writer, class); }
                writer.u16(stack.len() as u16);
                for stack_el in stack { stack_el.encode(writer, class); }
            }
        }
    }
}