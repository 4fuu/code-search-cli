use crate::core::symbol::SymbolKind;
use crate::lang::{kind_category, KindCategory, LanguageSupport};
use tree_sitter::Node;

pub struct GoSupport;

const SYMBOLS_QUERY: &str = include_str!("../../queries/go/symbols.scm");
const REFERENCES_QUERY: &str = include_str!("../../queries/go/references.scm");

/// Named type positions only.
const TYPE_REFS: &str = r#"
(type_identifier) @reference
"#;

/// Call expressions plus identifier for function-as-value.
const CALLABLE_REFS: &str = r#"
(call_expression function: (identifier) @reference)
(call_expression function: (selector_expression field: (field_identifier) @reference))
(identifier) @reference
"#;

impl LanguageSupport for GoSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_go::LANGUAGE.into()
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
        &[
            Function, Method, Struct, Interface, TypeAlias, Const, Variable,
        ]
    }

    fn symbol_kind_from_pattern(
        &self,
        pattern_index: usize,
        def_node: &Node,
    ) -> Option<SymbolKind> {
        // Pattern order from queries/go/symbols.scm:
        // 0: function_declaration
        // 1: method_declaration
        // 2: type_declaration (struct, interface, alias - distinguished by type child)
        // 3: const
        // 4: var
        match pattern_index {
            0 => Some(SymbolKind::Function),
            1 => Some(SymbolKind::Method),
            2 => {
                // Inspect the "type" field of the type_spec node
                if let Some(type_node) = def_node.child_by_field_name("type") {
                    match type_node.kind() {
                        "struct_type" => Some(SymbolKind::Struct),
                        "interface_type" => Some(SymbolKind::Interface),
                        _ => Some(SymbolKind::TypeAlias),
                    }
                } else {
                    Some(SymbolKind::TypeAlias)
                }
            }
            3 => Some(SymbolKind::Const),
            4 => Some(SymbolKind::Variable),
            _ => None,
        }
    }

    fn extract_signature(&self, def_node: &Node, source: &str) -> Option<String> {
        let text = def_node.utf8_text(source.as_bytes()).ok()?;
        let first_line = text.lines().next()?;
        let sig = first_line.trim_end().trim_end_matches('{').trim_end();
        if sig.is_empty() {
            None
        } else {
            Some(sig.to_string())
        }
    }

    fn extract_container_name(&self, def_node: &Node, source: &str) -> Option<String> {
        // For methods, the receiver type is the container
        if def_node.kind() == "method_declaration" {
            let receiver = def_node.child_by_field_name("receiver")?;
            let text = receiver.utf8_text(source.as_bytes()).ok()?;
            // Extract type name from receiver like (r *Router) -> Router
            let name = text
                .trim_matches(|c| c == '(' || c == ')')
                .split_whitespace()
                .last()?
                .trim_start_matches('*');
            return Some(name.to_string());
        }
        None
    }

    fn is_exported(&self, def_node: &Node, source: &str) -> bool {
        // In Go, exported names start with an uppercase letter.
        let name_text = if let Some(name_node) = def_node.child_by_field_name("name") {
            name_node.utf8_text(source.as_bytes()).unwrap_or("")
        } else {
            return true;
        };
        name_text.starts_with(|c: char| c.is_uppercase())
    }
}
