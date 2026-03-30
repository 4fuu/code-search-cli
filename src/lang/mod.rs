pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

use crate::core::language::Language;
use crate::core::symbol::SymbolKind;
use tree_sitter::Node;

pub fn get_language_support(lang: Language) -> Box<dyn LanguageSupport> {
    match lang {
        Language::Rust => Box::new(rust::RustSupport),
        Language::TypeScript => Box::new(typescript::TypeScriptSupport),
        Language::Python => Box::new(python::PythonSupport),
        Language::Go => Box::new(go::GoSupport),
    }
}

/// Broad category used to select the right reference query.
pub enum KindCategory {
    /// struct / enum / trait / interface / class / type_alias / impl
    Type,
    /// function / method
    Callable,
    /// Everything else (const, static, variable, module)
    Other,
}

pub fn kind_category(kind: SymbolKind) -> KindCategory {
    match kind {
        SymbolKind::Struct
        | SymbolKind::Enum
        | SymbolKind::Trait
        | SymbolKind::Interface
        | SymbolKind::Class
        | SymbolKind::TypeAlias
        | SymbolKind::Impl => KindCategory::Type,
        SymbolKind::Function | SymbolKind::Method => KindCategory::Callable,
        _ => KindCategory::Other,
    }
}

pub trait LanguageSupport {
    /// Return the tree-sitter Language for this language.
    fn ts_language(&self) -> tree_sitter::Language;

    /// Return the symbols.scm query source.
    fn symbols_query_source(&self) -> &str;

    /// Return the references.scm query source.
    fn references_query_source(&self) -> &str;

    /// Return a kind-aware references query source.
    /// Default falls back to the generic query.
    fn references_query_for_kind(&self, kind: SymbolKind) -> &str {
        let _ = kind;
        self.references_query_source()
    }

    /// Return the set of symbol kinds this language can produce.
    fn supported_kinds(&self) -> &'static [SymbolKind];

    /// Map a tree-sitter pattern index to a SymbolKind.
    fn symbol_kind_from_pattern(&self, pattern_index: usize, node: &Node) -> Option<SymbolKind>;

    /// Extract a signature string from a definition node.
    fn extract_signature(&self, node: &Node, source: &str) -> Option<String>;

    /// Determine if a symbol is exported/public.
    fn is_exported(&self, node: &Node, source: &str) -> bool;

    /// Extract container name (e.g., impl Foo → "Foo", class Bar → "Bar").
    fn extract_container_name(&self, node: &Node, source: &str) -> Option<String>;
}

/// Return true if the given language can produce symbols of the given kind.
pub fn language_supports_kind(lang: Language, kind: SymbolKind) -> bool {
    get_language_support(lang).supported_kinds().contains(&kind)
}

/// Get the first line of a node's source text, trim trailing `{` and whitespace,
/// and limit to `max_len` characters.
pub(crate) fn first_line_signature(node: &Node, source: &str, max_len: usize) -> Option<String> {
    let text = source.get(node.start_byte()..node.end_byte())?;
    let line = text.lines().next().unwrap_or("");
    let trimmed = line.trim_end_matches('{').trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() > max_len {
        Some(format!("{}…", &trimmed[..max_len]))
    } else {
        Some(trimmed.to_string())
    }
}

/// Walk up the parent chain looking for a node of one of the given kinds,
/// then extract the name child from it.
pub(crate) fn find_container_name(
    node: &Node,
    source: &str,
    container_kinds: &[&str],
    name_field: &str,
) -> Option<String> {
    let mut current = node.parent()?;
    loop {
        if container_kinds.contains(&current.kind()) {
            if let Some(name_node) = current.child_by_field_name(name_field) {
                let name = source.get(name_node.start_byte()..name_node.end_byte())?;
                return Some(name.to_string());
            }
        }
        current = current.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::language::Language;

    #[test]
    fn get_language_support_returns_correct_types() {
        let _ = get_language_support(Language::Rust);
        let _ = get_language_support(Language::TypeScript);
        let _ = get_language_support(Language::Python);
        let _ = get_language_support(Language::Go);
    }
}
