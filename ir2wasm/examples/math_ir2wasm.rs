use ir;
use ir2wasm;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("ret", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ]));
    func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
    func.push(ir::Ins::PushPath(ir::ValuePath::new_origin_only(ir::ValuePathOrigin::Local(0, ir::StorableType::Value(ir::ValueType::I32))), ir::ValueType::I32));
    func.push(ir::Ins::Push(ir::ValueType::I32));
    func.push(ir::Ins::Ret);

    unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let module = match ir2wasm::TranslationContext::translate_unit(&unit) {
        Ok(m) => m,
        Err(e) => panic!("TranslationError: {}", e)
    };

    let raw = module.encode();
    std::fs::write("ir2wasm/examples/binary.wasm", &raw).expect("Could not write output");
}