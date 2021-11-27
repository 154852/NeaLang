use crate::{Class, FieldRef, InnerClass, InnerClasses, MethodRef, NameAndType, Utf8, attribute::{self, Attribute}, constantpool::{self, Constant}, io::BinaryWriter};

#[derive(Debug)]
pub struct ClassAccessFlags(u16);

impl ClassAccessFlags {
    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_SUPER: u16 = 0x0020;
    pub const ACC_INTERFACE: u16 = 0x0200;
    pub const ACC_ABSTRACT: u16 = 0x0400;
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    pub const ACC_ANNOTATION: u16 = 0x2000;
    pub const ACC_ENUM: u16 = 0x4000;
    
    pub fn from_bits(bits: u16) -> ClassAccessFlags {
        ClassAccessFlags(bits)
    }

    pub fn bits(&self) -> u16 {
        self.0
    }
}

pub struct ClassFile {
    version_minor: u16, version_major: u16,
    access_flags: ClassAccessFlags,

    constant_pool: Vec<Constant>,

    this_index: usize,
    super_index: Option<usize>,

    interfaces: Vec<usize>,
    fields: Vec<Field>,
    methods: Vec<Method>,

    attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn new<T: Into<String>>(name: T) -> ClassFile {
        let mut cf = ClassFile {
            version_minor: 0,
            version_major: 55,
            access_flags: ClassAccessFlags(ClassAccessFlags::ACC_PUBLIC),
            constant_pool: Vec::new(),
            this_index: 0,
            super_index: None,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new()
        };

        let this_name = cf.add_constant(Constant::Utf8(constantpool::Utf8::new(name)));
        let this = cf.add_constant(Constant::Class(constantpool::Class::new(this_name)));
        cf.this_index = this;

        let super_name = cf.add_constant(Constant::Utf8(constantpool::Utf8::new("java/lang/Object")));
        let super_ = cf.add_constant(Constant::Class(constantpool::Class::new(super_name)));
        cf.super_index = Some(super_);

        cf.add_constant(Constant::Utf8(constantpool::Utf8::new("Code"))); // Add for later use
        cf.add_constant(Constant::Utf8(constantpool::Utf8::new("StackMapTable")));

        cf
    }

    pub fn name(&self) -> &str {
        match self.constant_pool.get(self.this_index) {
            Some(Constant::Class(cl)) =>
                match self.constant_pool.get(cl.name()) {
                    Some(Constant::Utf8(u)) => u.get_str(),
                    _ => panic!("Invalid class reference")
                },
            _ => panic!("Invalid class reference")
        }
    }

    pub fn this_index(&self) -> usize {
        self.this_index
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        self.constant_pool.push(constant);
        self.constant_pool.len() - 1
    }

    pub fn add_inner_class(&mut self, class: InnerClass) {
        for attr in &mut self.attributes {
            if let Attribute::InnerClasses(inner) = attr {
                inner.add_entry(class);
                return;
            }
        }

        let mut inner_classes = InnerClasses::new();
        inner_classes.add_entry(class);
        self.attributes.push(Attribute::InnerClasses(inner_classes));
        self.add_constant(Constant::Utf8(constantpool::Utf8::new("InnerClasses")));
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut writer = BinaryWriter::new();

        writer.u32(0xCAFEBABE);

        writer.u16(self.version_minor);
        writer.u16(self.version_major);
        
        writer.u16(self.constant_pool_index_span() as u16 + 1);
        for constant in &self.constant_pool {
            constant.encode(&mut writer, self);
        }

        writer.u16(self.access_flags.0);

        writer.u16(self.constant_pool_index_to_encodable_index(self.this_index));
        writer.u16(match self.super_index {
            None => 0,
            Some(index) => self.constant_pool_index_to_encodable_index(index)
        });

        writer.u16(self.interfaces.len() as u16);
        for interface in &self.interfaces {
            writer.u16(self.constant_pool_index_to_encodable_index(*interface));
        }

        writer.u16(self.fields.len() as u16);
        for field in &self.fields {
            field.encode(&mut writer, self);
        }

        writer.u16(self.methods.len() as u16);
        for method in &self.methods {
            method.encode(&mut writer, self);
        }

        writer.u16(self.attributes.len() as u16);
        for attr in &self.attributes {
            attr.encode(&mut writer, self);
        }

        writer.take()
    }

    pub fn consant_pool_index_of_str(&self, s: &str) -> Option<usize> {
        for (c, constant) in self.constant_pool.iter().enumerate() {
            if matches!(constant, Constant::Utf8(utf8) if utf8.get_str() == s) {
                return Some(c);
            }
        }

        None
    }

    pub fn const_matches_str(&self, idx: usize, utf8: &str) -> bool {
        return matches!(self.constant_pool.get(idx), Some(Constant::Utf8(u)) if u.get_str() == utf8);
    }

    pub fn const_name_and_type(&mut self, name: &str, desc: &str) -> usize {
        for (c, constant) in self.constant_pool.iter().enumerate() {
            if matches!(constant, Constant::NameAndType(nt) if self.const_matches_str(nt.name(), name) && self.const_matches_str(nt.desc(), desc)) {
                return c;
            }
        }

        let name = self.const_str(name);
        let desc = self.const_str(desc);
        self.constant_pool.push(Constant::NameAndType(NameAndType::new(name, desc)));
        self.constant_pool.len() - 1
    }

    pub fn const_class(&mut self, name: &str) -> usize {
        for (c, constant) in self.constant_pool.iter().enumerate() {
            if matches!(constant, Constant::Class(c) if self.const_matches_str(c.name(), name)) {
                return c;
            }
        }

        let name = self.const_str(name);
        self.constant_pool.push(Constant::Class(Class::new(name)));
        self.constant_pool.len() - 1
    }

    pub fn const_field(&mut self, class: &str, name: &str, desc: &str) -> usize {
        let name_and_type = self.const_name_and_type(name, desc);
        let class = self.const_class(class);

        for (c, constant) in self.constant_pool.iter().enumerate() {
            if matches!(constant, Constant::FieldRef(c) if c.class() == class && c.name_and_type() == name_and_type) {
                return c;
            }
        }

        self.constant_pool.push(Constant::FieldRef(FieldRef::new(class, name_and_type)));
        self.constant_pool.len() - 1
    }

    pub fn const_method(&mut self, class: &str, name: &str, desc: &str) -> usize {
        let name_and_type = self.const_name_and_type(name, desc);
        let class = self.const_class(class);

        for (c, constant) in self.constant_pool.iter().enumerate() {
            if matches!(constant, Constant::MethodRef(c) if c.class() == class && c.name_and_type() == name_and_type) {
                return c;
            }
        }

        self.constant_pool.push(Constant::MethodRef(MethodRef::new(class, name_and_type)));
        self.constant_pool.len() - 1
    }

    pub fn const_str(&mut self, s: &str) -> usize {
        if let Some(idx) = self.consant_pool_index_of_str(s) {
            return idx;
        }

        self.constant_pool.push(Constant::Utf8(Utf8::new(s)));
        self.constant_pool.len() - 1
    }

    pub fn constant_pool_index_to_encodable_index(&self, idx: usize) -> u16 {
        let mut size = 0;
        for constant in &self.constant_pool[0..idx] {
            size += constant.index_span() as u16;
        }
        size + 1
    }

    pub fn constant_pool_index_span(&self) -> usize {
        let mut size = 0;
        for constant in &self.constant_pool {
            size += constant.index_span();
        }
        size
    }
}

pub struct FieldAccessFlags(u16);

impl FieldAccessFlags {
    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_VOLATILE: u16 = 0x0040;
    pub const ACC_TRANSIENT: u16 = 0x0080;
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    pub const ACC_ENUM: u16 = 0x4000;
    
    pub fn from_bits(bits: u16) -> FieldAccessFlags {
        FieldAccessFlags(bits)
    }
}

pub struct Field {
    access_flags: FieldAccessFlags,
    name_index: usize,
    descriptor_index: usize,
    attributes: Vec<Attribute>
}

impl Field {
    pub fn new_on<T: Into<String>, U: Into<String>>(name: T, descriptor: U, class: &mut ClassFile) -> &mut Field {
        let name_index = class.add_constant(Constant::Utf8(constantpool::Utf8::new(name)));
        let descriptor_index = class.add_constant(Constant::Utf8(constantpool::Utf8::new(descriptor)));

        let field = Field {
            name_index, descriptor_index,
            access_flags: FieldAccessFlags::from_bits(FieldAccessFlags::ACC_PUBLIC),
            attributes: Vec::new()
        };

        class.fields.push(field);

        class.fields.last_mut().unwrap()
    }

    pub fn set_access(&mut self, flags: FieldAccessFlags) {
        self.access_flags = flags;
    }

    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(self.access_flags.0);
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.descriptor_index));
        
        writer.u16(self.attributes.len() as u16);
        for attr in &self.attributes {
            attr.encode(writer, class);
        }
    }
}

pub struct MethodAccessFlags(u16);

impl MethodAccessFlags {
    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_SYNCHRONIZED: u16 = 0x0020;
    pub const ACC_BRIDGE: u16 = 0x0040;
    pub const ACC_NATIVE: u16 = 0x0100;
    pub const ACC_ABSTRACT: u16 = 0x0400;
    pub const ACC_STRICT: u16 = 0x0800;
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    
    pub fn from_bits(bits: u16) -> MethodAccessFlags {
        MethodAccessFlags(bits)
    }
}

pub struct Method {
    access_flags: MethodAccessFlags,
    name_index: usize,
    descriptor_index: usize,
    attributes: Vec<Attribute>
}

impl Method {
    pub fn new_on<T: Into<String>, U: Into<String>>(name: T, descriptor: U, class: &mut ClassFile) -> &mut Method {
        let name_index = class.add_constant(Constant::Utf8(constantpool::Utf8::new(name)));
        let descriptor_index = class.add_constant(Constant::Utf8(constantpool::Utf8::new(descriptor)));

        let method = Method {
            name_index, descriptor_index,
            access_flags: MethodAccessFlags::from_bits(MethodAccessFlags::ACC_PUBLIC),
            attributes: Vec::new()
        };

        class.methods.push(method);

        class.methods.last_mut().unwrap()
    }

    pub fn set_access(&mut self, flags: MethodAccessFlags) {
        self.access_flags = flags;
    }

    pub fn add_code(&mut self, code: attribute::Code) {
        self.attributes.push(Attribute::Code(code));
    }

    fn encode(&self, writer: &mut BinaryWriter, class: &ClassFile) {
        writer.u16(self.access_flags.0);
        writer.u16(class.constant_pool_index_to_encodable_index(self.name_index));
        writer.u16(class.constant_pool_index_to_encodable_index(self.descriptor_index));
        
        writer.u16(self.attributes.len() as u16);
        for attr in &self.attributes {
            attr.encode(writer, class);
        }
    }
}