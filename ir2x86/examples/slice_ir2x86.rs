use ir::{self};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("slices", ir::Signature::new(vec![
        ir::ValueType::Ref(Box::new(ir::StorableType::Slice(Box::new(ir::StorableType::Value(ir::ValueType::I32)))))
    ], vec![ ir::ValueType::UPtr ]));
    
    
	func.push(ir::Ins::PushSliceLen(ir::StorableType::Value(ir::ValueType::I32)));
	func.push(ir::Ins::Ret);

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
	let (raw, _) = ctx.finish();
	println!("Assembled!");

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}