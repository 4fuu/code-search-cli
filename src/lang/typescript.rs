use crate::core::symbol::SymbolKind;
use crate::lang::{
    find_container_name, first_line_signature, kind_category, KindCategory, LanguageSupport,
};
use tree_sitter::Node;

pub struct TypeScriptSupport;

/// Type positions: annotations, generic args, extends/implements, new expressions.
const TYPE_REFS: &str = r#"
(type_identifier) @reference
(new_expression constructor: (identifier) @reference)
"#;

/// Call expressions, identifier for function-as-value, and method definitions.
const CALLABLE_REFS: &str = r#"
(call_expression function: (identifier) @reference)
(call_expression function: (member_expression property: (property_identifier) @reference))
(method_definition name: (property_identifier) @reference)
(identifier) @reference
"#;

impl LanguageSupport for TypeScriptSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    }

    fn symbols_query_source(&self) -> &str {
        include_str!("../../queries/typescript/symbols.scm")
    }

    fn references_query_source(&self) -> &str {
        include_str!("../../queries/typescript/references.scm")
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
            Function, Method, Class, Interface, TypeAlias, Enum, Variable,
        ]
    }

    fn symbol_kind_from_pattern(&self, pattern_index: usize, _node: &Node) -> Option<SymbolKind> {
        match pattern_index {
            0 => Some(SymbolKind::Function),
            1 => Some(SymbolKind::Class),
            2 => Some(SymbolKind::Interface),
            3 => Some(SymbolKind::TypeAlias),
            4 => Some(SymbolKind::Enum),
            5 => Some(SymbolKind::Method),
            6 => Some(SymbolKind::Variable), // export variable
            7 => Some(SymbolKind::Variable), // top-level variable
            _ => None,
        }
    }

    fn extract_signature(&self, node: &Node, source: &str) -> Option<String> {
        first_line_signature(node, source, 200)
    }

    fn is_exported(&self, node: &Node, _source: &str) -> bool {
        // Check if node or parent is an export_statement
        if node.kind() == "export_statement" {
            return true;
        }
        if let Some(parent) = node.parent() {
            if parent.kind() == "export_statement" {
                return true;
            }
            // Check grandparent (for variable_declarator inside lexical_declaration inside export_statement)
            if let Some(grandparent) = parent.parent() {
                if grandparent.kind() == "export_statement" {
                    return true;
                }
            }
        }
        false
    }

    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String> {
        find_container_name(node, source, &["class_declaration"], "name")
    }
}
