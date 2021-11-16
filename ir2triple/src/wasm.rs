pub fn encode(unit: &ir::TranslationUnit, path: &str, _relocatable: bool) -> Result<(), String> {
    let module = ir2wasm::TranslationContext::translate_unit(unit)?;
    
    std::fs::write(path, module.encode()).expect("Could not write");

    Ok(())
}