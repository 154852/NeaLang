fn correct() {
    let mut unit = ir::TranslationUnit::new();
    unit.add_function({
        let mut func = ir::Function::new("do_some_control_flow", ir::Signature::new(vec![
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

fn main() {
    correct();
}