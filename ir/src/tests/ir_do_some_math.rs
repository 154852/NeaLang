use crate::*;

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