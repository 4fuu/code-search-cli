use crate::core::language::Language;
use crate::core::symbol::{Symbol, TextRange};
use crate::lang::get_language_support;
use anyhow::{Context, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

struct CompiledSymbolsQuery {
    query: Query,
    name_idx: u32,
    def_idx: u32,
}

impl CompiledSymbolsQuery {
    fn compile(language: Language) -> Self {
        let support = get_language_support(language);
        let ts_language = support.ts_language();
        let query = Query::new(&ts_language, support.symbols_query_source())
            .unwrap_or_else(|e| panic!("query error for {language}: {e}"));
        let name_idx = query
            .capture_index_for_name("name")
            .unwrap_or_else(|| panic!("missing @name capture for {language}"));
        let def_idx = query
            .capture_index_for_name("definition")
            .unwrap_or_else(|| panic!("missing @definition capture for {language}"));
        Self {
            query,
            name_idx,
            def_idx,
        }
    }
}

fn compiled_symbols_query(language: Language) -> &'static CompiledSymbolsQuery {
    static RUST_QUERY: OnceLock<CompiledSymbolsQuery> = OnceLock::new();
    static TYPESCRIPT_QUERY: OnceLock<CompiledSymbolsQuery> = OnceLock::new();
    static PYTHON_QUERY: OnceLock<CompiledSymbolsQuery> = OnceLock::new();
    static GO_QUERY: OnceLock<CompiledSymbolsQuery> = OnceLock::new();

    match language {
        Language::Rust => RUST_QUERY.get_or_init(|| CompiledSymbolsQuery::compile(Language::Rust)),
        Language::TypeScript => {
            TYPESCRIPT_QUERY.get_or_init(|| CompiledSymbolsQuery::compile(Language::TypeScript))
        }
        Language::Python => {
            PYTHON_QUERY.get_or_init(|| CompiledSymbolsQuery::compile(Language::Python))
        }
        Language::Go => GO_QUERY.get_or_init(|| CompiledSymbolsQuery::compile(Language::Go)),
    }
}

thread_local! {
    static PARSERS: RefCell<HashMap<Language, Parser>> = RefCell::new(HashMap::new());
}

/// Parse a single file and extract symbols.
pub fn parse_file(rel_path: &Path, source: &str, language: Language) -> Result<Vec<Symbol>> {
    let support = get_language_support(language);
    let compiled_query = compiled_symbols_query(language);
    let tree = PARSERS.with(|parsers| -> Result<_> {
        let mut parsers = parsers.borrow_mut();
        let parser = parsers.entry(language).or_insert_with(|| {
            let mut parser = Parser::new();
            parser
                .set_language(&support.ts_language())
                .expect("failed to set parser language");
            parser
        });
        parser.parse(source, None).context("failed to parse source")
    })?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&compiled_query.query, tree.root_node(), source.as_bytes());

    let path_str = rel_path.to_string_lossy().replace('\\', "/");
    let mut symbols = Vec::new();
    // Track (start_byte, end_byte) of definition nodes we've already added
    // to handle overlapping patterns (e.g., a fn matched as both function and method).
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    while let Some(m) = matches.next() {
        let mut name_node = None;
        let mut def_node = None;

        for capture in m.captures {
            if capture.index == compiled_query.name_idx {
                name_node = Some(capture.node);
            } else if capture.index == compiled_query.def_idx {
                def_node = Some(capture.node);
            }
        }

        let (name_n, def_n) = match (name_node, def_node) {
            (Some(n), Some(d)) => (n, d),
            _ => continue,
        };

        let def_key = (def_n.start_byte(), def_n.end_byte());
        if !seen.insert(def_key) {
            continue;
        }

        let name = name_n
            .utf8_text(source.as_bytes())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }

        let kind = match support.symbol_kind_from_pattern(m.pattern_index, &def_n) {
            Some(k) => k,
            None => continue,
        };

        let signature = support.extract_signature(&def_n, source);
        let container_name = support.extract_container_name(&def_n, source);
        let exported = support.is_exported(&def_n, source);

        symbols.push(Symbol {
            name,
            kind,
            language,
            path: path_str.clone(),
            container_name,
            signature,
            range: TextRange {
                line: def_n.start_position().row + 1,
                column: def_n.start_position().column,
                end_line: def_n.end_position().row + 1,
                end_column: def_n.end_position().column,
            },
            selection_range: TextRange {
                line: name_n.start_position().row + 1,
                column: name_n.start_position().column,
                end_line: name_n.end_position().row + 1,
                end_column: name_n.end_position().column,
            },
            exported,
        });
    }

    Ok(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::language::Language;
    use crate::core::symbol::SymbolKind;

    fn parse_fixture(filename: &str, lang: Language) -> Vec<Symbol> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(filename);
        let source = std::fs::read_to_string(&path).unwrap();
        parse_file(
            Path::new(&format!("tests/fixtures/{}", filename)),
            &source,
            lang,
        )
        .unwrap()
    }

    fn find_symbol<'a>(symbols: &'a [Symbol], name: &str) -> Option<&'a Symbol> {
        symbols.iter().find(|s| s.name == name)
    }

    // --- Rust ---
    #[test]
    fn rust_structs() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "Parser").unwrap();
        assert_eq!(s.kind, SymbolKind::Struct);
        assert!(s.exported);
        let s = find_symbol(&syms, "Token").unwrap();
        assert_eq!(s.kind, SymbolKind::Struct);
    }

    #[test]
    fn rust_enums() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "Error").unwrap();
        assert_eq!(s.kind, SymbolKind::Enum);
        let s = find_symbol(&syms, "TokenKind").unwrap();
        assert_eq!(s.kind, SymbolKind::Enum);
    }

    #[test]
    fn rust_functions() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "tokenize").unwrap();
        assert_eq!(s.kind, SymbolKind::Function);
        assert!(s.exported);
        assert!(s.signature.as_deref().unwrap().contains("fn tokenize"));
    }

    #[test]
    fn rust_trait() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "Parseable").unwrap();
        assert_eq!(s.kind, SymbolKind::Trait);
    }

    #[test]
    fn rust_const_static() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "MAX_SIZE").unwrap();
        assert_eq!(s.kind, SymbolKind::Const);
        let s = find_symbol(&syms, "COUNTER").unwrap();
        assert_eq!(s.kind, SymbolKind::Static);
    }

    #[test]
    fn rust_type_alias() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "Result").unwrap();
        assert_eq!(s.kind, SymbolKind::TypeAlias);
    }

    #[test]
    fn rust_module() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let s = find_symbol(&syms, "utils").unwrap();
        assert_eq!(s.kind, SymbolKind::Module);
    }

    #[test]
    fn rust_methods_have_container() {
        let syms = parse_fixture("sample.rs", Language::Rust);
        let methods: Vec<_> = syms
            .iter()
            .filter(|s| s.kind == SymbolKind::Method && s.name == "new")
            .collect();
        assert!(!methods.is_empty());
        assert_eq!(methods[0].container_name.as_deref(), Some("Parser"));
    }

    // --- TypeScript ---
    #[test]
    fn ts_class() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "EventEmitter").unwrap();
        assert_eq!(s.kind, SymbolKind::Class);
        assert!(s.exported);
    }

    #[test]
    fn ts_interface() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "Logger").unwrap();
        assert_eq!(s.kind, SymbolKind::Interface);
        assert!(s.exported);
    }

    #[test]
    fn ts_enum() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "LogLevel").unwrap();
        assert_eq!(s.kind, SymbolKind::Enum);
    }

    #[test]
    fn ts_function() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "createLogger").unwrap();
        assert_eq!(s.kind, SymbolKind::Function);
        assert!(s.exported);
    }

    #[test]
    fn ts_not_exported() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "internalHelper").unwrap();
        assert_eq!(s.kind, SymbolKind::Function);
        assert!(!s.exported);
    }

    #[test]
    fn ts_exported_const() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "VERSION").unwrap();
        assert_eq!(s.kind, SymbolKind::Variable);
        assert!(s.exported);
    }

    #[test]
    fn ts_type_alias() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let s = find_symbol(&syms, "Result").unwrap();
        assert_eq!(s.kind, SymbolKind::TypeAlias);
    }

    #[test]
    fn ts_methods_have_container() {
        let syms = parse_fixture("sample.ts", Language::TypeScript);
        let methods: Vec<_> = syms
            .iter()
            .filter(|s| s.kind == SymbolKind::Method && s.name == "on")
            .collect();
        assert!(!methods.is_empty());
        assert_eq!(methods[0].container_name.as_deref(), Some("EventEmitter"));
    }

    // --- Python ---
    #[test]
    fn py_class() {
        let syms = parse_fixture("sample.py", Language::Python);
        let s = find_symbol(&syms, "Config").unwrap();
        assert_eq!(s.kind, SymbolKind::Class);
        assert!(s.exported);
    }

    #[test]
    fn py_private_class() {
        let syms = parse_fixture("sample.py", Language::Python);
        let s = find_symbol(&syms, "_InternalCache").unwrap();
        assert_eq!(s.kind, SymbolKind::Class);
        assert!(!s.exported);
    }

    #[test]
    fn py_function() {
        let syms = parse_fixture("sample.py", Language::Python);
        let s = find_symbol(&syms, "create_config").unwrap();
        assert_eq!(s.kind, SymbolKind::Function);
    }

    #[test]
    fn py_method_container() {
        let syms = parse_fixture("sample.py", Language::Python);
        let m = syms
            .iter()
            .find(|s| s.name == "validate" && s.kind == SymbolKind::Method)
            .unwrap();
        assert_eq!(m.container_name.as_deref(), Some("Config"));
    }

    #[test]
    fn py_module_variable() {
        let syms = parse_fixture("sample.py", Language::Python);
        let s = find_symbol(&syms, "MAX_RETRIES").unwrap();
        assert_eq!(s.kind, SymbolKind::Variable);
    }

    // --- Go ---
    #[test]
    fn go_struct() {
        let syms = parse_fixture("sample.go", Language::Go);
        let s = find_symbol(&syms, "Server").unwrap();
        assert_eq!(s.kind, SymbolKind::Struct);
        assert!(s.exported);
    }

    #[test]
    fn go_interface() {
        let syms = parse_fixture("sample.go", Language::Go);
        let s = find_symbol(&syms, "Handler").unwrap();
        assert_eq!(s.kind, SymbolKind::Interface);
    }

    #[test]
    fn go_function() {
        let syms = parse_fixture("sample.go", Language::Go);
        let s = find_symbol(&syms, "NewServer").unwrap();
        assert_eq!(s.kind, SymbolKind::Function);
        assert!(s.exported);
    }

    #[test]
    fn go_unexported_method() {
        let syms = parse_fixture("sample.go", Language::Go);
        let s = find_symbol(&syms, "handleRequest").unwrap();
        assert_eq!(s.kind, SymbolKind::Method);
        assert!(!s.exported);
    }

    #[test]
    fn go_method_container() {
        let syms = parse_fixture("sample.go", Language::Go);
        let m = syms
            .iter()
            .find(|s| s.name == "Register" && s.kind == SymbolKind::Method)
            .unwrap();
        assert_eq!(m.container_name.as_deref(), Some("Server"));
    }

    #[test]
    fn go_const_var() {
        let syms = parse_fixture("sample.go", Language::Go);
        let s = find_symbol(&syms, "MaxRetries").unwrap();
        assert_eq!(s.kind, SymbolKind::Const);
        let s = find_symbol(&syms, "DefaultTimeout").unwrap();
        assert_eq!(s.kind, SymbolKind::Variable);
    }
}
