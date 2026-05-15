use crate::core::symbol::SymbolKind;
use crate::lang::{
    find_container_name, first_line_signature, kind_category, KindCategory, LanguageSupport,
};
use tree_sitter::Node;

pub struct JavaSupport;

const SYMBOLS_QUERY: &str = include_str!("../../queries/java/symbols.scm");
const REFERENCES_QUERY: &str = include_str!("../../queries/java/references.scm");

const TYPE_REFS: &str = r#"
(type_identifier) @reference
"#;

const CALLABLE_REFS: &str = r#"
(method_invocation name: (identifier) @reference)
(method_declaration name: (identifier) @reference)
(constructor_declaration name: (identifier) @reference)
"#;

impl LanguageSupport for JavaSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_java::LANGUAGE.into()
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
        &[Method, Class, Interface, Enum, Const, Variable]
    }

    fn symbol_kind_from_pattern(&self, pattern_index: usize, node: &Node) -> Option<SymbolKind> {
        match pattern_index {
            0 | 3 => Some(SymbolKind::Class),
            1 => Some(SymbolKind::Interface),
            2 => Some(SymbolKind::Enum),
            4 | 5 => Some(SymbolKind::Method),
            6 => Some(if has_modifier(node, "final") {
                SymbolKind::Const
            } else {
                SymbolKind::Variable
            }),
            _ => None,
        }
    }

    fn extract_signature(&self, node: &Node, source: &str) -> Option<String> {
        first_line_signature(node, source, 200)
    }

    fn is_exported(&self, node: &Node, _source: &str) -> bool {
        has_modifier(node, "public")
    }

    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String> {
        find_container_name(
            node,
            source,
            &[
                "class_declaration",
                "interface_declaration",
                "enum_declaration",
                "record_declaration",
            ],
            "name",
        )
    }
}

fn has_modifier(node: &Node, modifier: &str) -> bool {
    (0..node.child_count())
        .filter_map(|idx| node.child(idx))
        .find(|child| child.kind() == "modifiers")
        .map(|mods| {
            (0..mods.child_count())
                .filter_map(|idx| mods.child(idx))
                .any(|child| child.kind() == modifier)
        })
        .unwrap_or(false)
}
