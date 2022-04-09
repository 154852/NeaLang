fn correct() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I32, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32
        ]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        let param2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param2, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    unit.validate().expect("Invalid IR");
}

fn too_few_locals() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            /* ----> */ ir::ValueType::I32, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32
        ]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param1
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::LocalUnderflow);
}

fn incorrect_local_type_access() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I64, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32
        ]));
        
        /* ----> */ let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        let param2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param2, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::LocalIncorrectType);
}

fn out_of_bounds_local_access() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I32, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32
        ]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(/* ----> */ ir::LocalIndex::new(2), ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::LocalDoesNotExist);
}

fn too_small_return() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I32, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32, /* ----> */ ir::ValueType::I32
        ]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        let param2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param2, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::StackUnderflow);
}

fn too_big_return() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I32, ir::ValueType::I32
        ], /* ----> */ vec![]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        let param2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param2, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::StackDepthNotZero);
}

fn invalid_stack_access() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I32, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32
        ]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        let param2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        /* ----> */ func.push(ir::Ins::Add(ir::ValueType::I64)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param2, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::StackIncorrectType);
}

fn stack_underflow() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_math", ir::Signature::new(vec![
            ir::ValueType::I32, ir::ValueType::I32
        ], vec![
            ir::ValueType::I32
        ]));
        
        let param1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
        let param2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        /* ----> */
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 20));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 30));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 20 + 30
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 10 * (20 + 30)
        
        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param1, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // (10 * (20 + 30)) / param1

        func.push(ir::Ins::PushPath(
            ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(param2, ir::StorableType::Value(ir::ValueType::I32))),
            ir::ValueType::I32
        ));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // ((10 * (20 + 30)) / param1) - param2
        
        func.push(ir::Ins::Ret);

        func
    });

    assert_eq!(unit.validate().expect_err("Not incorrect"), ir::ValidationError::StackUnderflow);
}

fn main() {
    correct();
    too_few_locals();
    incorrect_local_type_access();
    out_of_bounds_local_access();
    too_small_return();
    too_big_return();
    invalid_stack_access();
    stack_underflow();
}