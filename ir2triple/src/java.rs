use std::path::PathBuf;

pub fn encode(unit: &ir::TranslationUnit, path: &str, _relocatable: bool) -> Result<(), String> {
    let class = ir2java::TranslationContext::translate_unit(unit, PathBuf::from(path).file_stem().expect("Invalid path").to_str().expect("Invalid name"))?;

    std::fs::write(path, class.encode()).expect("Could not write");

    Ok(())
}