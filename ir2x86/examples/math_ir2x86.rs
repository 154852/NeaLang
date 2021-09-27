use ir::{self, ValueType};
use ir2x86::X86ForIRFunction;
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("add_5", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    
    // Save param to local
    let l1 = func.push_local(ir::Local::new(ir::ValueType::I32));
    func.push(ir::Ins::PopLocal(ir::ValueType::I32, l1));

    func.push(ir::Ins::PushLocal(ir::ValueType::I32, l1));
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    func.push(ir::Ins::Add(ir::ValueType::I32));
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    func.push(ir::Ins::Sub(ir::ValueType::I32));
    func.push(ir::Ins::Ret);

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id);
    
    let mut ins = func.build_x86(x86::Mode::X8664, &unit);
    x86::opt::pass_zero(&mut ins);
    
	let mut ctx = x86::EncodeContext::new();
	ctx.append_function(0, &ins);
	let raw = ctx.finish();
	println!("Assembled!");

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}