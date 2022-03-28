use std::path::PathBuf;

pub fn encode(unit: &ir::TranslationUnit, path: &str, _relocatable: bool) -> Result<(), String> {
    let path = PathBuf::from(path);
    let stem = path.file_stem().expect("Invalid path").to_str().expect("Invalid name");
    
    let class = ir2java::TranslationContext::translate_unit(unit, stem)?;
    std::fs::write(format!("{}.class", stem), class.encode()).expect("Could not write");

    for (secondary_name, secondary_class) in ir2java::TranslationContext::translate_unit_types(unit, &class, stem)? {
        std::fs::write(format!("{}.class", secondary_name), secondary_class.encode()).expect("Could not write");
    }

    Ok(())
}