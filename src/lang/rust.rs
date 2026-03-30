use crate::core::symbol::SymbolKind;
use crate::lang::{first_line_signature, kind_category, KindCategory, LanguageSupport};
use tree_sitter::Node;

pub struct RustSupport;

/// Only type-position nodes: annotations, bounds, impl targets, and qualified
/// paths like `Language::Rust` (where `Language` is the path prefix).
const TYPE_REFS: &str = r#"
(type_identifier) @reference
(scoped_identifier path: (identifier) @reference)
"#;

/// Call sites (direct, method, scoped) plus bare identifier for function-as-value.
const CALLABLE_REFS: &str = r#"
(call_expression function: (identifier) @reference)
(call_expression function: (field_expression field: (field_identifier) @reference))
(call_expression function: (scoped_identifier name: (identifier) @reference))
(identifier) @reference
"#;

impl LanguageSupport for RustSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn symbols_query_source(&self) -> &str {
        include_str!("../../queries/rust/symbols.scm")
    }

    fn references_query_source(&self) -> &str {
        include_str!("../../queries/rust/references.scm")
    }

    fn references_query_for_kind(&self, kind: SymbolKind) -> &str {
        match kind_category(kind) {
            KindCategory::Type => TYPE_REFS,
            KindCategory::Callable => CALLABLE_REFS,
            KindCategory::Other => self.references_query_source(),
        }
    }

    fn supported_kinds(&self) -> &'static [SymbolKind] {
        use SymbolKind::*;
        &[
            Function, Method, Struct, Enum, Trait, Impl, TypeAlias, Const, Static, Module,
        ]
    }

    fn symbol_kind_from_pattern(&self, pattern_index: usize, _node: &Node) -> Option<SymbolKind> {
        match pattern_index {
            0 => Some(SymbolKind::Function),
            1 => Some(SymbolKind::Method),
            2 => Some(SymbolKind::Struct),
            3 => Some(SymbolKind::Enum),
            4 => Some(SymbolKind::Trait),
            5 => Some(SymbolKind::Impl),
            6 => Some(SymbolKind::TypeAlias),
            7 => Some(SymbolKind::Const),
            8 => Some(SymbolKind::Static),
            9 => Some(SymbolKind::Module),
            _ => None,
        }
    }

    fn extract_signature(&self, node: &Node, source: &str) -> Option<String> {
        first_line_signature(node, source, 200)
    }

    fn is_exported(&self, node: &Node, source: &str) -> bool {
        let text = source.get(node.start_byte()..node.end_byte()).unwrap_or("");
        if text.starts_with("pub ") || text.starts_with("pub(") {
            return true;
        }
        // Check parent for visibility (e.g., pub struct wrapping)
        if let Some(parent) = node.parent() {
            let parent_text = source
                .get(parent.start_byte()..parent.end_byte())
                .unwrap_or("");
            if parent_text.starts_with("pub ") || parent_text.starts_with("pub(") {
                return true;
            }
        }
        false
    }

    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String> {
        let mut current = node.parent()?;
        loop {
            if current.kind() == "impl_item" {
                // Extract the type from the impl item
                if let Some(type_node) = current.child_by_field_name("type") {
                    let name = source.get(type_node.start_byte()..type_node.end_byte())?;
                    return Some(name.to_string());
                }
            }
            current = current.parent()?;
        }
    }
}
