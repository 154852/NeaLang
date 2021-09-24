use std::collections::HashMap;
use ir::{self, ValueType};
use ir2x86::{self, X86ForIRFunction};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("add_10", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    
    // Save param to local
    let l1 = func.push_local(ir::Local::new(ir::ValueType::I32));
    func.push(ir::Ins::PopLocal(ir::ValueType::I32, l1));

    func.push(ir::Ins::PushLocal(ir::ValueType::I32, l1));
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    func.push(ir::Ins::Add(ir::ValueType::I32));
    func.push(ir::Ins::Ret);

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id);
    
    let x86 = func.build_x86(x86::Mode::X8664, &unit);
    let mut raw = Vec::new();

    let mut local_symbol_map = HashMap::new();
    let mut unfilled_local_symbols = Vec::new();

    for ins in x86 {
        ins.encode(&mut raw, &mut local_symbol_map, &mut unfilled_local_symbols);
    }

    x86::UnfilledLocalSymbol::fill(&mut raw, &local_symbol_map, &unfilled_local_symbols);

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}