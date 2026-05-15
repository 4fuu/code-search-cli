use crate::core::symbol::SymbolKind;
use crate::lang::{
    find_container_name, first_line_signature, kind_category, KindCategory, LanguageSupport,
};
use tree_sitter::Node;

pub struct JavaScriptSupport;

const SYMBOLS_QUERY: &str = include_str!("../../queries/javascript/symbols.scm");
const REFERENCES_QUERY: &str = include_str!("../../queries/javascript/references.scm");

const TYPE_REFS: &str = r#"
(new_expression constructor: (identifier) @reference)
"#;

const CALLABLE_REFS: &str = r#"
(call_expression function: (identifier) @reference)
(call_expression function: (member_expression property: (property_identifier) @reference))
(method_definition name: (property_identifier) @reference)
(identifier) @reference
"#;

impl LanguageSupport for JavaScriptSupport {
    fn ts_language(&self) -> tree_sitter::Language {
        tree_sitter_javascript::LANGUAGE.into()
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
        &[Function, Method, Class, Const, Variable]
    }

    fn symbol_kind_from_pattern(&self, pattern_index: usize, node: &Node) -> Option<SymbolKind> {
        match pattern_index {
            0 | 3 => Some(SymbolKind::Function),
            1 => Some(SymbolKind::Class),
            2 => Some(SymbolKind::Method),
            4 | 5 => Some(match declaration_kind(node)? {
                LexicalKind::Const => SymbolKind::Const,
                LexicalKind::Variable => SymbolKind::Variable,
            }),
            _ => None,
        }
    }

    fn extract_signature(&self, node: &Node, source: &str) -> Option<String> {
        first_line_signature(node, source, 200)
    }

    fn is_exported(&self, node: &Node, _source: &str) -> bool {
        if node.kind() == "export_statement" {
            return true;
        }
        let mut current = Some(*node);
        while let Some(node) = current {
            if let Some(parent) = node.parent() {
                if parent.kind() == "export_statement" {
                    return true;
                }
                current = Some(parent);
            } else {
                break;
            }
        }
        false
    }

    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String> {
        find_container_name(node, source, &["class_declaration"], "name")
    }
}

enum LexicalKind {
    Const,
    Variable,
}

fn declaration_kind(node: &Node) -> Option<LexicalKind> {
    let mut current = Some(*node);
    while let Some(node) = current {
        if node.kind() == "lexical_declaration" {
            for idx in 0..node.child_count() {
                if node.child(idx)?.kind() == "const" {
                    return Some(LexicalKind::Const);
                }
            }
            return Some(LexicalKind::Variable);
        }
        if node.kind() == "variable_declaration" {
            return Some(LexicalKind::Variable);
        }
        current = node.parent();
    }
    None
}
