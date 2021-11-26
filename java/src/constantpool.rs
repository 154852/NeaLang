use crate::{ClassFile, io::BinaryWriter};

#[derive(Debug)]
pub enum Constant {
    Class(Class),
    FieldRef(FieldRef),
    MethodRef(MethodRef),
    InterfaceMethodRef(InterfaceMethodRef),
    String(JavaString),
    Integer(Integer),
    Float(Float),
    Long(Long),
    Double(Double),
    NameAndType(NameAndType),
    Utf8(Utf8),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    InvokeDynamic(InvokeDynamic)
}

const CLASS_CONSTANT_ID: u8 = 7;
const FIELD_REF_CONSTANT_ID: u8 = 9;
const METHOD_REF_CONSTANT_ID: u8 = 10;
const INTERFACE_METHOD_REF_CONSTANT_ID: u8 = 11;
const STRING_CONSTANT_ID: u8 = 8;
const INTEGER_CONSTANT_ID: u8 = 3;
const FLOAT_CONSTANT_ID: u8 = 4;
const LONG_CONSTANT_ID: u8 = 5;
const DOUBLE_CONSTANT_ID: u8 = 6;
const NAME_AND_TYPE_CONSTANT_ID: u8 = 12;
const UTF8_CONSTANT_ID: u8 = 1;
const METHOD_HANDLE_CONSTANT_ID: u8 = 15;
const METHOD_TYPE_CONSTANT_ID: u8 = 16;
const INVOKE_DYNAMIC_CONSTANT_ID: u8 = 18;

impl Constant {
    pub fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        match self {
            Constant::Class(class_const) => {
                writer.u8(CLASS_CONSTANT_ID);
                class_const.encode(writer, class);
            },
            Constant::FieldRef(field) => {
                writer.u8(FIELD_REF_CONSTANT_ID);
                field.encode(writer, class);
            },
            Constant::MethodRef(method) => {
                writer.u8(METHOD_REF_CONSTANT_ID);
                method.encode(writer, class);
            },
            Constant::InterfaceMethodRef(int) => {
                writer.u8(INTERFACE_METHOD_REF_CONSTANT_ID);
                int.encode(writer, class);
            },
            Constant::String(str) => {
                writer.u8(STRING_CONSTANT_ID);
                str.encode(writer, class);
            },
            Constant::Integer(int) => {
                writer.u8(INTEGER_CONSTANT_ID);
                int.encode(writer, class);
            },
            Constant::Float(float) => {
                writer.u8(FLOAT_CONSTANT_ID);
                float.encode(writer, class);
            },
            Constant::Long(long) => {
                writer.u8(LONG_CONSTANT_ID);
                long.encode(writer, class);
            },
            Constant::Double(double) => {
                writer.u8(DOUBLE_CONSTANT_ID);
                double.encode(writer, class);
            },
            Constant::NameAndType(nt) => {
                writer.u8(NAME_AND_TYPE_CONSTANT_ID);
                nt.encode(writer, class);
            },
            Constant::Utf8(utf8) => {
                writer.u8(UTF8_CONSTANT_ID);
                utf8.encode(writer, class);
            },
            Constant::MethodHandle(method) => {
                writer.u8(METHOD_HANDLE_CONSTANT_ID);
                method.encode(writer, class);
            },
            Constant::MethodType(mt) => {
                writer.u8(METHOD_TYPE_CONSTANT_ID);
                mt.encode(writer, class);
            },
            Constant::InvokeDynamic(id) => {
                writer.u8(INVOKE_DYNAMIC_CONSTANT_ID);
                id.encode(writer, class);
            },
        }
    }

    pub fn index_span(&self) -> usize {
        match self {
            Constant::Long(_) => 2,
            Constant::Double(_) => 2,
            _ => 1
        }
    }
}

#[derive(Debug)]
pub struct Class {
    pub name_index: usize
}

impl Class {
    pub fn new(name_index: usize) -> Class {
        Class {
            name_index
        }
    }

    pub fn name(&self) -> usize {
        self.name_index
    }
    
    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_index));
    }
}

#[derive(Debug)]
pub struct FieldRef {
    pub class_index: usize,
    pub name_and_type_index: usize
}

impl FieldRef {
    pub fn new(class_index: usize, name_and_type_index: usize) -> FieldRef {
        FieldRef {
            class_index, name_and_type_index
        }
    }

    pub fn class(&self) -> usize {
        self.class_index
    }

    pub fn name_and_type(&self) -> usize {
        self.name_and_type_index
    }
    
    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.class_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_and_type_index));
    }
}

#[derive(Debug)]
pub struct MethodRef {
    pub class_index: usize,
    pub name_and_type_index: usize
}

impl MethodRef {
    pub fn new(class_index: usize, name_and_type_index: usize) -> MethodRef {
        MethodRef {
            class_index, name_and_type_index
        }
    }

    pub fn class(&self) -> usize {
        self.class_index
    }

    pub fn name_and_type(&self) -> usize {
        self.name_and_type_index
    }

    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.class_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_and_type_index));
    }
}

#[derive(Debug)]
pub struct InterfaceMethodRef {
    pub class_index: usize,
    pub name_and_type_index: usize
}

impl InterfaceMethodRef {
    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.class_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_and_type_index));
    }
}

#[derive(Debug)]
pub struct JavaString {
    pub utf8_index: usize
}

impl JavaString {
    pub fn new(utf8_index: usize) -> JavaString {
        JavaString {
            utf8_index
        }
    }

    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.utf8_index));
    }
}

#[derive(Debug)]
pub struct Integer {
    pub value: u32
}

impl Integer {
    fn encode(&self, writer: &mut BinaryWriter, _class: &ClassFile) {
        writer.u32(self.value);
    }
}

#[derive(Debug)]
pub struct Float {
    pub value: f32
}

impl Float {
    fn encode(&self, writer: &mut BinaryWriter, _class: &ClassFile) {
        writer.f32(self.value);
    }
}

#[derive(Debug)]
pub struct Long {
    pub value: u64
}

impl Long {
    fn encode(&self, writer: &mut BinaryWriter, _class: &ClassFile) {
        writer.u64(self.value);
    }
}

#[derive(Debug)]
pub struct Double {
    pub value: f64
}

impl Double {
    fn encode(&self, writer: &mut BinaryWriter, _class: &ClassFile) {
        writer.f64(self.value);
    }
}

#[derive(Debug)]
pub struct NameAndType {
    pub name_index: usize,
    pub descriptor_index: usize
}

impl NameAndType {
    pub fn new(name_index: usize, descriptor_index: usize) -> NameAndType {
        NameAndType {
            name_index, descriptor_index
        }
    }

    pub fn name(&self) -> usize {
        self.name_index
    }

    pub fn desc(&self) -> usize {
        self.descriptor_index
    }

    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.descriptor_index));
    }
}

#[derive(Debug)]
pub struct Utf8 {
    pub string: String
}

impl Utf8 {
    pub fn new<T: Into<String>>(string: T) -> Utf8 {
        Utf8 {
            string: string.into()
        }
    }

    fn encode(&self, writer: &mut BinaryWriter, _class: &ClassFile) {
        writer.u16(self.string.len() as u16);
        writer.bytes(self.string.as_bytes());
    }

    pub fn get_str(&self) -> &str {
        &self.string
    }
}

#[derive(Debug)]
pub enum ReferenceKind {
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    InvokeVirtual,
    InvokeStatic,
    InvokeSpecial,
    NewInvokeSpecial,
    InvokeInterface
}

impl ReferenceKind {
    pub fn to_id(&self) -> u8 {
        match self {
            ReferenceKind::GetField => 1,
            ReferenceKind::GetStatic => 2,
            ReferenceKind::PutField => 3,
            ReferenceKind::PutStatic => 4,
            ReferenceKind::InvokeVirtual => 5,
            ReferenceKind::InvokeStatic => 6,
            ReferenceKind::InvokeSpecial => 7,
            ReferenceKind::NewInvokeSpecial => 8,
            ReferenceKind::InvokeInterface => 9,
        }
    }
}

#[derive(Debug)]
pub struct MethodHandle {
    pub reference_kind: ReferenceKind,
    pub reference_index: usize
}

impl MethodHandle {
    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u8(self.reference_kind.to_id());
        writer.u16(class.constant_pool_index_to_encodable_index(self.reference_index));
    }
}

#[derive(Debug)]
pub struct MethodType {
    pub descriptor_index: usize
}

impl MethodType {
    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.descriptor_index));
    }
}

#[derive(Debug)]
pub struct InvokeDynamic {
    pub bootstrap_method_attr_index: usize,
    pub name_and_type_index: usize
}

impl InvokeDynamic {
    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(class.constant_pool_index_to_encodable_index(self.bootstrap_method_attr_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_and_type_index));
    }
}