use ir::{self, ValueType};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

	// value = param
	// counter = 5
	// while counter != 0
	//  value = value * counter;
	//  counter -= 1
	// return value
    let mut func = ir::Function::new("factorial_5", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));

	let value = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
	func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, value));

	let counter = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
	func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
	func.push(ir::Ins::PopLocalValue(ir::ValueType::I32, counter));
    
    func.push(ir::Ins::Loop(
		vec![ // Body
			ir::Ins::PushLocalValue(ir::ValueType::I32, value),
			ir::Ins::PushLocalValue(ir::ValueType::I32, counter),
			ir::Ins::Mul(ir::ValueType::I32),
			ir::Ins::PopLocalValue(ir::ValueType::I32, value)
		],
		vec![ // Condition
			ir::Ins::PushLocalValue(ir::ValueType::I32, counter)
		],
		vec![ // Increment
			ir::Ins::PushLocalValue(ir::ValueType::I32, counter),
			ir::Ins::Dec(ir::ValueType::I32, 1),
			ir::Ins::PopLocalValue(ir::ValueType::I32, counter)
		]
	));

	func.push(ir::Ins::PushLocalValue(ir::ValueType::I32, value));
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
	ctx.append_function(&ins);
	let (raw, _) = ctx.take();
	println!("Assembled!");

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}