use std::collections::HashMap;
use ir::{self, ValueType};
use ir2x86::{self, X86ForIRFunction};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("return_3_things", ir::Signature::new(vec![ ], vec![ ValueType::I32, ValueType::I32, ValueType::I32 ]));
    
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
	func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 11));
	func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 12));
    func.push(ir::Ins::Ret);

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id);
    
    let mut x86 = func.build_x86(x86::Mode::X8664, &unit);
    x86::opt::pass_zero(&mut x86);
    let mut raw = Vec::new();

    let mut local_symbol_map = HashMap::new();
    let mut unfilled_local_symbols = Vec::new();

    for ins in x86 {
        ins.encode(&mut raw, &mut local_symbol_map, &mut unfilled_local_symbols);
    }

    x86::Relocation::fill(&mut raw, &local_symbol_map, &unfilled_local_symbols);

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");

    println!("Assembled!");
}