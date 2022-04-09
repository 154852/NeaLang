use crate::*;

/// Normal / boundary test - Verify that a valid program containing loops is considered valid by the checker
#[test]
fn loop_correct() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("factorial", Signature::new(vec![
            ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let value = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        
        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::PushLiteral(ValueType::I32, 1));
        func.push(Ins::Pop(ValueType::I32));

        func.push(Ins::Loop(vec![ // Body
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::Mul(ValueType::I32),

            Ins::Pop(ValueType::I32)
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Gt(ValueType::I32)
        ], vec![ // Increment
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            Ins::Dec(ValueType::I32, 1),
            
            Ins::Pop(ValueType::I32),
        ]));

        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Ret);

        func
    });

    unit.validate().expect("Invalid IR");
}

/// Erroneous test - Verify that a program which attempts to use a number as appose to a boolean for a condition is considered invalid
#[test]
fn loop_not_a_bool_cond() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("factorial", Signature::new(vec![
            ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let value = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        
        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::PushLiteral(ValueType::I32, 1));
        func.push(Ins::Pop(ValueType::I32));

        func.push(Ins::Loop(vec![ // Body
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::Mul(ValueType::I32),

            Ins::Pop(ValueType::I32)
        ], vec![ // Condition
            /* ---> */ Ins::PushLiteral(ValueType::I32, 1), // Always true, but an i32
        ], vec![ // Increment
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            Ins::Dec(ValueType::I32, 1),
            
            Ins::Pop(ValueType::I32),
        ]));

        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackIncorrectType);
}

/// Erroneous test - Verify that a program which attempts to loop with a non-empty stack is considered invalid
#[test]
fn loop_not_empty_stack() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("factorial", Signature::new(vec![
            ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let value = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        
        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::PushLiteral(ValueType::I32, 1));
        func.push(Ins::Pop(ValueType::I32));

        /* ---> */ func.push(Ins::PushLiteral(ValueType::I32, 1));

        func.push(Ins::Loop(vec![ // Body
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::Mul(ValueType::I32),

            Ins::Pop(ValueType::I32)
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Gt(ValueType::I32)
        ], vec![ // Increment
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            Ins::Dec(ValueType::I32, 1),
            
            Ins::Pop(ValueType::I32),
        ]));

        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackDepthNotZero);
}

/// Erroneous test - Verify that a program which attempts to loop within it's body with a non-empty stack is considered invalid
#[test]
fn loop_not_empty_stack_after_body() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("factorial", Signature::new(vec![
            ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let value = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        
        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::PushLiteral(ValueType::I32, 1));
        func.push(Ins::Pop(ValueType::I32));

        func.push(Ins::Loop(vec![ // Body
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::Mul(ValueType::I32),

            Ins::Pop(ValueType::I32),

            /* ---> */ Ins::PushLiteral(ValueType::I32, 1)
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Gt(ValueType::I32)
        ], vec![ // Increment
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            Ins::Dec(ValueType::I32, 1),
            
            Ins::Pop(ValueType::I32),
        ]));

        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackDepthNotZero);
}

/// Erroneous test - Verify that a program with more than one value in a loop condition is considered invalid
#[test]
fn loop_not_one_after_condition() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("factorial", Signature::new(vec![
            ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let value = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        
        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::PushLiteral(ValueType::I32, 1));
        func.push(Ins::Pop(ValueType::I32));

        func.push(Ins::Loop(vec![ // Body
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::Mul(ValueType::I32),

            Ins::Pop(ValueType::I32)
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Gt(ValueType::I32),
            /* ---> */ Ins::PushLiteral(ValueType::Bool, 1)
        ], vec![ // Increment
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            Ins::Dec(ValueType::I32, 1),
            
            Ins::Pop(ValueType::I32),
        ]));

        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackDepthNotOne);
}

/// Erroneous test - Verify that a program which does not terminate with a return is considered invalid
#[test]
fn loop_missing_ret() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("factorial", Signature::new(vec![
            ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let value = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        
        func.push(Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32));
        func.push(Ins::PushLiteral(ValueType::I32, 1));
        func.push(Ins::Pop(ValueType::I32));

        func.push(Ins::Loop(vec![ // Body
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(value, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::Mul(ValueType::I32),

            Ins::Pop(ValueType::I32)
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Gt(ValueType::I32),
        ], vec![ // Increment
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),
            Ins::Dec(ValueType::I32, 1),
            
            Ins::Pop(ValueType::I32),
        ]));

        /* ---> no return */

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::NoFinalReturn);
}

/// Normal / boundary test - Verify that a valid program containing nested if/if-else statements are considered correct
#[test]
fn if_correct() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("scaling", Signature::new(vec![
            ValueType::I32, ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::IfElse(vec![ // True
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 2),
            Ins::Mul(ValueType::I32),
            Ins::Ret
        ], vec![ // False
            Ins::If(vec![ // True
                Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
                Ins::Push(ValueType::I32),
    
                Ins::PushLiteral(ValueType::I32, 9),
                Ins::Mul(ValueType::I32),
                Ins::Ret
            ], vec![ // Condition
                Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))), ValueType::I32),
                Ins::Push(ValueType::I32),

                Ins::PushLiteral(ValueType::I32, 2),

                Ins::Eq(ValueType::I32)
            ]),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 12),
            Ins::Mul(ValueType::I32),
            Ins::Ret
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Eq(ValueType::I32)
        ]));

        // Note that the lack of return is OK since all control flow paths in the above if statements terminate with a ret

        func
    });

    unit.validate().expect("Invalid IR");
}

/// Erroneous / boundary test - Verify that a program containing control paths without return statements is considered invalid
#[test]
fn if_no_return() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("scaling", Signature::new(vec![
            ValueType::I32, ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::IfElse(vec![ // True
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 2),
            Ins::Mul(ValueType::I32),
            Ins::Ret
        ], vec![ // False
            Ins::If(vec![ // True
                Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
                Ins::Push(ValueType::I32),
    
                Ins::PushLiteral(ValueType::I32, 9),
                Ins::Mul(ValueType::I32),
                Ins::Ret
            ], vec![ // Condition
                Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))), ValueType::I32),
                Ins::Push(ValueType::I32),

                Ins::PushLiteral(ValueType::I32, 2),

                Ins::Eq(ValueType::I32)
            ]),

            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 12),
            Ins::Mul(ValueType::I32),
            Ins::Drop // don't return
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Eq(ValueType::I32)
        ]));

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::NoFinalReturn);
}

/// Erroneous - Verify that entering an if statement with a non-empty stack is erroneous
#[test]
fn if_non_empty_stack() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("scaling", Signature::new(vec![
            ValueType::I32, ValueType::I32,
        ], vec![
            ValueType::I32,
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::IfElse(vec![ // True
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 2),
            Ins::Mul(ValueType::I32),
            Ins::Ret
        ], vec![ // False
            /* ---> */ Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))), ValueType::I32),
            /* ---> */ Ins::Push(ValueType::I32),

            Ins::If(vec![ // True    
                Ins::PushLiteral(ValueType::I32, 9),
                Ins::Mul(ValueType::I32),
                Ins::Ret
            ], vec![ // Condition
                Ins::PushLiteral(ValueType::I32, 2),

                Ins::Eq(ValueType::I32)
            ]),

            Ins::PushLiteral(ValueType::I32, 12),
            Ins::Mul(ValueType::I32),
            Ins::Ret
        ], vec![ // Condition
            Ins::PushPath(ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))), ValueType::I32),
            Ins::Push(ValueType::I32),

            Ins::PushLiteral(ValueType::I32, 1),

            Ins::Eq(ValueType::I32)
        ]));

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackDepthNotZero);
}