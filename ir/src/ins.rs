use crate::{CompoundTypeRef, PropertyIndex, StorableType, unit::*};

#[derive(Debug)]
pub enum Ins {
    /// Pushes the local at the given index to the stack. The local must match be a value, and must match in value type to the ValueType given
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    /// 
    /// // Save param to local
    /// let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32))); // Allocate local of type i32
    /// func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, l1)); // Save the given param to the local
    /// 
    /// func.push(ir::Ins::PushLocalValue(ir::ValueType::I32, l1)); // Push the local back onto the stack
    /// func.push(ir::Ins::Ret);
    /// ```
    PushLocalValue(ValueType, LocalIndex),

    /// Pushes a reference the local at the given index to the stack. The local must have the given storable type.
    /// Note: Returning a local ref can cause severe problems in some architectures
    /// # Examples
    /// ```
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ], vec![ ]));
    /// 
    /// // Save param to local
    /// let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32))); // Allocate local of type i32
    /// func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, l1)); // Save the given param to the local
    /// 
    /// func.push(ir::Ins::PushLocalRef(ir::StorableType::Value(ir::ValueType::I32), l1));
    /// func.push(ir::Ins::Drop);
    ///
    /// func.push(ir::Ins::Ret);
    /// ```
    PushLocalRef(StorableType, LocalIndex),
    
    /// Pops the last value on the stack to the local at the given index. The local must be a value, and the local and popped item must match in value type to the ValueType given
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    /// 
    /// // Save param to local
    /// let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32))); // Allocate local of type i32
    /// func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, l1)); // Save the given param to the local
    /// 
    /// func.push(ir::Ins::PushLocalValue(ir::ValueType::I32, l1)); // Push the local back onto the stack
    /// func.push(ir::Ins::Ret);
    /// ```
    PopLocalValue(ValueType, LocalIndex),

    /// Pops the value followd by the ref from the stack, and writes the value to the ref. The ref must be a value of the given ValueType, which must be the valuetype of the popped value.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut func = ir::Function::new("do_nothing", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ]));
    /// 
    /// // Save param to local
    /// let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32))); // Allocate local of type i32
    /// func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, l1)); // Save the given param to the local
    /// 
    /// func.push(ir::Ins::PushLocalRef(ir::StorableType::Value(ir::ValueType::I32), l1)); // Push a reference to the local onto the stack
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    /// func.push(ir::Ins::PopRef(ir::ValueType::I32));
    /// func.push(ir::Ins::Ret);
    /// ```
    PopRef(ValueType),

    /// Pop the compound type ref of the given type from the stack, and push the value of the field at the given index in that struct. The field must have the given valuetype.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut example_struct = ir::StructContent::new();
    /// example_struct.push_prop(ir::StructProperty::new("first_field", ir::StorableType::Value(ir::ValueType::I32)));
    /// example_struct.push_prop(ir::StructProperty::new("second_field", ir::StorableType::Value(ir::ValueType::I32)));
    /// 
    /// let example_struct = ir::CompoundType::new("example_struct", ir::TypeContent::Struct(example_struct));
    /// 
    /// let mut func = ir::Function::new("structs", ir::Signature::new(vec![ ], vec![ ir::ValueType::I32 ]));
    /// 
	/// let local = func.push_local(ir::Local::new(ir::StorableType::Compound(example_struct.clone())));
    /// 
	/// func.push(ir::Ins::PushLocalRef(ir::StorableType::Compound(example_struct.clone()), local));
    /// func.push(ir::Ins::PushProperty(example_struct.clone(), ir::ValueType::I32, 1)); // Second field
	/// func.push(ir::Ins::Ret);
    /// ```
    PushProperty(CompoundTypeRef, ValueType, PropertyIndex),

    /// Pop the compound type ref of the given type from the stack, and push a reference to the value of the field at the given index in that struct. The field must have the given storabletype.
    /// # Examples
    /// ```
    /// use ir;
    /// let mut example_struct = ir::StructContent::new();
    /// example_struct.push_prop(ir::StructProperty::new("first_field", ir::StorableType::Value(ir::ValueType::I32)));
    /// example_struct.push_prop(ir::StructProperty::new("second_field", ir::StorableType::Value(ir::ValueType::I32)));
    /// 
    /// let example_struct = ir::CompoundType::new("example_struct", ir::TypeContent::Struct(example_struct));
    /// 
    /// let mut func = ir::Function::new("structs", ir::Signature::new(vec![ ], vec![ ]));
    /// 
	/// let local = func.push_local(ir::Local::new(ir::StorableType::Compound(example_struct.clone())));
    /// 
	/// func.push(ir::Ins::PushLocalRef(ir::StorableType::Compound(example_struct.clone()), local));
    /// func.push(ir::Ins::PushPropertyRef(example_struct.clone(), ir::StorableType::Value(ir::ValueType::I32), 0)); // First field
    /// func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    /// func.push(ir::Ins::PopRef(ir::ValueType::I32));
	/// func.push(ir::Ins::Ret);
    /// ```
    PushPropertyRef(CompoundTypeRef, StorableType, PropertyIndex),

    /// Pop a reference to a slice from the stack, and push it's length as a usize. The slice must be of the given storable type.
    PushSliceLen(StorableType),

    /// Pop a uptr index from the stack, pop a reference to a slice from the stack, and push the element in the slice at that index. The slice must be of the given type, and the pushed value will have that type. The slice type must be a value.
    PushSliceElement(StorableType),

    /// Pop a uptr index from the stack, pop a reference to a slice from the stack, and push a reference to the element in the slice at that index. The slice must be of the given type, and the pushed value will have that type.
    PushSliceElementRef(StorableType),

    /// Push a reference to the global at the given index, which must have the given storable type
    PushGlobalRef(StorableType, GlobalIndex),

    /// Allocates a value of the given type, and pushes a reference to it
    New(StorableType),

    /// Pops a uptr, then allocates a slice of the given type of the given length, and pushes a reference to it
    NewSlice(StorableType),

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
    Loop(Vec<Ins>, Vec<Ins>, Vec<Ins>), // Code, Condition, Inc

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
    If(Vec<Ins>),

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
    IfElse(Vec<Ins>, Vec<Ins>),

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