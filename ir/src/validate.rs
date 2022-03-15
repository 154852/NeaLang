use crate::{Function, Ins, StorableType, TranslationUnit, CompoundContent, ValuePath, ValuePathComponent, ValuePathOrigin, ValueType};

macro_rules! pop {
    ($stack:expr, $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        match $stack.pop()? {
            $( $pattern )|+ $( if $guard )? => {},
            _ => return Err(ValidationError::StackIncorrectType)
        };
    };
    ($stack:expr, = $e:expr) => {
        if $stack.pop()? != $e {
            return Err(ValidationError::StackIncorrectType);
        }
    };
}

macro_rules! pop_path {
    ($stack:expr, $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        match $stack.pop_path()? {
            $( $pattern )|+ $( if $guard )? => {},
            _ => return Err(ValidationError::StackIncorrectType)
        };
    };
    ($stack:expr, = $e:expr) => {
        if $stack.pop_path()? != $e {
            return Err(ValidationError::StackIncorrectType);
        }
    };
}

macro_rules! peek {
    ($stack:expr, $idx:expr, $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        match $stack.peek($idx)? {
            $( $pattern )|+ $( if $guard )? => {},
            _ => return Err(ValidationError::StackIncorrectType)
        };
    };
    ($stack:expr, $idx:expr, = $e:expr) => {
        if $stack.peek($idx)? != $e {
            return Err(ValidationError::StackIncorrectType);
        }
    };
}

#[derive(Debug)]
enum ValueOrPath {
    Value(ValueType),
    Path(ValueType)
}

struct TypeStack {
    types: Vec<ValueOrPath>
}

impl TypeStack {
    fn new() -> TypeStack {
        TypeStack {
            types: Vec::new()
        }
    }

    fn push(&mut self, value_type: ValueType) {
        self.types.push(ValueOrPath::Value(value_type));
    }

    fn peek(&self, index: usize) -> Result<&ValueType, ValidationError> {
        match self.types.get(self.types.len() - 1 - index) {
            Some(ValueOrPath::Value(v)) => Ok(v),
            Some(ValueOrPath::Path(_)) => Err(ValidationError::StackIncorrectType),
            None => Err(ValidationError::StackUnderflow)
        }
    }

    fn pop(&mut self) -> Result<ValueType, ValidationError> {
        match self.types.pop() {
            Some(ValueOrPath::Value(v)) => Ok(v),
            Some(ValueOrPath::Path(_)) => Err(ValidationError::StackIncorrectType),
            None => Err(ValidationError::StackUnderflow)
        }
    }

    fn push_path(&mut self, path_value: ValueType) {
        self.types.push(ValueOrPath::Path(path_value));
    }

    fn pop_path(&mut self) -> Result<ValueType, ValidationError> {
        match self.types.pop() {
            Some(ValueOrPath::Path(v)) => Ok(v),
            Some(ValueOrPath::Value(_)) => Err(ValidationError::StackIncorrectType),
            None => Err(ValidationError::StackUnderflow)
        }
    }

    fn depth(&self) -> usize {
        self.types.len()
    }
}

enum BlockElement {
    Loop,
    If,
    IfElse
}

struct BlockStack {
    elements: Vec<BlockElement>
}

impl BlockStack {
    fn new() -> BlockStack {
        BlockStack {
            elements: Vec::new()
        }
    }

    fn with<T>(&mut self, element: BlockElement, mut cb: T) -> Result<(), ValidationError> where T: FnMut(&mut BlockStack) -> Result<(), ValidationError> {
        self.elements.push(element);
        let res = cb(self);
        self.elements.pop();

        res
    }

    fn is_breakable(&self, index: usize) -> bool {
        match self.elements.get(self.elements.len() - 1 - index) {
            Some(BlockElement::Loop) => true,
            _ => false
        }
    }

    fn is_continuable(&self, index: usize) -> bool {
        match self.elements.get(self.elements.len() - 1 - index) {
            Some(BlockElement::Loop) => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    StackUnderflow,
    StackIncorrectType,
    StackNotNum,
    StackDepthNotZero,
    StackDepthNotOne,
    StackNotValue,
    PathIncorrectType,
    PathNotValue,
    LocalDoesNotExist,
    LocalIncorrectType,
    LocalUnderflow,
    GlobalIncorrectType,
    PropertyIncorrectType,
    PropertyDoesNotExist,
    GlobalDoesNotExist,
    FunctionDoesNotExist,
    NotBreakable,
    NotContinuable,
    NoFinalReturn,
    NotARef,
    LengthWrite,
    PathUnderflow
}

impl Ins {
    fn resolve_path(path: &ValuePath, stack: &mut TypeStack, function: &Function, unit: &TranslationUnit) -> Result<ValueType, ValidationError> {
        let mut curr_type = match path.origin() {
            ValuePathOrigin::Local(local_idx, local_type) =>
                match function.get_local(*local_idx) {
                    None => return Err(ValidationError::LocalDoesNotExist),
                    Some(x) =>
                        if x.local_type() != local_type {
                            return Err(ValidationError::LocalIncorrectType)
                        } else {
                            local_type.clone()
                        },
                },
            ValuePathOrigin::Global(global_idx, global_type) =>
                match unit.get_global(*global_idx) {
                    None => return Err(ValidationError::GlobalDoesNotExist),
                    Some(x) =>
                        if x.global_type() != global_type {
                            return Err(ValidationError::GlobalIncorrectType)
                        } else {
                            global_type.clone()
                        },
                },
            ValuePathOrigin::Deref(expected_deref_type) =>
                match stack.pop()? {
                    ValueType::Ref(deref_type) =>
                        if deref_type.as_ref() != expected_deref_type {
                            return Err(ValidationError::StackIncorrectType)
                        } else {
                            expected_deref_type.clone()
                        },
                    _ => return Err(ValidationError::NotARef)
                },
        };

        for component in path.components() {
            curr_type = match component {
                ValuePathComponent::Slice(expected_slice_type) =>
                    match curr_type {
                        StorableType::Slice(slice_type) =>
                            if slice_type.as_ref() != expected_slice_type {
                                return Err(ValidationError::PathIncorrectType)
                            } else {
                                pop!(stack, ValueType::Index(s) if s == slice_type);
                                expected_slice_type.clone()   
                            }
                        _ => return Err(ValidationError::PathIncorrectType)
                    },
                ValuePathComponent::Property(prop_idx, expected_compound_type, prop_type) =>
                    match curr_type {
                        StorableType::Compound(compound_type) =>
                            if &compound_type != expected_compound_type {
                                return Err(ValidationError::PathIncorrectType)
                            } else {
                                match compound_type.content() {
                                    CompoundContent::Struct(struc) =>
                                        match struc.prop(*prop_idx) {
                                            Some(x) =>
                                                if x.prop_type() != prop_type {
                                                    return Err(ValidationError::PropertyIncorrectType)
                                                } else {
                                                    prop_type.clone()
                                                },
                                            None => return Err(ValidationError::PropertyDoesNotExist)
                                        },
                                }
                            },
                        _ => return Err(ValidationError::PathIncorrectType)
                    },
                ValuePathComponent::Length =>
                    match curr_type {
                        StorableType::Slice(_) => StorableType::Value(ValueType::UPtr),
                        _ => return Err(ValidationError::PathIncorrectType)
                    }
            }
        }

        match curr_type {
            StorableType::Value(v) => Ok(v),
            _ => return Err(ValidationError::PathNotValue)
        }
    }
    
    fn validate(&self, stack: &mut TypeStack, blocks: &mut BlockStack, function: &Function, unit: &TranslationUnit) -> Result<(), ValidationError> {
        match &self {
            Ins::Push(expected_vt) => Ok({
                pop_path!(stack, = *expected_vt);
                stack.push(expected_vt.clone());
            }),
            Ins::Pop(expected_vt) => Ok({
                pop!(stack, = *expected_vt);
                pop_path!(stack, = *expected_vt);
            }),
            Ins::PushPath(path, expected_vt) => Ok({
                let resolved_vt = Ins::resolve_path(path, stack, function, unit)?;
                if &resolved_vt != expected_vt {
                    return Err(ValidationError::PathIncorrectType)
                }
                stack.push_path(resolved_vt);
            }),
            Ins::Index(slice_type) => Ok({
                pop!(stack, ValueType::UPtr);
                stack.push(ValueType::Index(Box::new(slice_type.clone())));
            }),
            Ins::New(object_type) => Ok({
                stack.push(ValueType::Ref(Box::new(object_type.clone())));
            }),
            Ins::NewSlice(slice_type) => Ok({
                pop!(stack, ValueType::UPtr);
                stack.push(ValueType::Ref(Box::new(StorableType::Slice(Box::new(slice_type.clone())))));
            }),
            Ins::Free(object_type) => Ok({
                pop!(stack, ValueType::Ref(target) if target.as_ref() == object_type);
            }),
            Ins::FreeSlice(slice_type) => Ok({
                pop!(stack, ValueType::Ref(target) if matches!(target.as_ref(), StorableType::Slice(target_slice_type) if target_slice_type.as_ref() == slice_type));
            }),
            Ins::Convert(from, to) => Ok({
                if !from.is_num() || !to.is_num() {
                    return Err(ValidationError::StackNotValue)
                }
                
                pop!(stack, = *from);
                stack.push(to.clone());
            }),
            Ins::Call(idx) => Ok({
                if let Some(func) = unit.get_function(*idx) {
                    let sig = func.signature();

                    // Params come off the stack in reverse order
                    for i in 0..sig.param_count() {
                        pop!(stack, = sig.params()[sig.param_count() - i - 1]);
                    }

                    // Returns are pushed onto the stack in order
                    for i in 0..sig.return_count() {
                        stack.push(sig.returns()[i].clone());
                    }
                } else {
                    return Err(ValidationError::FunctionDoesNotExist);
                }
            }),
            Ins::Ret => Ok({
                if stack.depth() < function.signature().return_count() {
                    return Err(ValidationError::StackUnderflow)
                } else if stack.depth() > function.signature().return_count() {
                    return Err(ValidationError::StackDepthNotZero)
                }

                for i in 0..function.signature().return_count() {
                    pop!(stack, = function.signature().returns()[function.signature().return_count() - i - 1]);
                }
            }),
            Ins::Inc(vt, _) | Ins::Dec(vt, _) => Ok({
                if !vt.is_num() { return Err(ValidationError::StackNotNum) }
                peek!(stack, 0, = vt);
            }),
            Ins::Neg(vt) => Ok({
                if !vt.is_num() { return Err(ValidationError::StackNotNum) }
                peek!(stack, 0, = vt);
            }),
            Ins::Add(operand_type) | Ins::Mul(operand_type) | Ins::Div(operand_type) | Ins::Sub(operand_type) => Ok({
                if !operand_type.is_num() { return Err(ValidationError::StackNotNum) }
                pop!(stack, = *operand_type);
                peek!(stack, 0, = operand_type);
            }),
            Ins::Eq(operand_type) | Ins::Ne(operand_type) | Ins::Lt(operand_type) | Ins::Le(operand_type) | Ins::Gt(operand_type) | Ins::Ge(operand_type) => Ok({
                pop!(stack, = *operand_type);
                pop!(stack, = *operand_type);
                stack.push(ValueType::Bool);
            }),
            Ins::BoolAnd | Ins::BoolOr => Ok({
                pop!(stack, = ValueType::Bool);
                peek!(stack, 0, = &ValueType::Bool);
            }),
            Ins::Loop(block, condition, inc) => Ok({
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::Loop, |blocks| {
                    for el in block { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotOne); }

                for el in inc { el.validate(stack, blocks, function, unit)?; }
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotOne); }

                for el in condition { el.validate(stack, blocks, function, unit)?; }
                if stack.depth() != 1 { return Err(ValidationError::StackDepthNotOne); }

                pop!(stack, ValueType::Bool);
            }),
            Ins::If(block, cond) => Ok({
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }

                for el in cond { el.validate(stack, blocks, function, unit)?; }
                pop!(stack, ValueType::Bool);

                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::If, |blocks| {
                    for el in block { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
            }),
            Ins::IfElse(true_then, else_then, cond) => Ok({
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }

                for el in cond { el.validate(stack, blocks, function, unit)?; }
                pop!(stack, ValueType::Bool);
                
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::IfElse, |blocks| {
                    for el in true_then { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }

                blocks.with(BlockElement::IfElse, |blocks| {
                    for el in else_then { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
            }),
            Ins::Break(idx) => Ok({
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                if !blocks.is_breakable(*idx) { return Err(ValidationError::NotBreakable) }
            }),
            Ins::Continue(idx) => Ok({
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                if !blocks.is_continuable(*idx) { return Err(ValidationError::NotContinuable) }
            }),
            Ins::PushLiteral(vt, _) => Ok(stack.push(vt.clone())),
            Ins::Drop => Ok({ stack.pop()?; }),
        }
    }
}

// TODO: Loops are not handled here, so if the only way to exit a loop is to return, it will still be considered invalid
fn ensure_returns(block: &Vec<Ins>) -> Result<(), ValidationError> {
    match block.last() {
        Some(Ins::Ret) => Ok(()),
        Some(Ins::IfElse(a, b, _)) => {
            ensure_returns(a)?;
            ensure_returns(b)?;
            Ok(())
        },
        _ => Err(ValidationError::NoFinalReturn)
    }
}

impl Function {
    pub fn validate(&self, unit: &TranslationUnit) -> Result<(), ValidationError> {
        if self.is_extern() { return Ok(()); }

        let mut type_stack = TypeStack::new();
        let mut block_stack = BlockStack::new();

        if self.signature().param_count() > self.local_count() {
            return Err(ValidationError::LocalUnderflow);
        }

        for ins in self.code().iter() {
            ins.validate(&mut type_stack, &mut block_stack, self, unit)?;
        }

        if type_stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }

        ensure_returns(self.code())?;

        Ok(())
    }
}

impl TranslationUnit {
    pub fn validate(&self) -> Result<(), ValidationError> {
        for function in self.functions() {
            function.validate(self)?;
        }

        // TODO: Validate global defaults

        Ok(())
    }
}