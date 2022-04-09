use crate::*;

/// Normal test - Verify that a valid program is considered valid by the checker
#[test]
fn correct() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I32, ValueType::I32
        ], vec![
            ValueType::I32
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    unit.validate().expect("Invalid IR");
}

/// Erroneous test - Verify that a program where not all parameter locals are initialised in considered invalid and given the correct error
#[test]
fn too_few_locals() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            /* ----> */ ValueType::I32, ValueType::I32
        ], vec![
            ValueType::I32
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param1
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::LocalUnderflow);
}

/// Erroneous test - Verify that a program where an attempt to access a local with an incorrect given type is considered invalid
#[test]
fn incorrect_local_type_access() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I64, ValueType::I32
        ], vec![
            ValueType::I32
        ]));
        
        /* ----> */ let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::LocalIncorrectType);
}

/// Erroneous / boundary test - Verify that a program where an attempt to access a local which does not exist is considered invalid
#[test]
fn out_of_bounds_local_access() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I32, ValueType::I32
        ], vec![
            ValueType::I32
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(/* ----> */ LocalIndex::new(2), StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::LocalDoesNotExist);
}

/// Erroneous / boundary test - Verify that a program with one too few return values compared to it's signature is considered invalid
#[test]
fn too_small_return() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I32, ValueType::I32
        ], vec![
            ValueType::I32, /* ----> */ ValueType::I32
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackUnderflow);
}

/// Erroneous / boundary test - Verify that a program with one too many return values compared to it's signature is considered invalid
#[test]
fn too_big_return() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I32, ValueType::I32
        ], /* ----> */ vec![]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackDepthNotZero);
}

/// Erroneous test - Verify that a program which attempts to take a an item off the stack with the incorrect type is considered invalid
#[test]
fn invalid_stack_access() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I32, ValueType::I32
        ], vec![
            ValueType::I32
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        func.push(Ins::PushLiteral(ValueType::I32, 10));
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        /* ----> */ func.push(Ins::Add(ValueType::I64)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackIncorrectType);
}

/// Erroneous test - Verify that a program which attempts to operate on more items than there are on the stack is considered invalid
#[test]
fn stack_underflow() {
    let mut unit = TranslationUnit::new();
    unit.add_function({
        let mut func = Function::new("do_some_math", Signature::new(vec![
            ValueType::I32, ValueType::I32
        ], vec![
            ValueType::I32
        ]));
        
        let param1 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));
        let param2 = func.push_local(Local::new(StorableType::Value(ValueType::I32)));

        /* ----> */
        func.push(Ins::PushLiteral(ValueType::I32, 20));
        func.push(Ins::PushLiteral(ValueType::I32, 30));
        func.push(Ins::Add(ValueType::I32)); // 20 + 30
        func.push(Ins::Mul(ValueType::I32)); // 10 * (20 + 30)
        
        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param1, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Div(ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(Ins::PushPath(
            ValuePath::new_origin_only(ValuePathOrigin::Local(param2, StorableType::Value(ValueType::I32))),
            ValueType::I32
        ));
        func.push(Ins::Push(ValueType::I32));
        func.push(Ins::Sub(ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ValidationError::StackUnderflow);
}