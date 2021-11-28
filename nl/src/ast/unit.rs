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
    pub fn to_extern_ir_on(&self, unit: &mut ir::TranslationUnit, target_arch_name: &str) -> Result<(), IrGenError> {
        for node in &self.nodes {
            match node {
                TopLevelNode::StructDeclaration(decl) => {
                    let ct = decl.to_ir(unit, self)?;
                    unit.add_type(ct);
                },
                _ => {}
            }
        }

        for node in &self.nodes {
            match node {
                TopLevelNode::Function(func) => {
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

    pub fn to_ir_on(&self, unit: &mut ir::TranslationUnit, target_arch_name: &str) -> Result<(), IrGenError> {
        for node in &self.nodes {
            match node {
                TopLevelNode::StructDeclaration(decl) => {
                    let ct = decl.to_ir(unit, self)?;
                    unit.add_type(ct);
                },
                _ => {}
            }
        }

        let mut first_index = None;
        for node in &self.nodes {
            match node {
                TopLevelNode::Function(func) => {
                    if !func.arch_matches(target_arch_name)? { continue; }

                    let func = func.to_ir_base(unit, self)?;
                    let idx = unit.add_function(func);
                    if first_index.is_none() {
                        first_index = Some(idx);
                    }
                },
                _ => {}
            }
        }

        let mut id = 0;
        for node in &self.nodes {
            match node {
                TopLevelNode::Function(func) => {
                    if !func.arch_matches(target_arch_name)? { continue; }

                    if func.code.is_some() {
                        // Safe to unwrap as this wouldn't be running otherwise
                        func.append_ir(unit, id + first_index.unwrap())?;
                    }
                    id += 1;
                },
                _ => {}
            }
        }

        Ok(())
    }
}