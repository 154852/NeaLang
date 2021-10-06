use ir;
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("convert", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I64 ]));
    
    func.push(ir::Ins::Convert(ir::ValueType::I32, ir::ValueType::I64));
    func.push(ir::Ins::Ret);

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id);
    
    // let mut ins = func.build_x86(x86::Mode::X8664, &unit);
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