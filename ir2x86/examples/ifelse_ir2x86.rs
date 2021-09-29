use ir::{self, ValueType};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("not_eq", ir::Signature::new(vec![ ValueType::I32, ValueType::I32 ], vec![ ValueType::I32 ]));
    
    func.push(ir::Ins::Eq(ValueType::I32));
    func.push(ir::Ins::IfElse(vec![
		ir::Ins::PushLiteral(ir::ValueType::I32, 0),
		ir::Ins::Ret,
	], vec![
		ir::Ins::PushLiteral(ir::ValueType::I32, 1),
		ir::Ins::Ret,
	]));

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id);
    
    let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);
    let mut ins = ctx.translate_function(&func, &unit);
    x86::opt::pass_zero(&mut ins);
    
	let mut ctx = x86::EncodeContext::new();
	ctx.append_function(0, &ins);
	let raw = ctx.finish();
	println!("Assembled!");

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}