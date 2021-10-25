use wasm;

fn main() {
    let mut module = wasm::Module::new();

    let func = module.add_type(wasm::FunctionType::new(vec![], vec![wasm::ValType::Num(wasm::NumType::I32)]));
    module.add_function(func);

    module.add_code(wasm::Code::new(vec![], wasm::Expr::with(vec![
        wasm::Ins::ConstI32(42),
        wasm::Ins::Return
    ])));

    let raw = module.encode();
    std::fs::write("wasm/examples/binary.wasm", &raw).expect("Could not write output");
}