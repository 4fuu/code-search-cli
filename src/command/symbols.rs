use crate::core::output::print_symbols;
use crate::{search_symbols, SymbolSearchRequest, SymbolsArgs};
use anyhow::Result;

pub fn run(args: SymbolsArgs) -> Result<()> {
    let result = search_symbols(SymbolSearchRequest {
        repo_path: std::env::current_dir()?,
        name: args.name.clone(),
        kind: args.kind,
        language: args.lang,
        path_pattern: args.path.clone(),
        limit: args.limit,
        offset: args.offset,
    })?;
    for warning in &result.warnings {
        eprintln!("warning: {warning}");
    }

    print_symbols(
        &result.items,
        "symbols",
        &args.format,
        Some(result.total),
        args.offset,
        args.limit,
    )
}

#[cfg(test)]
mod tests {
    use crate::core::discover::path_matches;

    #[test]
    fn path_matches_plain() {
        assert!(path_matches("src/core/parser.rs", "core"));
        assert!(path_matches("src/core/parser.rs", "parser.rs"));
        assert!(!path_matches("src/core/parser.rs", "lang"));
    }

    #[test]
    fn path_matches_glob_star() {
        assert!(path_matches("src/core/parser.rs", "src/*/parser.rs"));
        assert!(path_matches("src/core/parser.rs", "*.rs"));
        assert!(!path_matches("src/core/parser.rs", "*.go"));
    }

    #[test]
    fn path_matches_glob_prefix() {
        assert!(path_matches("src/core/parser.rs", "src/*"));
        assert!(!path_matches("tests/foo.rs", "src/*"));
    }
}
