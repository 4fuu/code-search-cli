use crate::core::symbol::SymbolKind;
use crate::lang::LanguageSupport;
use tree_sitter::Node;

pub struct PythonSupport;

const SYMBOLS_QUERY: &str = include_str!("../../queries/python/symbols.scm");
const REFERENCES_QUERY: &str = include_str!("../../queries/python/references.scm");

impl LanguageSupport for PythonSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn symbols_query_source(&self) -> &str {
        SYMBOLS_QUERY
    }

    fn references_query_source(&self) -> &str {
        REFERENCES_QUERY
    }

    fn supported_kinds(&self) -> &'static [SymbolKind] {
        use SymbolKind::*;
        &[Function, Method, Class, Variable]
    }

    fn symbol_kind_from_pattern(
        &self,
        pattern_index: usize,
        _def_node: &Node,
    ) -> Option<SymbolKind> {
        // Pattern order from queries/python/symbols.scm:
        // 0: function_definition (top-level)
        // 1: class_definition
        // 2: function_definition inside class (method)
        // 3: module-level assignment
        match pattern_index {
            0 => Some(SymbolKind::Function),
            1 => Some(SymbolKind::Class),
            2 => Some(SymbolKind::Method),
            3 => Some(SymbolKind::Variable),
            _ => None,
        }
    }

    fn extract_signature(&self, def_node: &Node, source: &str) -> Option<String> {
        let text = def_node.utf8_text(source.as_bytes()).ok()?;
        let first_line = text.lines().next()?;
        let sig = first_line.trim_end().trim_end_matches(':').trim_end();
        if sig.is_empty() {
            None
        } else {
            Some(sig.to_string())
        }
    }

    fn extract_container_name(&self, def_node: &Node, source: &str) -> Option<String> {
        let parent = def_node.parent()?;
        if parent.kind() == "block" {
            let grandparent = parent.parent()?;
            if grandparent.kind() == "class_definition" {
                let name_node = grandparent.child_by_field_name("name")?;
                return Some(name_node.utf8_text(source.as_bytes()).ok()?.to_string());
            }
        }
        None
    }

    fn is_exported(&self, def_node: &Node, source: &str) -> bool {
        // In Python, names starting with _ are conventionally private.
        if let Some(name_node) = def_node.child_by_field_name("name") {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
            return !name.starts_with('_');
        }
        // For assignments, check the left side
        if def_node.kind() == "assignment" {
            if let Some(left) = def_node.child_by_field_name("left") {
                let name = left.utf8_text(source.as_bytes()).unwrap_or("");
                return !name.starts_with('_');
            }
        }
        true
    }
}
