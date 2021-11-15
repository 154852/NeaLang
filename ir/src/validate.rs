use crate::{Function, Ins, StorableType, TranslationUnit, TypeContent, ValuePath, ValuePathComponent, ValuePathOrigin, ValueType};

struct TypeStack {
    types: Vec<ValueType>
}

impl TypeStack {
    fn new() -> TypeStack {
        TypeStack {
            types: Vec::new()
        }
    }

    fn push(&mut self, value_type: ValueType) {
        self.types.push(value_type);
    }

    fn ensure(&self, value_type: &ValueType, index: usize) -> Result<(), ValidationError> {
        if let Some(t) = self.types.get(self.types.len() - 1 - index) {
            if t != value_type {
                Err(ValidationError::StackIncorrectType)
            } else {
                Ok(())
            }
        } else {
            Err(ValidationError::StackUnderflow)
        }
    }

    fn pop_any(&mut self) -> Result<ValueType, ValidationError> {
        if let Some(t) = self.types.pop() {
            Ok(t)
        } else {
            Err(ValidationError::StackUnderflow)
        }
    }

    fn pop(&mut self, value_type: &ValueType) -> Result<(), ValidationError> {
        if let Some(t) = self.types.pop() {
            if &t != value_type {
                Err(ValidationError::StackIncorrectType)
            } else {
                Ok(())
            }
        } else {
            Err(ValidationError::StackUnderflow)
        }
    }

    fn depth(&self) -> usize {
        self.types.len()
    }
}

struct PathStack {
    types: Vec<ValueType>
}

impl PathStack {
    pub fn new() -> PathStack {
        PathStack {
            types: Vec::new()
        }
    }

    pub fn pop(&mut self) -> Result<ValueType, ValidationError> {
        match self.types.pop() {
            Some(x) => Ok(x),
            None => Err(ValidationError::PathUnderflow)
        }
    }

    pub fn push(&mut self, value: ValueType) {
        self.types.push(value)
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
        self.elements.get(self.elements.len() - 1 - index).map_or(false, |x| matches!(x, BlockElement::Loop))
    }

    fn is_continuable(&self, index: usize) -> bool {
        self.elements.get(self.elements.len() - 1 - index).map_or(false, |x| matches!(x, BlockElement::Loop))
    }
}

#[derive(Debug)]
pub enum ValidationError {
    StackUnderflow,
    StackIncorrectType,
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
    fn resolve_path(&self, path: &ValuePath, stack: &mut TypeStack, function: &Function, unit: &TranslationUnit, write: bool) -> Result<ValueType, ValidationError> {
        let mut st = match path.origin() {
            ValuePathOrigin::Local(idx, local_type) => match function.get_local(*idx) {
                None => return Err(ValidationError::LocalDoesNotExist),
                Some(x) => if x.local_type() != local_type {
                    return Err(ValidationError::LocalIncorrectType)
                } else {
                    local_type.clone()
                },
            },
            ValuePathOrigin::Global(idx, global_type) => match unit.get_global(*idx) {
                None => return Err(ValidationError::GlobalDoesNotExist),
                Some(x) => if x.global_type() != global_type {
                    return Err(ValidationError::GlobalIncorrectType)
                } else {
                    global_type.clone()
                },
            },
            ValuePathOrigin::Deref(deref_type) => match stack.pop_any()? {
                ValueType::Ref(st) => if st.as_ref() != deref_type {
                    return Err(ValidationError::StackIncorrectType)
                } else {
                    deref_type.clone()
                },
                _ => return Err(ValidationError::NotARef)
            },
        };

        for component in path.components() {
            st = match component {
                ValuePathComponent::Slice(slice_type) => match st {
                    StorableType::Slice(st) => {
                        if st.as_ref() != slice_type { return Err(ValidationError::PathIncorrectType) }
                        stack.pop(&ValueType::UPtr)?;
                        slice_type.clone()
                    },
                    _ => return Err(ValidationError::PathIncorrectType)
                },
                ValuePathComponent::Property(idx, compound_type, prop_type) => match st {
                    StorableType::Compound(ct) => if &ct != compound_type {
                        return Err(ValidationError::PathIncorrectType)
                    } else {
                        match ct.content() {
                            TypeContent::Struct(struc) => match struc.prop(*idx) {
                                Some(x) => if x.prop_type() != prop_type {
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
                ValuePathComponent::Length => if write { return Err(ValidationError::LengthWrite) } else {
                    match st {
                        StorableType::Slice(_) => StorableType::Value(ValueType::UPtr),
                        _ => return Err(ValidationError::PathIncorrectType)
                    }
                }
            }
        }

        match st {
            StorableType::Value(v) => Ok(v),
            _ => return Err(ValidationError::PathNotValue)
        }
    }
    
    fn validate(&self, stack: &mut TypeStack, path_stack: &mut PathStack, blocks: &mut BlockStack, function: &Function, unit: &TranslationUnit) -> Result<(), ValidationError> {
        match &self {
            Ins::Push(expected_vt) => {
                let vt = path_stack.pop()?;

                if &vt != expected_vt {
                    return Err(ValidationError::PathIncorrectType)
                }

                stack.push(vt);
                Ok(())
            },
            Ins::Pop(expected_vt) => {
                stack.pop(&expected_vt)?;
                let vt = path_stack.pop()?;

                if &vt != expected_vt {
                    return Err(ValidationError::PathIncorrectType)
                }

                Ok(())
            },
            Ins::PushPath(path, expected_vt) => {
                let vt = self.resolve_path(path, stack, function, unit, false)?;
                if &vt != expected_vt {
                    return Err(ValidationError::PathIncorrectType)
                }
                path_stack.push(vt);
                Ok(())
            },
            Ins::New(st) => {
                stack.push(ValueType::Ref(Box::new(st.clone())));
                Ok(())
            },
            Ins::NewSlice(st) => {
                stack.pop(&ValueType::UPtr)?;
                stack.push(ValueType::Ref(Box::new(StorableType::Slice(Box::new(st.clone())))));
                Ok(())
            },
            Ins::Convert(from, to) => {
                if !from.is_num() || !to.is_num() {
                    return Err(ValidationError::StackNotValue)
                }
                
                stack.pop(from)?;
                stack.push(to.clone());
                Ok(())
            },
            Ins::Call(idx) => {
                if *idx >= unit.functions().len() { Err(ValidationError::FunctionDoesNotExist) }
                else {
                    let sig = unit.functions()[*idx].signature();

                    // Params come off the stack in reverse order
                    for i in 0..sig.params().len() {
                        stack.pop(&sig.params()[sig.params().len() - i - 1])?;
                    }

                    // Returns are pushed onto the stack in order
                    for i in 0..sig.returns().len() {
                        stack.push(sig.returns()[i].clone());
                    }

                    Ok(())
                }
            },
            Ins::Ret => {
                if stack.depth() < function.signature().returns().len() {
                    Err(ValidationError::StackUnderflow)
                } else if stack.depth() > function.signature().returns().len() {
                    Err(ValidationError::StackDepthNotZero)
                } else {
                    for i in 0..function.signature().returns().len() {
                        stack.pop(&function.signature().returns()[function.signature().returns().len() - i - 1])?;
                    }
                    Ok(())
                }
            },
            Ins::Inc(vt, _) => stack.ensure(vt, 0),
            Ins::Dec(vt, _) => stack.ensure(vt, 0),
            Ins::Add(vt) => stack.pop(vt).and(stack.ensure(vt, 0)),
            Ins::Mul(vt) => stack.pop(vt).and(stack.ensure(vt, 0)),
            Ins::Div(vt) => stack.pop(vt).and(stack.ensure(vt, 0)),
            Ins::Sub(vt) => stack.pop(vt).and(stack.ensure(vt, 0)),
            Ins::Eq(vt) | Ins::Ne(vt) | Ins::Lt(vt) | Ins::Le(vt) | Ins::Gt(vt) | Ins::Ge(vt) => {
                stack.pop(vt).and(stack.pop(vt))?;
                stack.push(ValueType::Bool);
                Ok(())
            },
            Ins::Loop(block, condition, inc) => {
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::Loop, |blocks| {
                    for el in block { el.validate(stack, path_stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotOne); }

                for el in inc { el.validate(stack, path_stack, blocks, function, unit)?; }
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotOne); }

                for el in condition { el.validate(stack, path_stack, blocks, function, unit)?; }
                if stack.depth() != 1 { return Err(ValidationError::StackDepthNotOne); }

                stack.pop(&ValueType::Bool)?;
                
                Ok(())
            },
            Ins::If(block) => {
                stack.pop(&ValueType::Bool)?;

                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::If, |blocks| {
                    for el in block { el.validate(stack, path_stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                
                Ok(())
            },
            Ins::IfElse(block_a, block_b) => {
                stack.pop(&ValueType::Bool)?;
                
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::IfElse, |blocks| {
                    for el in block_a { el.validate(stack, path_stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }

                blocks.with(BlockElement::IfElse, |blocks| {
                    for el in block_b { el.validate(stack, path_stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                
                Ok(())
            },
            Ins::Break(idx) => {
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                if blocks.is_breakable(*idx) {
                    Ok(())
                } else {
                    Err(ValidationError::NotBreakable)
                }
            },
            Ins::Continue(idx) => {
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                if blocks.is_continuable(*idx) {
                    Ok(())
                } else {
                    Err(ValidationError::NotContinuable)
                }
            },
            Ins::PushLiteral(vt, _) => Ok(stack.push(vt.clone())),
            Ins::Drop => stack.pop_any().map(|_| ()),
        }
    }
}

// TODO: Loops are not handled here, so if the only way to exit a loop is to return, it will still be considered invalid
fn ensure_returns(block: &Vec<Ins>) -> Result<(), ValidationError> {
    match block.last() {
        Some(Ins::Ret) => Ok(()),
        Some(Ins::IfElse(a, b)) => {
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
        let mut path_stack = PathStack::new();
        let mut block_stack = BlockStack::new();

        if self.signature().params().len() > self.locals().len() {
            return Err(ValidationError::LocalUnderflow);
        }

        for ins in self.code().iter() {
            ins.validate(&mut type_stack, &mut path_stack, &mut block_stack, self, unit)?;
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

        Ok(())
    }
}