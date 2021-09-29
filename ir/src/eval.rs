use crate::{BlockMoveDepth, Function, Ins, Local, LocalIndex, TranslationUnit, ValueType};

pub struct StackElement {
    value: u64,
    value_type: ValueType
}

impl StackElement {
    pub fn new(value: u64, value_type: ValueType) -> StackElement {
        StackElement {
            value, value_type
        }
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    pub fn get(&self) -> u64 {
        self.value
    }

    pub fn set(&mut self, value: u64) {
        self.value = value;
    }
}

impl std::fmt::Debug for StackElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value_type.signed() {
            f.write_fmt(format_args!("{}", self.value as i64))
        } else {
            f.write_fmt(format_args!("{}", self.value as u64))
        }
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

    fn pop(&mut self, vt: ValueType) -> StackElement {
        assert!(self.elements.len() >= 1);
        let element = self.elements.pop().unwrap();
        assert_eq!(vt, element.value_type);
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
    locals: Vec<StackElement>
}

impl FunctionContext {
    fn new(locals: &Vec<Local>) -> FunctionContext {
        FunctionContext {
            locals: locals.iter().map(|x| StackElement::new(0, x.value_type())).collect()
        }
    }

    fn set_local(&mut self, idx: LocalIndex, vt: ValueType, value: StackElement) {
        assert!(idx < self.locals.len());
        assert_eq!(self.locals[idx].value_type(), vt);
        assert_eq!(value.value_type(), vt);
        self.locals[idx].set(value.value);
    }

    fn get_local(&self, idx: LocalIndex, vt: ValueType) -> u64 {
        assert!(idx < self.locals.len());
        assert_eq!(self.locals[idx].value_type(), vt);
        self.locals[idx].value
    }
}

#[derive(Debug)]
pub enum EvalError {

}

enum EvalResultAction {
    Next, Ret,
    Break(BlockMoveDepth),
    Continue(BlockMoveDepth),
}

impl Ins {
    fn evaluate(&self, stack: &mut Stack, function: &Function, ctx: &mut FunctionContext, unit: &TranslationUnit) -> Result<EvalResultAction, EvalError> {
        match &self {
            Ins::PushLocal(vt, idx) => {
                stack.push(StackElement::new(ctx.get_local(*idx, *vt), *vt));
                Ok(EvalResultAction::Next)
            },
            Ins::PopLocal(vt, idx) => {
                ctx.set_local(*idx, *vt, stack.pop(*vt));
                Ok(EvalResultAction::Next)
            },
            Ins::PushGlobal(_, _, _) => todo!(),
            Ins::PopGlobal(_, _, _) => todo!(),
            Ins::Call(_) => todo!(),
            Ins::Ret => Ok(EvalResultAction::Ret),
            Ins::Inc(vt, x) => {
                let val = stack.pop(*vt).value;
                stack.push(StackElement::new(val + x, *vt));
                Ok(EvalResultAction::Next)
            },
            Ins::Dec(vt, x) => {
                let val = stack.pop(*vt).value;
                stack.push(StackElement::new(val - x, *vt));
                Ok(EvalResultAction::Next)
            },
            Ins::Add(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(left.value + right.value, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Mul(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(left.value * right.value, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Div(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(left.value + right.value, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Sub(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(left.value + right.value, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Eq(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(if left.value == right.value { 1 } else { 0 }, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Ne(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(if left.value != right.value { 1 } else { 0 }, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Lt(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(if left.value < right.value { 1 } else { 0 }, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Le(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(if left.value <= right.value { 1 } else { 0 }, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Gt(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(if left.value > right.value { 1 } else { 0 }, *vt));

                Ok(EvalResultAction::Next)
            },
            Ins::Ge(vt) => {
                let right = stack.pop(*vt);
                let left = stack.pop(*vt);

                stack.push(StackElement::new(if left.value >= right.value { 1 } else { 0 }, *vt));

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

                    if stack.pop_any().value == 0 { break }

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
                if stack.pop_any().value != 0 {
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
                if stack.pop_any().value != 0 {
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
                stack.push(StackElement::new(*lit, *vt));
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
            result.push(stack.pop(self.signature().returns()[len - i - 1]));
        }

        result.reverse();

        Ok(result)
    }
}