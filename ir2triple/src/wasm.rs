pub fn encode(unit: &ir::TranslationUnit, path: &str, relocatable: bool) -> Result<(), String> {
    let module = ir2wasm::TranslationContext::translate_unit(unit)?;

    if !relocatable && module.import_count() != 0 {
        return Err(format!("Could not statically link wasm module, missing {} imports", module.import_count()));
    }
    
    std::fs::write(path, module.encode()).expect("Could not write");

    Ok(())
}