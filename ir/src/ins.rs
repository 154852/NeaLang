use crate::{unit::*};

#[derive(Debug)]
pub enum Ins {
    /// Pushes the local at the given index to the stack. The local must match be a value, and must match in value type to the ValueType given
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// // Save param to local
    /// let l1 = func.push_local(ir::Local::new(ir::ValueType::I32)); // Allocate local of type i32
    /// func.push(ir::Ins::PopLocal(ir::ValueType::I32, l1)); // Save the given param to the local
    /// 
    /// func.push(ir::Ins::PushLocal(ir::ValueType::I32, l1)); // Push the local back onto the stack
    /// func.push(ir::Ins::Ret);
    /// ```
    PushLocalValue(ValueType, LocalIndex),
    
    /// Pops the last value on the stack to the local at the given index. The local must be a value, and the local and popped item must match in value type to the ValueType given
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// // Save param to local
    /// let l1 = func.push_local(ir::Local::new(ir::ValueType::I32)); // Allocate local of type i32
    /// func.push(ir::Ins::PopLocal(ir::ValueType::I32, l1)); // Save the given param to the local
    /// 
    /// func.push(ir::Ins::PushLocal(ir::ValueType::I32, l1)); // Push the local back onto the stack
    /// func.push(ir::Ins::Ret);
    /// ```
    PopLocalValue(ValueType, LocalIndex),
    
    /// Calls the function at the given index.
    /// The parameters to the function will be popped from the stack in reverse order, meaning that the first param should be pushed first.
    /// The return values will be popped from the stack in reverse order, so the first return value should be pushed first.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("divide_10_by_5", ir::Signature::new(vec![], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 5));
    /// func.push(ir::Ins::Call(divide_function_index));
    /// func.push(ir::Ins::Ret);
    /// ```
    Call(FunctionIndex),
    
    /// Exits the current function, the returned values should be on the stack so that they are popped in reversed order.
    /// The values on the stack at this point must conform to the return values signature of the function
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("return_two_values", ir::Signature::new(vec![], vec![ ValueType::I32, ValueType::U8 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ValueType::U8, 5));
    /// func.push(ir::Ins::Ret);
    /// ```
    Ret,

    /// Adds the given value to the last item on the stack. The item must have the given value type, and will maintain that same valuetype.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("add_1", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Inc(ValueType::I32, 1));
    /// func.push(ir::Ins::Ret);
    /// ```
    Inc(ValueType, u64),

    /// Subtracts the given value to the last item on the stack. The item must have the given value type, and will maintain that same valuetype.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("sub_1", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Dec(ValueType::I32, 1));
    /// func.push(ir::Ins::Ret);
    /// ```
    Dec(ValueType, u64),

    /// Adds the last two items on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("add", ir::Signature::new(vec![ ValueType::I32, ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Add(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Add(ValueType),
    
    /// Multiplies the last two items on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("mul", ir::Signature::new(vec![ ValueType::I32, ValueType::I32 ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::Mul(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Mul(ValueType),
    
    /// Divides the second to last item by the last item on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("divide_10_by_5", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 5));
    /// func.push(ir::Ins::Div(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Div(ValueType),
    
    /// Subtracts the last item from the second to last item on the stack, and pushes the result. The two items must both have the given value type, and the result will have the same value type.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("sub_5_from_10", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 10));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 5));
    /// func.push(ir::Ins::Sub(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Sub(ValueType),

    /// Pushes 1 if the last two items on the stack are equal, and 0 otherwise. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 5));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 5));
    /// func.push(ir::Ins::Eq(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Eq(ValueType),

    /// Pushes 0 if the last two items on the stack are equal, and 1 otherwise. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 2));
    /// func.push(ir::Ins::Ne(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Ne(ValueType),

    /// Pushes 1 if the second to last item on the stack is less than the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 2));
    /// func.push(ir::Ins::Lt(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Lt(ValueType),

    /// Pushes 1 if the second to last item on the stack is less than or equal to the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::Le(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Le(ValueType),

    /// Pushes 1 if the second to last item on the stack is greater than the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 2));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::Gt(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Gt(ValueType),

    /// Pushes 1 if the second to last item on the stack is greater than or equal to the last. The two items must both have the given value type, and the result will be a bool.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("always_true", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 2));
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::Ge(ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    Ge(ValueType),

    /// Continues to loop over it's code while it's condition does not evaluate to zero, running inc after each iteration
    /// Initially evaluates the condition. If it is zero, it breaks. Otherwise it runs code, followed by inc, and then re-evaluates.
    /// Code will start and end with an empty stack, condition will start with an empty stack and end with one (bool) value, inc will start and end with an empty stack.
    /// If a continue is reached inside of the loop, inc is executed and the loop starts again.
    /// If a break is reached, the whole loop is stopped immediately.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("loop_5_times", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// let l1 = func.push_local(ir::Local::new(ir::ValueType::I32)); // Allocate local of type i32
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 5));
    /// func.push(ir::Ins::PopLocal(ir::ValueType::I32, l1));
    ///
    /// func.push(ir::Ins::Loop(
    ///     vec![ // Body
    ///         // do nothing
    ///     ],
    ///     vec![ // Condition
    ///         ir::Ins::PushLocal(ir::ValueType::I32, l1),
    ///     ],
    ///     vec![ // Increment (or in this case, decrement)
    ///         ir::Ins::PushLocal(ir::ValueType::I32, l1),
    ///         ir::Ins::Dec(ValueType::I32, 1),
    ///         ir::Ins::PopLocal(ir::ValueType::I32, l1),
    ///     ]
    /// ));
    /// func.push(ir::Ins::Ret);
    /// ```
    Loop(Vec<Ins>, Vec<Ins>, Vec<Ins>), // Code, Condition, Inc

    /// Executes it's children if the popped item is nonzero.
    /// The instruction must start with one (bool) item on the stack, and it's content must end with 0.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("if_1", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::If(vec![
    ///     // Do something
    /// ]));
    /// func.push(ir::Ins::Ret);
    /// ```
    If(Vec<Ins>),

    /// Executes it's first set of children if the popped item is nonzero, otherwise executes it's second set.
    /// The instruction must start with one (bool) item on the stack, and both branches must end with 0.
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("if_1_else", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ValueType::I32, 1));
    /// func.push(ir::Ins::IfElse(vec![
    ///     // Do something
    /// ], vec![
    ///     // Do something else
    /// ]));
    /// func.push(ir::Ins::Ret);
    /// ```
    IfElse(Vec<Ins>, Vec<Ins>),

    /// Breaks out of the loop at the depth above the current instruction given. The depth must refer to a Loop, and must run with an empty stack
    /// # Examples
    /// ```
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
    /// let mut func = ir::Function::new("continue_immediately", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// Infinite loop
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
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 42));
    /// func.push(ir::Ins::Ret);
    /// ```
    PushLiteral(ValueType, u64),

    /// Pop the last item from the stack, regardless of the type. There must be 1 or more items on the stack at this point.
    /// On register based targets this will be removed entirely
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ], vec![ ValueType::I32 ]));
    /// 
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 42));
    /// func.push(ir::Ins::Drop);
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 21));
    /// func.push(ir::Ins::Ret);
    /// ```
    Drop
}