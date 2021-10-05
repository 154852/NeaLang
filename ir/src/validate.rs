use crate::{Function, Ins, Signature, StorableType, TranslationUnit, TypeContent, ValueType};

struct TypeStack {
    types: Vec<ValueType>
}

impl TypeStack {
    fn from_signature(sig: &Signature) -> TypeStack {
        TypeStack {
            types: sig.params().clone()
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
    LocalDoesNotExist,
    LocalIncorrectType,
    FieldIncorrectType,
    GlobalDoesNotExist,
    FunctionDoesNotExist,
    NotBreakable,
    NotContinuable,
    NoFinalReturn
}

impl Ins {
    fn validate(&self, stack: &mut TypeStack, blocks: &mut BlockStack, function: &Function, unit: &TranslationUnit) -> Result<(), ValidationError> {
        match &self {
            Ins::PushLocalValue(vt, idx) => {
                if let Some(local) = function.locals().get(*idx) {
                    if !local.local_type().is_value(vt) {
                        Err(ValidationError::LocalIncorrectType)
                    } else {
                        Ok(stack.push(vt.clone()))
                    }
                } else {
                    Err(ValidationError::LocalDoesNotExist)
                }
            },
            Ins::PushLocalRef(st, idx) => {
                if let Some(local) = function.locals().get(*idx) {
                    if local.local_type() != st {
                        Err(ValidationError::LocalIncorrectType)
                    } else {
                        Ok(stack.push(ValueType::Ref(Box::new(st.clone()))))
                    }
                } else {
                    Err(ValidationError::LocalDoesNotExist)
                }
            },
            Ins::PopLocalValue(vt, idx) => {
                if let Some(local) = function.locals().get(*idx) {
                    if !local.local_type().is_value(vt) {
                        Err(ValidationError::LocalIncorrectType)
                    } else {
                        stack.pop(vt)
                    }
                } else {
                    Err(ValidationError::LocalDoesNotExist)
                }
            },
            Ins::PopRef(vt) => {
                stack.pop(vt)?;
                stack.pop(&ValueType::Ref(Box::new(StorableType::Value(vt.clone()))))?;
                Ok(())
            },
            Ins::PushProperty(ct, vt, idx) => {
                stack.pop(&ValueType::Ref(Box::new(StorableType::Compound(ct.clone()))))?;
                stack.push(vt.clone());
                
                match ct.as_ref().content() {
                    TypeContent::Struct(s) => {
                        if !matches!(s.prop(*idx), Some(p) if p.prop_type().is_value(vt)) {
                            Err(ValidationError::FieldIncorrectType)
                        } else {
                            Ok(())
                        }
                    },
                }
            },
            Ins::PushPropertyRef(ct, st, idx) => {
                stack.pop(&ValueType::Ref(Box::new(StorableType::Compound(ct.clone()))))?;
                stack.push(ValueType::Ref(Box::new(st.clone())));
                
                match ct.as_ref().content() {
                    TypeContent::Struct(s) => {
                        if !matches!(s.prop(*idx), Some(p) if p.prop_type() == st) {
                            Err(ValidationError::FieldIncorrectType)
                        } else {
                            Ok(())
                        }
                    },
                }
            },
            Ins::PushSliceLen(st) => {
                stack.pop(&ValueType::Ref(Box::new(StorableType::Slice(Box::new(st.clone())))))?;
                stack.push(ValueType::UPtr);

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
                    for el in block { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotOne); }

                for el in inc { el.validate(stack, blocks, function, unit)?; }
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotOne); }

                for el in condition { el.validate(stack, blocks, function, unit)?; }
                if stack.depth() != 1 { return Err(ValidationError::StackDepthNotOne); }

                stack.pop(&ValueType::Bool)?;
                
                Ok(())
            },
            Ins::If(block) => {
                stack.pop(&ValueType::Bool)?;

                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::If, |blocks| {
                    for el in block { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                
                Ok(())
            },
            Ins::IfElse(block_a, block_b) => {
                stack.pop(&ValueType::Bool)?;
                
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::IfElse, |blocks| {
                    for el in block_a { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }

                blocks.with(BlockElement::IfElse, |blocks| {
                    for el in block_b { el.validate(stack, blocks, function, unit)?; }
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

        let mut type_stack = TypeStack::from_signature(&self.signature());
        let mut block_stack = BlockStack::new();

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

        Ok(())
    }
}