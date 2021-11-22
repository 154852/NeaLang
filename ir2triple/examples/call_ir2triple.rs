use ir;
use ir2triple;

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let func_a_id = unit.add_function(ir::Function::new("nl_entry", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ])));
    let func_b_id = unit.add_function(ir::Function::new("ret_add_1", ir::Signature::new(vec![ ir::ValueType::I32 ], vec![ ir::ValueType::I32 ])));

    let func_a = unit.get_function_mut(func_a_id).unwrap();
    func_a.push(ir::Ins::PushLiteral(ir::ValueType::I32, 7));
    func_a.push(ir::Ins::Call(func_b_id));
    func_a.push(ir::Ins::Add(ir::ValueType::I32));
    func_a.push(ir::Ins::Ret);

    let func_b = unit.get_function_mut(func_b_id).unwrap();
    func_b.push(ir::Ins::Inc(ir::ValueType::I32, 1));
    func_b.push(ir::Ins::Ret);

    // Link with `gcc ir2triple/examples/binary.elf ir2triple/examples/entry.c -o ir2triple/examples/out`
    ir2triple::linux_elf::encode(&unit, "ir2triple/examples/binary.elf", true).expect("Could not encode");
}