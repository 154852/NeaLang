use ir::{self};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut example_struct = ir::StructContent::new();
    example_struct.push_prop(ir::StructProperty::new("first_field", ir::StorableType::Value(ir::ValueType::I32)));
    example_struct.push_prop(ir::StructProperty::new("second_field", ir::StorableType::Value(ir::ValueType::I32)));

    let example_struct = ir::CompoundType::new("example_struct", ir::TypeContent::Struct(example_struct));
	unit.add_type(example_struct.clone());

    let mut func = ir::Function::new("structs", ir::Signature::new(vec![ ], vec![ ]));
    
	let local = func.push_local(ir::Local::new(ir::StorableType::Compound(example_struct.clone())));
    
	func.push(ir::Ins::PushLocalRef(ir::StorableType::Compound(example_struct.clone()), local));
    func.push(ir::Ins::PushPropertyRef(example_struct.clone(), ir::StorableType::Value(ir::ValueType::I32), 1));
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 32));
    func.push(ir::Ins::PopRef(ir::ValueType::I32));
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
    
	let mut ctx = x86::EncodeContext::new(true);
	ctx.append_function(0, &ins);
	let (raw, _) = ctx.finish();
	println!("Assembled!");

    // View with `objdump -D ir2x86/examples/binary.bin -b binary -m i386 -Mintel,x86-64`
    std::fs::write("ir2x86/examples/binary.bin", &raw).expect("Could not write output");
}