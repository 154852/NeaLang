use ir::{self, ValueType};

fn main() {
    let mut unit = ir::TranslationUnit::new();

    let mut func = ir::Function::new("add_10", ir::Signature::new(vec![ ValueType::I32 ], vec![ ValueType::I32 ]));
    
    // Save param to local
    let l1 = func.push_local(ir::Local::new(ir::StorableType::Value(ir::ValueType::I32)));
    func.push(ir::Ins::Pop(ir::ObjectSource::Local(l1), ir::ValueType::I32));

    func.push(ir::Ins::Push(ir::ObjectSource::Local(l1), ir::ValueType::I32));
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    func.push(ir::Ins::PushLiteral(ir::ValueType::I32, 5));
    func.push(ir::Ins::Add(ir::ValueType::I32));
    func.push(ir::Ins::Add(ir::ValueType::I32));
    func.push(ir::Ins::Ret);

    let func_id = unit.add_function(func);

    match unit.validate() {
        Ok(_) => println!("Validated!"),
        Err(e) => panic!("Validation error: {:#?}", e)
    }

    let func = unit.get_function(func_id);
    
    match func.evaluate(&unit, vec![ ir::StackElement::new_num(7, ValueType::I32) ]) {
        Ok(res) => println!("Res: {:#?}", res),
        Err(e) => panic!("Evaluation error: {:#?}", e)
    }
}