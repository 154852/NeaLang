use ir::{self};
use x86;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    unit.add_function(ir::Function::new_extern("nl_new_object", ir::Signature::new(vec![ ir::ValueType::UPtr ], vec![ ir::ValueType::UPtr ])));
    unit.add_function(ir::Function::new_extern("nl_new_slice", ir::Signature::new(vec![ ir::ValueType::UPtr, ir::ValueType::UPtr ], vec![ ir::ValueType::UPtr ])));

    let mut func = ir::Function::new("slices", ir::Signature::new(vec![ ], vec![ ]));
    
    // func.push(ir::Ins::PushLiteral(ir::ValueType::UPtr, 20));
    // func.push(ir::Ins::NewSlice(ir::StorableType::Value(ir::ValueType::I32)));
    // func.push(ir::Ins::PushLiteral(ir::ValueType::UPtr, 3));
    // func.push(ir::Ins::PushSliceElementRef(ir::StorableType::Value(ir::ValueType::I32)));
    // func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 10));
    // func.push(ir::Ins::PopRef(ir::ValueType::I32));
    // func.push(ir::Ins::Ret);

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id).unwrap();
    
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