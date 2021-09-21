use crate::{Function, Ins, TranslationUnit, ValueType, Signature};

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

    fn ensure(&self, value_type: ValueType, index: usize) -> Result<(), ValidationError> {
        if let Some(t) = self.types.get(self.types.len() - 1 - index) {
            if *t != value_type {
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

    fn pop(&mut self, value_type: ValueType) -> Result<(), ValidationError> {
        if let Some(t) = self.types.pop() {
            if t != value_type {
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
    GlobalDoesNotExist,
    FunctionDoesNotExist,
    NotBreakable,
    NotContinuable,
    NoFinalReturn
}

impl Ins {
    fn validate(&self, stack: &mut TypeStack, blocks: &mut BlockStack, function: &Function, unit: &TranslationUnit) -> Result<(), ValidationError> {
        match &self {
            Ins::PushLocal(vt, idx) => {
                if let Some(local) = function.locals().get(*idx) {
                    if local.value_type() != *vt {
                        Err(ValidationError::LocalIncorrectType)
                    } else {
                        Ok(stack.push(*vt))
                    }
                } else {
                    Err(ValidationError::LocalDoesNotExist)
                }
            },
            Ins::PopLocal(vt, idx) => {
                if let Some(local) = function.locals().get(*idx) {
                    if local.value_type() != *vt {
                        Err(ValidationError::LocalIncorrectType)
                    } else {
                        stack.pop(*vt)
                    }
                } else {
                    Err(ValidationError::LocalDoesNotExist)
                }
            },
            Ins::PushGlobal(_, _, _) => todo!(),
            Ins::PopGlobal(_, _, _) => todo!(),
            Ins::Call(idx) => {
                if *idx >= unit.functions().len() { Err(ValidationError::FunctionDoesNotExist) }
                else {
                    let sig = unit.functions()[*idx].signature();

                    // Params come off the stack in reverse order
                    for i in 0..sig.params().len() {
                        stack.pop(sig.params()[sig.params().len() - i - 1])?;
                    }

                    // Returns are pushed onto the stack in order
                    for i in 0..sig.returns().len() {
                        stack.push(sig.returns()[i]);
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
                        stack.pop(function.signature().returns()[function.signature().returns().len() - i - 1])?;
                    }
                    Ok(())
                }
			},
            Ins::Inc(vt, _) => stack.ensure(*vt, 0),
            Ins::Dec(vt, _) => stack.ensure(*vt, 0),
            Ins::Add(vt) => stack.pop(*vt).and(stack.ensure(*vt, 0)),
            Ins::Mul(vt) => stack.pop(*vt).and(stack.ensure(*vt, 0)),
            Ins::Div(vt) => stack.pop(*vt).and(stack.ensure(*vt, 0)),
            Ins::Sub(vt) => stack.pop(*vt).and(stack.ensure(*vt, 0)),
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

                stack.pop_any()?;
                
                Ok(())
            },
            Ins::If(block) => {
                stack.pop_any()?;

                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                blocks.with(BlockElement::If, |blocks| {
                    for el in block { el.validate(stack, blocks, function, unit)?; }
                    Ok(())
                })?;
                if stack.depth() != 0 { return Err(ValidationError::StackDepthNotZero); }
                
                Ok(())
            },
            Ins::IfElse(block_a, block_b) => {
                stack.pop_any()?;
                
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
            Ins::PushLiteral(vt, _) => Ok(stack.push(*vt)),
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