use crate::cli::SymbolsArgs;
use crate::core::cache::{refresh_cache, SymbolLoad};
use crate::core::discover::{has_ignored_path_match, path_matches};
use crate::core::output::print_symbols;
use crate::core::repo::find_repo_root;
use crate::core::symbol::SymbolKind;
use crate::lang::language_supports_kind;
use anyhow::{Context, Result};

pub fn run(args: SymbolsArgs) -> Result<()> {
    let repo_root = find_repo_root(&std::env::current_dir()?)?;
    let symbol_index = refresh_cache(&repo_root, SymbolLoad::All)?
        .symbol_index
        .context("missing repository symbol index")?;
    let mut symbols = if let Some(ref name) = args.name {
        symbol_index.substring_name_matches(name)
    } else {
        symbol_index.into_symbols()
    };

    // Filter by name (case-insensitive substring)
    if let Some(ref name) = args.name {
        let lower = name.to_lowercase();
        symbols.retain(|s| s.name.to_lowercase().contains(&lower));
    }

    // Filter by kind
    if let Some(kind) = args.kind {
        symbols.retain(|s| s.kind == kind);
    }

    // Filter by language
    if let Some(lang) = args.lang {
        symbols.retain(|s| s.language == lang);
    }

    // Filter by path glob
    if let Some(ref path_pattern) = args.path {
        symbols.retain(|s| path_matches(&s.path, path_pattern));
    }

    // Sort: exact match > prefix > kind priority > path
    if let Some(ref name) = args.name {
        let lower = name.to_lowercase();
        symbols.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase() == lower;
            let b_exact = b.name.to_lowercase() == lower;
            if a_exact != b_exact {
                return b_exact.cmp(&a_exact);
            }

            let a_prefix = a.name.to_lowercase().starts_with(&lower);
            let b_prefix = b.name.to_lowercase().starts_with(&lower);
            if a_prefix != b_prefix {
                return b_prefix.cmp(&a_prefix);
            }

            let a_prio = kind_priority(&a.kind);
            let b_prio = kind_priority(&b.kind);
            if a_prio != b_prio {
                return a_prio.cmp(&b_prio);
            }

            a.path.cmp(&b.path)
        });
    } else {
        symbols.sort_by(|a, b| {
            let a_prio = kind_priority(&a.kind);
            let b_prio = kind_priority(&b.kind);
            if a_prio != b_prio {
                return a_prio.cmp(&b_prio);
            }
            a.path.cmp(&b.path).then(a.name.cmp(&b.name))
        });
    }

    if symbols.is_empty() {
        if let (Some(kind), Some(lang)) = (args.kind, args.lang) {
            if !language_supports_kind(lang, kind) {
                eprintln!("warning: {} does not produce '{}' symbols", lang, kind);
            }
        }
        if let Some(ref pattern) = args.path {
            if has_ignored_path_match(&repo_root, pattern) {
                eprintln!(
                    "warning: '--path {}' matches files that are excluded by ignore rules",
                    pattern
                );
            }
        }
    }

    let total = symbols.len();
    let end = args.offset.saturating_add(args.limit).min(total);
    let symbols = if args.offset >= total {
        Vec::new()
    } else {
        symbols
            .into_iter()
            .skip(args.offset)
            .take(end - args.offset)
            .collect()
    };

    print_symbols(
        &symbols,
        "symbols",
        &args.format,
        Some(total),
        args.offset,
        args.limit,
    )
}

fn kind_priority(kind: &SymbolKind) -> u8 {
    match kind {
        SymbolKind::Class | SymbolKind::Struct | SymbolKind::Interface | SymbolKind::Trait => 0,
        SymbolKind::Enum => 1,
        SymbolKind::Function | SymbolKind::Method => 2,
        SymbolKind::TypeAlias => 3,
        SymbolKind::Const | SymbolKind::Static | SymbolKind::Variable => 4,
        SymbolKind::Impl => 5,
        SymbolKind::Module => 6,
    }
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
