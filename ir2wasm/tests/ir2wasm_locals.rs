use ir;
use ir2wasm;

#[test]
fn values() {
    let mut unit = ir::TranslationUnit::new();

    let putchar = unit.add_function({
        let mut func = ir::Function::new_extern("putchar", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![]));
        func.push_attr(ir::FunctionAttr::ExternLocation("core".to_string()));
        func
    });

    unit.add_function({
        let mut func = ir::Function::new("main", ir::Signature::new(vec![ ], vec![ ]));

        let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I64)));
        let l2 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));

        func.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(l1, ir::StorableType::Value(ir::ValueType::I64))), ir::ValueType::I64));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I64, 4));
        func.push(ir::Ins::Pop(ir::ValueType::I64));

        func.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(l2, ir::StorableType::Value(ir::ValueType::I32))), ir::ValueType::I32));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 8));
        func.push(ir::Ins::Pop(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(l1, ir::StorableType::Value(ir::ValueType::I64))), ir::ValueType::I64));
        func.push(ir::Ins::Push(ir::ValueType::I64));
        func.push(ir::Ins::Convert(ir::ValueType::I64, ir::ValueType::I32));
        func.push(ir::Ins::Neg(ir::ValueType::I32));
        
        func.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(l2, ir::StorableType::Value(ir::ValueType::I32))), ir::ValueType::I32));
        func.push(ir::Ins::Push(ir::ValueType::I32));
        
        func.push(ir::Ins::Add(ir::ValueType::I32)); // -4 + 8 = 4

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, '0' as u64));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 4 + '0' = '4'
        func.push(ir::Ins::Call(putchar));
        func.push(ir::Ins::Ret);

        func
    });

    unit.validate().expect("Invalid unit");

    let module = ir2wasm::TranslationContext::translate_unit(&unit).expect("Translation");

    let raw = module.encode();
    std::fs::write("tests/locals.wasm", &raw).expect("Could not write output");

    let output = std::process::Command::new("node")
        .arg("tests/wasm.js")
        .arg("tests/locals.wasm")
        .output().expect("Could not run");
    
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, "4".as_bytes());
}

#[test]
fn compounds() {
    let mut unit = ir::TranslationUnit::new();

    let putchar = unit.add_function({
        let mut func = ir::Function::new_extern("putchar", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![]));
        func.push_attr(ir::FunctionAttr::ExternLocation("core".to_string()));
        func
    });

    let compound_type_a = ir::CompoundType::new("CompoundTypeA", ir::CompoundContent::Struct({
        let mut content = ir::StructContent::new();
        content.push_prop(ir::StructProperty::new("a", ir::StorableType::Value(ir::ValueType::I32)));
        content.push_prop(ir::StructProperty::new("b", ir::StorableType::Value(ir::ValueType::I32)));
        content
    }));
    unit.add_type(compound_type_a.clone());

    let compound_type_b = ir::CompoundType::new("CompoundTypeB", ir::CompoundContent::Struct({
        let mut content = ir::StructContent::new();
        content.push_prop(ir::StructProperty::new("a", ir::StorableType::Value(ir::ValueType::I32)));
        content.push_prop(ir::StructProperty::new("b", ir::StorableType::Compound(compound_type_a.clone())));
        content
    }));
    unit.add_type(compound_type_b.clone());

    unit.add_function({
        let mut func = ir::Function::new("main", ir::Signature::new(vec![ ], vec![ ]));

        let l1 = func.push_local(ir::Local::new(ir::StorableType::Compound(compound_type_a.clone())));
        let l2 = func.push_local(ir::Local::new(ir::StorableType::Compound(compound_type_b.clone())));

        // Initialise all the local fields

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l1, ir::StorableType::Compound(compound_type_a.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(0), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
        func.push(ir::Ins::Pop(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l1, ir::StorableType::Compound(compound_type_a.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
        func.push(ir::Ins::Pop(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l2, ir::StorableType::Compound(compound_type_b.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(0), compound_type_b.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 3));
        func.push(ir::Ins::Pop(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l2, ir::StorableType::Compound(compound_type_b.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_b.clone(), ir::StorableType::Compound(compound_type_a.clone())),
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(0), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 4));
        func.push(ir::Ins::Pop(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l2, ir::StorableType::Compound(compound_type_b.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_b.clone(), ir::StorableType::Compound(compound_type_a.clone())),
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
        func.push(ir::Ins::Pop(ir::ValueType::I32));

        // Push their values back on
        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l2, ir::StorableType::Compound(compound_type_b.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_b.clone(), ir::StorableType::Compound(compound_type_a.clone())),
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::Push(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l2, ir::StorableType::Compound(compound_type_b.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(0), compound_type_b.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::Push(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l1, ir::StorableType::Compound(compound_type_a.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::Push(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l1, ir::StorableType::Compound(compound_type_a.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(0), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::Push(ir::ValueType::I32));

        func.push(ir::Ins::PushPath(ir::ValuePath::new(
            ir::ValuePathOrigin::Local(l2, ir::StorableType::Compound(compound_type_b.clone())),
            vec![
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(1), compound_type_b.clone(), ir::StorableType::Compound(compound_type_a.clone())),
                ir::ValuePathComponent::Property(ir::PropertyIndex::new(0), compound_type_a.clone(), ir::StorableType::Value(ir::ValueType::I32))
            ]
        ), ir::ValueType::I32));
        func.push(ir::Ins::Push(ir::ValueType::I32));

        func.push(ir::Ins::Mul(ir::ValueType::I32));
        func.push(ir::Ins::Mul(ir::ValueType::I32));
        func.push(ir::Ins::Mul(ir::ValueType::I32));
        func.push(ir::Ins::Mul(ir::ValueType::I32));

        // Result should be 1*2*3*4*5 = 120

        func.push(ir::Ins::Call(putchar));
        func.push(ir::Ins::Ret);

        func
    });

    unit.validate().expect("Invalid unit");

    let module = ir2wasm::TranslationContext::translate_unit(&unit).expect("Translation");

    let raw = module.encode();
    std::fs::write("tests/compounds.wasm", &raw).expect("Could not write output");

    let output = std::process::Command::new("node")
        .arg("tests/wasm.js")
        .arg("tests/compounds.wasm")
        .output().expect("Could not run");
    
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, &[120]);
}