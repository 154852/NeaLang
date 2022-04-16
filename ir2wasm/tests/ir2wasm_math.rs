use ir;
use ir2wasm;

#[test]
fn main() {
    let mut unit = ir::TranslationUnit::new();

    let putchar = unit.add_function({
        let mut func = ir::Function::new_extern("putchar", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![]));
        func.push_attr(ir::FunctionAttr::ExternLocation("core".to_string()));
        func
    });

    unit.add_function({
        let mut func = ir::Function::new("main", ir::Signature::new(vec![ ], vec![ ]));
    
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 1));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 3));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 2 + 3 = 5
        func.push(ir::Ins::Mul(ir::ValueType::I32)); // 1 * 5 = 5
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 2));
        func.push(ir::Ins::Div(ir::ValueType::I32)); // 5 / 2 = 2 (integer divide)
        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, (-1 as i64) as u64));
        func.push(ir::Ins::Sub(ir::ValueType::I32)); // 2 - -1 = 3

        func.push(ir::Ins::PushLiteral(ir::ValueType::I32, '0' as u64));
        func.push(ir::Ins::Add(ir::ValueType::I32)); // 3 + '0' = '3'
        func.push(ir::Ins::Call(putchar));
        func.push(ir::Ins::Ret);

        func
    });

    unit.validate().expect("Invalid unit");

    let module = ir2wasm::TranslationContext::translate_unit(&unit).expect("Translation");

    let raw = module.encode();
    std::fs::write("tests/math.wasm", &raw).expect("Could not write output");

    let output = std::process::Command::new("node")
        .arg("tests/wasm.js")
        .arg("tests/math.wasm")
        .output().expect("Could not run");
    
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, "3".as_bytes());
}