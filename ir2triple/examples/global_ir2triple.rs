use ir;
use ir2triple;

fn main() {
	let mut unit = ir::TranslationUnit::new();

    let func_a_id = unit.add_function(ir::Function::new("main", ir::Signature::new(vec![ ], vec![ ])));

	let global_a = unit.add_global(ir::Global::new("object", ir::StorableType::Value(ir::ValueType::U32), false));

	let func_a = unit.get_function_mut(func_a_id);
    func_a.push(ir::Ins::PushGlobalRef(ir::StorableType::Value(ir::ValueType::U32), global_a));
    func_a.push(ir::Ins::Drop);
    func_a.push(ir::Ins::Ret);

	// Link with `gcc ir2triple/examples/binary.elf ir2triple/examples/entry.c -o ir2triple/examples/out`
	ir2triple::linux_elf::encode(&unit, "ir2triple/examples/binary.elf", true).expect("Could not encode");
}