use crate::{BlockMoveDepth, Function, Ins, Local, LocalIndex, StorableType, TranslationUnit, ValueType};

#[derive(Debug)]
pub enum StackValue {
    Num(u64),
    LocalRef(LocalIndex)
}

pub struct StackElement {
    value: StackValue,
    value_type: ValueType
}

impl StackElement {
    pub fn new_num(value: u64, value_type: ValueType) -> StackElement {
        StackElement {
            value: StackValue::Num(value), value_type
        }
    }

    pub fn new(value: StackValue, value_type: ValueType) -> StackElement {
        StackElement {
            value, value_type
        }
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn get_num(&self) -> u64 {
        match self.value {
            StackValue::Num(n) => n,
            _ => panic!("Expected num, found {:?}", self.value)
        }
    }

    pub fn get(&self) -> &StackValue {
        &self.value
    }

    pub fn set(&mut self, value: StackValue) {
        self.value = value;
    }
}

impl std::fmt::Debug for StackElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            StackValue::Num(n) => if self.value_type.signed() {
                f.write_fmt(format_args!("{}", n as i64))
            } else {
                f.write_fmt(format_args!("{}", n as u64))
            },
            StackValue::LocalRef(r) => f.write_fmt(format_args!("<local {}>", r))
        }
    }
}

enum LocalElementValue {
    Num(u64),
    LocalRef(LocalIndex),
}

struct LocalElement {
    local_type: StorableType,
    value: LocalElementValue
}

impl LocalElement {
    pub fn new(local_type: StorableType, value: LocalElementValue) -> LocalElement {
        LocalElement {
            local_type,
            value
        }
    }

    pub fn local_type(&self) -> &StorableType {
        &self.local_type
    }

    pub fn get(&self) -> &LocalElementValue {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut LocalElementValue {
        &mut self.value
    }
}

struct Stack {
    elements: Vec<StackElement>
}

impl Stack {
    fn from_params(params: Vec<StackElement>) -> Stack {
        Stack {
            elements: params
        }
    }

    fn pop(&mut self, vt: &ValueType) -> StackElement {
        assert!(self.elements.len() >= 1);
        let element = self.elements.pop().unwrap();
        assert_eq!(vt, &element.value_type);
        element
    }

    fn pop_any(&mut self) -> StackElement {
        assert!(self.elements.len() >= 1);
        self.elements.pop().unwrap()
    }

    fn push(&mut self, el: StackElement) {
        self.elements.push(el);
    }
}

struct FunctionContext {
    locals: Vec<LocalElement>
}

impl FunctionContext {
    fn new(locals: &Vec<Local>) -> FunctionContext {
        FunctionContext {
            locals: locals.iter().map(|x| LocalElement::new(
                x.local_type().clone(),
                match x.local_type() {
                    StorableType::Value(_) => LocalElementValue::Num(0),
                    _ => todo!(),
                }
            )).collect()
        }
    }

    fn get_local_value(&self, idx: LocalIndex, vt: &ValueType) -> u64 {
        assert!(idx < self.locals.len());
        assert!(matches!(self.locals[idx].local_type(), StorableType::Value(v) if v == vt));
        
        match &self.locals[idx].get() {
            LocalElementValue::Num(x) => *x,
            _ => unreachable!(),
        }
    }

    fn get_local_value_mut(&mut self, idx: LocalIndex, vt: &ValueType) -> &mut u64 {
        assert!(idx < self.locals.len());
        assert!(matches!(self.locals[idx].local_type(), StorableType::Value(v) if v == vt));
        
        match self.locals[idx].get_mut() {
            LocalElementValue::Num(x) => x,
            _ => unreachable!(),
        }
    }

    fn get_local_mut(&mut self, idx: LocalIndex, st: &StorableType) -> &mut LocalElement {
        assert!(idx < self.locals.len());
        assert_eq!(self.locals[idx].local_type(), st);
        &mut self.locals[idx]
    }
}

#[derive(Debug)]
pub enum EvalError {
    CallToExtern
}

enum EvalResultAction {
    Next, Ret,
    Break(BlockMoveDepth),
    Continue(BlockMoveDepth),
}

impl Ins {
    fn evaluate(&self, stack: &mut Stack, function: &Function, ctx: &mut FunctionContext, unit: &TranslationUnit) -> Result<EvalResultAction, EvalError> {
        match &self {
            Ins::PushLocalValue(vt, idx) => {
                stack.push(StackElement::new_num(ctx.get_local_value(*idx, vt), vt.clone()));
                Ok(EvalResultAction::Next)
            },
            Ins::PushLocalRef(st, idx) => {
                stack.push(StackElement::new(StackValue::LocalRef(*idx), ValueType::Ref(Box::new(st.clone()))));
                Ok(EvalResultAction::Next)
            },
            Ins::PopLocalValue(vt, idx) => {
                *ctx.get_local_value_mut(*idx, vt) = stack.pop(vt).get_num();
                Ok(EvalResultAction::Next)
            },
            Ins::PopRef(vt) => {
                let val = stack.pop(vt);
                let dest = stack.pop(&ValueType::Ref(Box::new(StorableType::Value(vt.clone()))));

                match dest.value {
                    StackValue::Num(_) => panic!("Cannot PopRef to non-ref"),
                    StackValue::LocalRef(idx) => {
                        match vt {
                            ValueType::Ref(st) => {
                                ctx.get_local_mut(idx, st).value = LocalElementValue::LocalRef(match val.value {
                                    StackValue::LocalRef(i) => i,
                                    _ => unreachable!()
                                });
                            },
                            _ => {
                                *ctx.get_local_value_mut(idx, vt) = val.get_num();
                            }
                        }
                    },
                }

                Ok(EvalResultAction::Next)
            },
            Ins::PushProperty(_, _, _) => todo!(),
            Ins::PushPropertyRef(_, _, _) => todo!(),
            Ins::PushSliceLen(_) => todo!(),
            Ins::Call(_) => todo!(),
            Ins::Ret => Ok(EvalResultAction::Ret),
            Ins::Inc(vt, x) => {
                let val = stack.pop(vt).get_num();
                stack.push(StackElement::new_num(val + x, vt.clone()));
                Ok(EvalResultAction::Next)
            },
            Ins::Dec(vt, x) => {
                let val = stack.pop(vt).get_num();
                stack.push(StackElement::new_num(val - x, vt.clone()));
                Ok(EvalResultAction::Next)
            },
            Ins::Add(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(left.get_num() + right.get_num(), vt.clone()));

                Ok(EvalResultAction::Next)
            },
            Ins::Mul(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(left.get_num() * right.get_num(), vt.clone()));

                Ok(EvalResultAction::Next)
            },
            Ins::Div(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(left.get_num() + right.get_num(), vt.clone()));

                Ok(EvalResultAction::Next)
            },
            Ins::Sub(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(left.get_num() + right.get_num(), vt.clone()));

                Ok(EvalResultAction::Next)
            },
            Ins::Eq(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(if left.get_num() == right.get_num() { 1 } else { 0 }, ValueType::Bool));

                Ok(EvalResultAction::Next)
            },
            Ins::Ne(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(if left.get_num() != right.get_num() { 1 } else { 0 }, ValueType::Bool));

                Ok(EvalResultAction::Next)
            },
            Ins::Lt(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(if left.get_num() < right.get_num() { 1 } else { 0 }, ValueType::Bool));

                Ok(EvalResultAction::Next)
            },
            Ins::Le(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(if left.get_num() <= right.get_num() { 1 } else { 0 }, ValueType::Bool));

                Ok(EvalResultAction::Next)
            },
            Ins::Gt(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(if left.get_num() > right.get_num() { 1 } else { 0 }, ValueType::Bool));

                Ok(EvalResultAction::Next)
            },
            Ins::Ge(vt) => {
                let right = stack.pop(vt);
                let left = stack.pop(vt);

                stack.push(StackElement::new_num(if left.get_num() >= right.get_num() { 1 } else { 0 }, ValueType::Bool));

                Ok(EvalResultAction::Next)
            },
            Ins::Loop(body, condition, inc) => {
                'outer: loop {
                    for ins in condition {
                        match ins.evaluate(stack, function, ctx, unit)? {
                            EvalResultAction::Ret => return Ok(EvalResultAction::Ret),
                            EvalResultAction::Next => {},
                            EvalResultAction::Break(_) => panic!("Break from inside loop condition"),
                            EvalResultAction::Continue(_) => panic!("Continue from inside loop condition")
                        }
                    }

                    if stack.pop(&ValueType::Bool).get_num() == 0 { break }

                    'inner: for ins in body {
                        match ins.evaluate(stack, function, ctx, unit)? {
                            EvalResultAction::Ret => return Ok(EvalResultAction::Ret),
                            EvalResultAction::Next => {},
                            EvalResultAction::Break(depth) => {
                                if depth == 0 {
                                    break 'outer;
                                } else {
                                    return Ok(EvalResultAction::Break(depth - 1));
                                }
                            },
                            EvalResultAction::Continue(depth) => {
                                if depth == 0 {
                                    break 'inner;
                                } else {
                                    return Ok(EvalResultAction::Break(depth - 1));
                                }
                            }
                        }
                    }

                    for ins in inc {
                        match ins.evaluate(stack, function, ctx, unit)? {
                            EvalResultAction::Ret => return Ok(EvalResultAction::Ret),
                            EvalResultAction::Next => {},
                            EvalResultAction::Break(_) => panic!("Break from inside loop increment"),
                            EvalResultAction::Continue(_) => panic!("Continue from inside loop increment")
                        }
                    }
                }

                Ok(EvalResultAction::Next)
            },
            Ins::If(body) => {
                if stack.pop(&ValueType::Bool).get_num() != 0 {
                    for ins in body {
                        match ins.evaluate(stack, function, ctx, unit)? {
                            EvalResultAction::Ret => return Ok(EvalResultAction::Ret),
                            EvalResultAction::Next => {},
                            EvalResultAction::Break(depth) => return Ok(EvalResultAction::Break(depth - 1)), // For valid IR depth cannot be zero
                            EvalResultAction::Continue(depth) => return Ok(EvalResultAction::Continue(depth - 1))
                        }
                    }
                }

                Ok(EvalResultAction::Next)
            },
            Ins::IfElse(body_a, body_b) => {
                if stack.pop(&ValueType::Bool).get_num() != 0 {
                    for ins in body_a {
                        match ins.evaluate(stack, function, ctx, unit)? {
                            EvalResultAction::Ret => return Ok(EvalResultAction::Ret),
                            EvalResultAction::Next => {},
                            EvalResultAction::Break(depth) => return Ok(EvalResultAction::Break(depth - 1)), // For valid IR depth cannot be zero
                            EvalResultAction::Continue(depth) => return Ok(EvalResultAction::Continue(depth - 1))
                        }
                    }
                } else {
                    for ins in body_b {
                        match ins.evaluate(stack, function, ctx, unit)? {
                            EvalResultAction::Ret => return Ok(EvalResultAction::Ret),
                            EvalResultAction::Next => {},
                            EvalResultAction::Break(depth) => return Ok(EvalResultAction::Break(depth - 1)), // For valid IR depth cannot be zero
                            EvalResultAction::Continue(depth) => return Ok(EvalResultAction::Continue(depth - 1))
                        }
                    }
                }

                Ok(EvalResultAction::Next)
            },
            Ins::Break(depth) => return Ok(EvalResultAction::Break(*depth)),
            Ins::Continue(depth) => return Ok(EvalResultAction::Continue(*depth)),
            Ins::PushLiteral(vt, lit) => {
                stack.push(StackElement::new_num(*lit, vt.clone()));
                Ok(EvalResultAction::Next)
            },
            Ins::Drop => {
                stack.pop_any();
                Ok(EvalResultAction::Next)
            },
        }
    }
}

impl Function {
    fn evaluate_on(&self, stack: &mut Stack, unit: &TranslationUnit) -> Result<(), EvalError> {
        if self.is_extern() { return Err(EvalError::CallToExtern); }

        let mut ctx = FunctionContext::new(self.locals());

        for ins in self.code() {
            match ins.evaluate(stack, self, &mut ctx, unit)? {
                EvalResultAction::Ret => return Ok(()),
                EvalResultAction::Next => {},
                EvalResultAction::Break(_) => unreachable!(),
                EvalResultAction::Continue(_) => unreachable!()
            }
        }
        
        panic!("There was no final ret, this should be caught by validation");
    }
    
    /// Simulates a function using the given params. Functions should already be validated before they are evaluated, and so unvalidated code my lead either to panics or undefined behaviour.
    /// # Examples
    /// ```
    /// use ir::*;
    ///
    /// let mut unit = ir::TranslationUnit::new();
    /// 
    /// let mut func = ir::Function::new("add", ir::Signature::new(vec![ ValueType::I32, ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Add(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// 
    /// let func_id = unit.add_function(func);
    /// 
    /// assert!(unit.validate().is_ok());
    /// 
    /// let func = unit.get_function(func_id);
    /// let returns = func.evaluate(&unit, vec![ ir::StackElement::new(7, ValueType::I32), ir::StackElement::new(8, ValueType::I32) ]).unwrap();
    /// assert_eq!(returns[0].get(), 15);
    /// ```
    pub fn evaluate(&self, unit: &TranslationUnit, params: Vec<StackElement>) -> Result<Vec<StackElement>, EvalError> {
        // Params go on the stack in order
        let mut stack = Stack::from_params(params);

        self.evaluate_on(&mut stack, unit)?;

        let len = self.signature().returns().len();
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            // Returns come off the stack in reverse order
            result.push(stack.pop(&self.signature().returns()[len - i - 1]));
        }

        result.reverse();

        Ok(result)
    }
}