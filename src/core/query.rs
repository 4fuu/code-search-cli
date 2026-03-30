use crate::core::language::Language;
use crate::core::symbol::SymbolKind;
use crate::lang::get_language_support;
use anyhow::{Context, Result};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

/// A reference occurrence found in source code.
#[derive(Debug, Clone)]
pub struct Reference {
    /// Path relative to repo root.
    pub path: String,
    pub line: usize,
    pub column: usize,
    /// The matched source text snippet for context.
    pub context: String,
}

/// Find references to a symbol name in a source file using Tree-sitter queries.
/// When `kind` is provided a narrower, kind-aware query is used instead of the
/// generic one, reducing false positives.
pub fn find_references(
    source: &str,
    language: Language,
    name: &str,
    rel_path: &str,
    kind: Option<SymbolKind>,
) -> Result<Vec<Reference>> {
    let support = get_language_support(language);

    let mut parser = Parser::new();
    parser.set_language(&support.ts_language())?;
    let tree = parser.parse(source, None).context("failed to parse")?;

    let query_source = match kind {
        Some(k) => support.references_query_for_kind(k),
        None => support.references_query_source(),
    };
    let query = tree_sitter::Query::new(&support.ts_language(), query_source)
        .map_err(|e| anyhow::anyhow!("reference query error: {}", e))?;

    let ref_idx = query
        .capture_index_for_name("reference")
        .context("missing @reference capture")?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut refs = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            if capture.index != ref_idx {
                continue;
            }
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            if text != name {
                continue;
            }

            let line = node.start_position().row;
            let context = lines.get(line).unwrap_or(&"").trim().to_string();

            refs.push(Reference {
                path: rel_path.to_string(),
                line: line + 1,
                column: node.start_position().column,
                context,
            });
        }
    }

    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::language::Language;
    use std::path::Path;

    #[test]
    fn rust_references() {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.rs"),
        )
        .unwrap();
        let refs = find_references(
            &source,
            Language::Rust,
            "Parser",
            "tests/fixtures/sample.rs",
            None,
        )
        .unwrap();
        assert!(!refs.is_empty(), "expected references to Parser");
        for r in &refs {
            assert_eq!(r.path, "tests/fixtures/sample.rs");
            assert!(r.line > 0);
            assert!(!r.context.is_empty());
        }
    }

    #[test]
    fn go_references() {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.go"),
        )
        .unwrap();
        let refs = find_references(
            &source,
            Language::Go,
            "Server",
            "tests/fixtures/sample.go",
            None,
        )
        .unwrap();
        assert!(!refs.is_empty(), "expected references to Server");
    }

    #[test]
    fn ts_references() {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.ts"),
        )
        .unwrap();
        let refs = find_references(
            &source,
            Language::TypeScript,
            "EventEmitter",
            "tests/fixtures/sample.ts",
            None,
        )
        .unwrap();
        assert!(!refs.is_empty(), "expected references to EventEmitter");
    }

    #[test]
    fn py_references() {
        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.py"),
        )
        .unwrap();
        let refs = find_references(
            &source,
            Language::Python,
            "Config",
            "tests/fixtures/sample.py",
            None,
        )
        .unwrap();
        assert!(!refs.is_empty(), "expected references to Config");
    }

    #[test]
    fn no_references_for_nonexistent() {
        let source = "fn main() { println!(\"hello\"); }";
        let refs = find_references(
            source,
            Language::Rust,
            "nonexistent_symbol",
            "test.rs",
            None,
        )
        .unwrap();
        assert!(refs.is_empty());
    }
}
