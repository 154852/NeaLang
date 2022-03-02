use std::collections::HashMap;
use ir;
use syntax::Span;

/// Represents a specific kind of error, and any necessary metadata it needs to show a nice error message
#[derive(Debug)]
pub enum IrGenErrorKind {
    UnknownType,
    VariableDoesNotExist(String), // Variable name
    FunctionDoesNotExist(String), // Function name
    MethodNotStatic,
    InvalidInteger,
    BinaryOpTypeMismatch,
    AssignmentTypeMismatch,
    CannotInferType,
    CallArgParamCountMismatch(usize, usize), // Found, expected
    CallArgTypeMismatch(String, String), // Expected type, found type
    CallNotOneReturnInExpr,
    InvalidLHS,
    InvalidRHS,
    CompositeTypeOnStack,
    PropDoesNotExist(String, String), // Prop name, type name
    IllegalIndexObject,
    IllegalIndexValue,
    NonValueCast,
    StdLinkError,
    UnknownAnnotation(String), // Annotation name
    InvalidAnnotationExpression(String), // Annotation string
    NonConstExprInSlice,
    InvalidDropType(String), // Type name
    NotABool,
    NoReturnValue,
    ReturnValueWhenVoid,
    IncorrectReturnType(String, String)
}

pub struct IrGenError {
    span: Span,
    kind: IrGenErrorKind
}

impl IrGenError {
    pub fn new(span: Span, kind: IrGenErrorKind) -> IrGenError {
        IrGenError {
            span, kind
        }
    }

    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }

    pub fn message(&self) -> String {
        match &self.kind {
            IrGenErrorKind::UnknownType => format!("Unknown type"),
            IrGenErrorKind::VariableDoesNotExist(name) => format!("Variable '{}' does not exist", name),
            IrGenErrorKind::FunctionDoesNotExist(name) => format!("Function or method '{}' does not exist", name),
            IrGenErrorKind::InvalidInteger => format!("Invalid integer"),
            IrGenErrorKind::BinaryOpTypeMismatch => format!("Type mismatch in binary operation"),
            IrGenErrorKind::AssignmentTypeMismatch => format!("Type mismatch in assignment"),
            IrGenErrorKind::CannotInferType => format!("Cannot infer type"),
            IrGenErrorKind::CallArgParamCountMismatch(found, expected) => format!("Incorrect arg count, found {}, expected {}", found, expected),
            IrGenErrorKind::CallArgTypeMismatch(found, expected) => format!("Type mismatch in arg, found {}, expected {}", found, expected),
            IrGenErrorKind::CallNotOneReturnInExpr => format!("Call to function returning not one value"),
            IrGenErrorKind::InvalidLHS => format!("Invalid lhs expression"),
            IrGenErrorKind::InvalidRHS => format!("Invalid rhs expression"),
            IrGenErrorKind::CompositeTypeOnStack => format!("Composite type in expression"),
            IrGenErrorKind::PropDoesNotExist(prop, type_name) => format!("Property '{}' does not exist on {} type", prop, type_name),
            IrGenErrorKind::IllegalIndexObject => format!("Not a slice, cannot index"),
            IrGenErrorKind::IllegalIndexValue => format!("Can only index with a uptr"),
            IrGenErrorKind::NonValueCast => format!("Cannot cast to non-value type"),
            IrGenErrorKind::StdLinkError => format!("Not linked with std, try importing std"),
            IrGenErrorKind::UnknownAnnotation(name) => format!("Unknown annotation '{}'", name),
            IrGenErrorKind::InvalidAnnotationExpression(name) => format!("Invalid annotation, expected {}", name),
            IrGenErrorKind::MethodNotStatic => format!("Method is not static"),
            IrGenErrorKind::NonConstExprInSlice => format!("Slice literals can only take compile time known expressions"),
            IrGenErrorKind::InvalidDropType(name) => format!("Cannot drop value of type {}", name),
            IrGenErrorKind::NotABool => format!("Expected a bool for condition"),
            IrGenErrorKind::NoReturnValue => format!("Expected a value in return"),
            IrGenErrorKind::ReturnValueWhenVoid => format!("No value expected in return for void function"),
            IrGenErrorKind::IncorrectReturnType(found, expected) => format!("Return type mismatch, found {}, expected {}", found, expected)
        }
    }
}

/// Convert a storable type to a user displayable name
pub fn storable_type_to_string(st: &ir::StorableType) -> String {
    match st {
        ir::StorableType::Compound(ct) => ct.name().to_string(),
        ir::StorableType::Value(v) => value_type_to_string(v),
        ir::StorableType::Slice(slice_type) => {
            let mut s = storable_type_to_string(slice_type);
            s.push_str("[]");
            s
        },
        ir::StorableType::SliceData(_) => unreachable!(),
    }
}

/// Convert a value type to a user displayable name
pub fn value_type_to_string(vt: &ir::ValueType) -> String {
    match vt {
        ir::ValueType::U8 => "u8".to_string(),
        ir::ValueType::I8 => "i8".to_string(),
        ir::ValueType::U16 => "u16".to_string(),
        ir::ValueType::I16 => "i16".to_string(),
        ir::ValueType::U32 => "u32".to_string(),
        ir::ValueType::I32 => "i32".to_string(),
        ir::ValueType::U64 => "u64".to_string(),
        ir::ValueType::I64 => "i64".to_string(),
        ir::ValueType::UPtr => "uptr".to_string(),
        ir::ValueType::IPtr => "iptr".to_string(),
        ir::ValueType::Bool => "bool".to_string(),
        ir::ValueType::Ref(st) => storable_type_to_string(st),
        ir::ValueType::Index(_) => "uptr".to_string(),
    }
}

/// Represents the function level context while generating IR, is aware of locals (and their names), and which function this is
pub struct IrGenFunctionContext<'a> {
    pub ir_unit: &'a mut ir::TranslationUnit,
    pub function_idx: ir::FunctionIndex,

    pub local_map: HashMap<&'a str, ir::LocalIndex>
}

impl<'a> IrGenFunctionContext<'a> {
    pub fn func(&self) -> &ir::Function {
        self.ir_unit.get_function(self.function_idx).unwrap()
    }

    pub fn func_mut(&mut self) -> &mut ir::Function {
        self.ir_unit.get_function_mut(self.function_idx).unwrap()
    }

    pub fn push_local(&mut self, name: &'a str, st: ir::StorableType) -> ir::LocalIndex {
        let idx = self.func_mut().push_local(ir::Local::new(st));
        self.local_map.insert(name, idx);

        idx
    }
}

/// A generic target for IR code - acts as a bridge to a Vec<Ins> for now, but may carry more information on the current block in future
pub struct IrGenCodeTarget {
    ins: Vec<ir::Ins>
}

impl IrGenCodeTarget {
    pub fn new() -> IrGenCodeTarget {
        IrGenCodeTarget {
            ins: Vec::new()
        }
    }

    pub fn push(&mut self, ins: ir::Ins) {
        self.ins.push(ins);
    }

    pub fn take(self) -> Vec<ir::Ins> {
        self.ins
    }
}