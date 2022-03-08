use crate::{irgen::IrGenError, lexer::TokenStream};

use super::TopLevelNode;

#[derive(Debug)]
pub struct TranslationUnit {
    pub nodes: Vec<TopLevelNode>
}

impl TranslationUnit {
    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<TranslationUnit> {
        let mut nodes = Vec::new();

        while !stream.finished() {
            nodes.push(syntax::ex!(syntax::parse!(stream, TopLevelNode::parse), stream.error("Expected a function")));
        }

        syntax::MatchResult::Ok(TranslationUnit {
            nodes
        })
    }
}

impl TranslationUnit {
    /// Defines this unit in the ir, but does not append function code
    pub fn to_extern_ir_on(&self, unit: &mut ir::TranslationUnit, target_arch_name: &str) -> Result<(), IrGenError> {
        // 1. Declare all the types - must be done first so function signatures can use these types
        for node in &self.nodes {
            match node {
                TopLevelNode::StructDeclaration(decl) => {
                    let ct = decl.to_ir(unit, self)?;
                    unit.add_type(ct);
                },
                _ => {}
            }
        }

        // 2. Then insert function bases
        for node in &self.nodes {
            match node {
                TopLevelNode::Function(func) => {
                    // Filter out functions not of the correct arch
                    if !func.arch_matches(target_arch_name)? { continue; }
                    
                    let mut func = func.to_ir_base(unit, self)?;
                    func.set_extern();
                    unit.add_function(func);
                },
                _ => {}
            }
        }

        Ok(())
    }

    /// Both defines the unit and appends function code - to_extern_ir_on should *not* have been called first.
    pub fn to_ir_on(&self, unit: &mut ir::TranslationUnit, target_arch_name: &str) -> Result<(), IrGenError> {
        // 1. Declare all the types - must be done first so function signatures can use these types
        for node in &self.nodes {
            match node {
                TopLevelNode::StructDeclaration(decl) => {
                    let ct = decl.to_ir(unit, self)?;
                    unit.add_type(ct);
                },
                _ => {}
            }
        }

        // 2. Add the function bases - must be done before adding code so that the code can use other functions
        let mut first_index = None;
        for node in &self.nodes {
            match node {
                TopLevelNode::Function(func) => {
                    // Filter out functions not of the correct arch
                    if !func.arch_matches(target_arch_name)? { continue; }

                    let func = func.to_ir_base(unit, self)?;
                    let idx = unit.add_function(func);
                    if first_index.is_none() {
                        first_index = Some(idx.idx());
                    }
                },
                _ => {}
            }
        }

        // 3. Then add code
        let mut id = 0;
        for node in &self.nodes {
            match node {
                TopLevelNode::Function(func) => {
                    if !func.arch_matches(target_arch_name)? { continue; }

                    if func.code.is_some() {
                        // Safe to unwrap as this wouldn't be running otherwise
                        func.append_ir(unit, ir::FunctionIndex::new(id + first_index.unwrap()))?;
                    }
                    id += 1;
                },
                _ => {}
            }
        }

        Ok(())
    }
}