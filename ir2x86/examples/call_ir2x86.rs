use ir::{self, ValueType};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let func_a_id = unit.add_function(ir::Function::new("call", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ])));
    let func_b_id = unit.add_function(ir::Function::new("ret_add_1", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ])));

	let func_a = unit.get_function_mut(func_a_id);
    func_a.push(ir::Ins::Call(func_b_id));
    func_a.push(ir::Ins::Ret);

	let func_b = unit.get_function_mut(func_b_id);
    func_b.push(ir::Ins::Inc(ir::ValueType::I32, 1));
	func_b.push(ir::Ins::Ret);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let ctx = ir2x86::TranslationContext::new(x86::Mode::X8664);
	let mut encode_ctx = x86::EncodeContext::new();

	let mut ins_a = ctx.translate_function(unit.get_function(func_a_id), &unit);
    x86::opt::pass_zero(&mut ins_a);
	encode_ctx.append_function(func_a_id, &ins_a);

	let mut ins_b = ctx.translate_function(unit.get_function(func_b_id), &unit);
    x86::opt::pass_zero(&mut ins_b);
	encode_ctx.append_function(func_b_id, &ins_b);
    
	let raw = encode_ctx.finish();
	println!("Assembled!");

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}