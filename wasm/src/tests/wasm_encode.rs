use crate::*;

#[test]
fn module() {
    let mut module = Module::new();
    module.add_type(FunctionType::new(vec![ ValType::Num(NumType::I32) ], vec![ ValType::Num(NumType::I32) ]));
    module.add_function(0);
    module.add_code(Code::new(vec![ ValType::Num(NumType::I32) ], Expr::with(vec![
        Ins::LocalGet(0),
        Ins::ConstI32(42),
        Ins::Add(NumType::I32),
        Ins::Return
    ])));

    assert_eq!(module.encode(), std::fs::read("src/tests/module.wasm").expect("Could not read module.wasm"));
}

#[test]
fn imports_and_exports() {
    let mut module = Module::new();
    module.add_type(FunctionType::new(vec![ ValType::Num(NumType::I32), ValType::Num(NumType::I32) ], vec![ ValType::Num(NumType::I32) ]));
    module.add_type(FunctionType::new(vec![ ValType::Num(NumType::I32) ], vec![ ValType::Num(NumType::I32) ]));
    module.add_import(Import::new("std", "add", ImportDescriptor::Type(0)));
    module.add_export(Export::new("add_12", ExportDescriptor::Func(1)));
    module.add_function(1);
    module.add_code(Code::new(vec![ ValType::Num(NumType::I32) ], Expr::with(vec![
        Ins::LocalGet(0),
        Ins::ConstI32(12),
        Ins::Call(0),
        Ins::Return
    ])));

    assert_eq!(module.encode(), std::fs::read("src/tests/imports_and_exports.wasm").expect("Could not read imports_and_exports.wasm"));
}