use crate::{CompoundTypeRef, GlobalIndex, PropertyIndex, StorableType, ValueType, unit::*};

#[derive(Debug)]
pub enum ValuePathOrigin {
    Local(LocalIndex, StorableType),
    Global(GlobalIndex, StorableType),
    Deref(StorableType)
}

#[derive(Debug)]
pub enum ValuePathComponent {
    Slice(StorableType),
    Property(PropertyIndex, CompoundTypeRef, StorableType),
    Length
}

#[derive(Debug)]
pub struct ValuePath {
    origin: ValuePathOrigin,
    components: Vec<ValuePathComponent>
}

impl ValuePath {
    pub fn new(origin: ValuePathOrigin, components: Vec<ValuePathComponent>) -> ValuePath {
        ValuePath {
            origin, components
        }
    }

    pub fn new_origin_only(origin: ValuePathOrigin) -> ValuePath {
        ValuePath {
            origin,
            components: Vec::new()
        }
    }

    pub fn origin(&self) -> &ValuePathOrigin {
        &self.origin
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn component(&self, idx: usize) -> Option<&ValuePathComponent> {
        self.components.get(idx)
    }

    pub fn components(&self) -> &Vec<ValuePathComponent> {
        &self.components
    }
}

#[derive(Debug)]
pub enum Ins {
    /// Push the given path to the path stack, popping from the stack as needed to get references / indices etc
    PushPath(ValuePath, ValueType),
    /// Pop a path from the path stack and push the value it points to
    Push(ValueType),
    /// Pop a path from the path stack, and pop the value it should point to
    Pop(ValueType),

    /// Pop a uptr and push an Index in it's place
    Index(StorableType),

    /// Allocates a value of the given type, and pushes a reference to it
    New(StorableType),

    /// Pops a uptr, then allocates a slice of the given type of the given length, and pushes a reference to it
    NewSlice(StorableType),

    /// Pops a reference to an object of the given storable type and frees it, if possible
    Free(StorableType),

    /// Pops a reference to a slice of the given storable type and frees it, if possible
    FreeSlice(StorableType),

    /// Convert from one valuetype to another. All conversions must be numeric or boolean.
    /// Longer -> Smaller  truncates the higher bits 
    /// Same size  does not change bit structure, even between signs
    /// Smaller -> Longer
    ///     Signed -> Signed  should be sign extended
    ///     Unsigned -> Unsigned  should be zero extended
    ///     Signed -> Unsigned  should be sign extended, then converted
    ///     Unsigned -> Signed  should be zero extended, then converted
    /// Boolean -> Num  will be zero if false, and non-zero otherwise
    /// Num -> Boolean  will be false if zero, and true otherwise
    Convert(ValueType, ValueType),
    
    /// Calls the function at the given index.
    /// The parameters to the function will be popped from the stack in reverse order, meaning that the first param should be pushed first.
    /// The return values will be popped from the stack in reverse order, so the first return value should be pushed first.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("divide_10_by_5", ir::Signature::new(vec![], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::Call(0 /* some index */));
    /// func.push(ir::Ins::Ret);
    /// ```
    Call(FunctionIndex),
    
    /// Exits the current function, the returned values should be on the stack so that they are popped in reversed order.
    /// The values on the stack at this point must conform to the return values signature of the function
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("return_two_values", ir::Signature::new(vec![], vec![ir::ValueType::I32, ir::ValueType::U8 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::U8, 5));
    /// func.push(ir::Ins::Ret);
    /// ```
    Ret,

    /// Adds the given value to the last item on the stack. The item must have the given value type, and will maintain that same valuetype.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("add_1", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Inc(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::Ret);
    /// ```
    Inc(ValueType, u64),

    /// Subtracts the given value to the last item on the stack. The item must have the given value type, and will maintain that same valuetype.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("sub_1", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Dec(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::Ret);
    /// ```
    Dec(ValueType, u64),

    /// Adds the last two items on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("add", ir::Signature::new(vec![ ir::ValueType::I32, ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Add(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Add(ValueType),
    
    /// Multiplies the last two items on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("mul", ir::Signature::new(vec![ ir::ValueType::I32, ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Mul(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Mul(ValueType),
    
    /// Divides the second to last item by the last item on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("divide_10_by_5", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::Div(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Div(ValueType),
    
    /// Subtracts the last item from the second to last item on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("sub_5_from_10", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::Sub(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Sub(ValueType),

    /// Pushes 1 if the last two items on the stack are equal, and 0 otherwise. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::Eq(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Eq(ValueType),

    /// Pushes 0 if the last two items on the stack are equal, and 1 otherwise. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
    /// func.push(ir::Ins::Ne(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Ne(ValueType),

    /// Pushes 1 if the second to last item on the stack is less than the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
    /// func.push(ir::Ins::Lt(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Lt(ValueType),

    /// Pushes 1 if the second to last item on the stack is less than or equal to the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::Le(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Le(ValueType),

    /// Pushes 1 if the second to last item on the stack is greater than the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::Gt(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Gt(ValueType),

    /// Pushes 1 if the second to last item on the stack is greater than or equal to the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
    /// func.push(ir::Ins::Ge(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Ge(ValueType),

    /// Pushes boolean 1 if both of the last two (popped) items on the stack are 1
    BoolAnd,

    /// Pushes boolean 1 if either of the last two (popped) items on the stack are 1
    BoolOr,

    /// Code, Condition, Inc
    /// Continues to loop over it's code while it's condition does not evaluate to zero, running inc after each iteration
    /// Initially evaluates the condition. If it is zero, it breaks. Otherwise it runs code, followed by inc, and then re-evaluates.
    /// Code will start and end with an empty stack, condition will start with an empty stack and end with one (bool) value, inc will start and end with an empty stack.
    /// If a continue is reached inside of the loop, inc is executed and the loop starts again.
    /// If a break is reached, the whole loop is stopped immediately.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("loop_5_times", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32))); // Allocate local of type i32
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, l1));
    ///
    /// func.push(ir::Ins::Loop(
    ///     vec![ // Body
    ///         // do nothing
    ///     ],
    ///     vec![ // Condition
    ///         ir::Ins::PushLocalValue(ir::ValueType::I32, l1),
    ///         ir::Ins::PushLocalValue(ir::ValueType::I32, 0),
    ///         ir::Ins::Ne(ir::ValueType::I32),
    ///     ],
    ///     vec![ // Increment (or in this case, decrement)
    ///         ir::Ins::PushLocalValue(ir::ValueType::I32, l1),
    ///         ir::Ins::Dec(ir::ValueType::I32, 1),
    ///         ir::Ins::PopLocalValue(ir::ValueType::I32, l1),
    ///     ]
    /// ));
    /// func.push(ir::Ins::Ret);
    /// ```
    Loop(Vec<Ins>, Vec<Ins>, Vec<Ins>),

    /// Code, Condition
    /// Executes it's children if the popped item is nonzero.
    /// The instruction must start with one (bool) item on the stack, and it's content must end with 0.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("if_1", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::Bool, 1));
    /// func.push(ir::Ins::If(vec![
    ///     // Do something
    /// ]));
    /// func.push(ir::Ins::Ret);
    /// ```
    If(Vec<Ins>, Vec<Ins>),

    /// true_then, false_then, condition
    /// Executes it's first set of children if the popped item is nonzero, otherwise executes it's second set.
    /// The instruction must start with one (bool) item on the stack, and both branches must end with 0.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("if_1_else", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::Bool, 1));
    /// func.push(ir::Ins::IfElse(vec![
    ///     // Do something
    /// ], vec![
    ///     // Do something else
    /// ]));
    /// func.push(ir::Ins::Ret);
    /// ```
    IfElse(Vec<Ins>, Vec<Ins>, Vec<Ins>),

    /// Breaks out of the loop at the depth above the current instruction given. The depth must refer to a Loop, and must run with an empty stack
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("break_immediately", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// func.push(ir::Ins::Loop(
    ///     vec![ // Body
    ///         ir::Ins::Break(0),
    ///         // This code will never be run
    ///         // ...
    ///     ],
    ///     vec![ // Condition
    ///         ir::Ins::PushLiteral(ir::ValueType::I32, 1),
    ///     ],
    ///     vec![ // Increment
    ///     ]
    /// ));
    /// func.push(ir::Ins::Ret);
    /// ```
    Break(BlockMoveDepth),

    /// Continues to the the increment and then top of the loop at the depth above the current instruction given. The depth must refer to a Loop, and must run with an empty stack.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("continue_immediately", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// // Infinite loop
    /// func.push(ir::Ins::Loop(
    ///     vec![ // Body
    ///         ir::Ins::Continue(0),
    ///         // This code will never be run
    ///         // ...
    ///     ],
    ///     vec![ // Condition
    ///         ir::Ins::PushLiteral(ir::ValueType::I32, 1),
    ///     ],
    ///     vec![ // Increment
    ///     ]
    /// ));
    /// func.push(ir::Ins::Ret);
    /// ```
    Continue(BlockMoveDepth),

    /// Push the literal number with the given valuetype to the stack
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 42));
    /// func.push(ir::Ins::Ret);
    /// ```
    PushLiteral(ValueType, u64),

    /// Pop the last item from the stack, regardless of the type. There must be 1 or more items on the stack at this point.
    /// On register based targets this will be removed entirely
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 42));
    /// func.push(ir::Ins::Drop);
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 21));
    /// func.push(ir::Ins::Ret);
    /// ```
    Drop
}