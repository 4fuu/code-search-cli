use crate::core::symbol::SymbolKind;
use crate::lang::{
    find_container_name, first_line_signature, kind_category, KindCategory, LanguageSupport,
};
use tree_sitter::Node;

pub struct PhpSupport;

const SYMBOLS_QUERY: &str = include_str!("../../queries/php/symbols.scm");
const REFERENCES_QUERY: &str = include_str!("../../queries/php/references.scm");

const CALLABLE_REFS: &str = r#"
(function_call_expression function: [(qualified_name (name) @reference) (name) @reference])
(scoped_call_expression name: (name) @reference)
(member_call_expression name: (name) @reference)
(method_declaration name: (name) @reference)
"#;

impl LanguageSupport for PhpSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_php::LANGUAGE_PHP.into()
    }

    fn symbols_query_source(&self) -> &str {
        SYMBOLS_QUERY
    }

    fn references_query_source(&self) -> &str {
        REFERENCES_QUERY
    }

    fn references_query_for_kind(&self, kind: SymbolKind) -> &str {
        match kind_category(kind) {
            KindCategory::Callable => CALLABLE_REFS,
            KindCategory::Type | KindCategory::Other => REFERENCES_QUERY,
        }
    }

    fn supported_kinds(&self) -> &'static [SymbolKind] {
        use SymbolKind::*;
        &[Module, Interface, Class, Function, Method, Const, Variable]
    }

    fn symbol_kind_from_pattern(&self, pattern_index: usize, _node: &Node) -> Option<SymbolKind> {
        match pattern_index {
            0 => Some(SymbolKind::Module),
            1 | 2 => Some(SymbolKind::Interface),
            3 => Some(SymbolKind::Class),
            4 => Some(SymbolKind::Function),
            5 => Some(SymbolKind::Method),
            6 => Some(SymbolKind::Const),
            7 => Some(SymbolKind::Variable),
            _ => None,
        }
    }

    fn extract_signature(&self, node: &Node, source: &str) -> Option<String> {
        first_line_signature(node, source, 200)
    }

    fn is_exported(&self, node: &Node, source: &str) -> bool {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        !text.trim_start().starts_with("private ")
    }

    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String> {
        find_container_name(
            node,
            source,
            &[
                "class_declaration",
                "interface_declaration",
                "trait_declaration",
            ],
            "name",
        )
    }
}
