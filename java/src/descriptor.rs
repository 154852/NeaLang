#[derive(Debug)]
pub enum Descriptor {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Reference(String),
    Short,
    Boolean,
    Array(usize, Box<Descriptor>),
    Void
}

impl Descriptor {
    pub fn to_string(&self) -> String {
        match self {
            Descriptor::Byte => "B".to_string(),
            Descriptor::Char => "C".to_string(),
            Descriptor::Double => "D".to_string(),
            Descriptor::Float => "D".to_string(),
            Descriptor::Int => "I".to_string(),
            Descriptor::Long => "J".to_string(),
            Descriptor::Reference(name) => format!("L{};", name),
            Descriptor::Short => "S".to_string(),
            Descriptor::Boolean => "Z".to_string(),
            Descriptor::Array(d, r) => {
                let mut s = r.to_string();
                for _ in 0..*d {
                    s.insert(0, '[');
                }
                s
            },
            Descriptor::Void => "V".to_string(),
        }
    }

    pub fn function_to_string(params: &Vec<Descriptor>, return_value: &Descriptor) -> String {
        let mut string = String::from("(");
        for param in params {
            string.push_str(&param.to_string());
        }
        string.push(')');
        string.push_str(&return_value.to_string());
        string
    }
}