use crate::core::symbol::SymbolKind;
use crate::lang::{
    find_container_name, first_line_signature, kind_category, KindCategory, LanguageSupport,
};
use tree_sitter::Node;

pub struct RubySupport;

const SYMBOLS_QUERY: &str = include_str!("../../queries/ruby/symbols.scm");
const REFERENCES_QUERY: &str = include_str!("../../queries/ruby/references.scm");

const TYPE_REFS: &str = r#"
(constant) @reference
"#;

const CALLABLE_REFS: &str = r#"
(call method: (identifier) @reference)
(identifier) @reference
"#;

impl LanguageSupport for RubySupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_ruby::LANGUAGE.into()
    }

    fn symbols_query_source(&self) -> &str {
        SYMBOLS_QUERY
    }

    fn references_query_source(&self) -> &str {
        REFERENCES_QUERY
    }

    fn references_query_for_kind(&self, kind: SymbolKind) -> &str {
        match kind_category(kind) {
            KindCategory::Type => TYPE_REFS,
            KindCategory::Callable => CALLABLE_REFS,
            KindCategory::Other => REFERENCES_QUERY,
        }
    }

    fn supported_kinds(&self) -> &'static [SymbolKind] {
        use SymbolKind::*;
        &[Method, Class, Module, Const, Variable]
    }

    fn symbol_kind_from_pattern(&self, pattern_index: usize, _node: &Node) -> Option<SymbolKind> {
        match pattern_index {
            0 => Some(SymbolKind::Module),
            1 => Some(SymbolKind::Class),
            2 | 3 => Some(SymbolKind::Method),
            4 => Some(SymbolKind::Const),
            5 => Some(SymbolKind::Variable),
            _ => None,
        }
    }

    fn extract_signature(&self, node: &Node, source: &str) -> Option<String> {
        first_line_signature(node, source, 200)
    }

    fn is_exported(&self, node: &Node, source: &str) -> bool {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
            return !name.starts_with('_');
        }
        if node.kind() == "assignment" {
            if let Some(left) = node.child_by_field_name("left") {
                let name = left.utf8_text(source.as_bytes()).unwrap_or("");
                return !name.starts_with('_');
            }
        }
        true
    }

    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String> {
        find_container_name(node, source, &["class", "module"], "name")
    }
}
