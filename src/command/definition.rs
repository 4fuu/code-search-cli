use crate::cli::DefinitionArgs;
use crate::core::cache::{refresh_cache, SymbolLoad};
use crate::core::discover::has_ignored_path_match;
use crate::core::output::print_symbols;
use crate::core::repo::find_repo_root;
use crate::core::symbol::SymbolKind;
use crate::lang::language_supports_kind;
use anyhow::{Context, Result};

pub fn run(args: DefinitionArgs) -> Result<()> {
    let repo_root = find_repo_root(&std::env::current_dir()?)?;
    let mut results = refresh_cache(&repo_root, SymbolLoad::All)?
        .symbol_index
        .context("missing repository symbol index")?
        .exact_name_matches(&args.name);

    // Filter by kind
    if let Some(kind) = args.kind {
        results.retain(|s| s.kind == kind);
    }

    // Filter by language
    if let Some(lang) = args.lang {
        results.retain(|s| s.language == lang);
    }

    // Filter by path
    if let Some(ref path_pattern) = args.path {
        results.retain(|s| s.path.contains(path_pattern.as_str()));
    }

    if results.is_empty() {
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

    results.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.selection_range.line.cmp(&b.selection_range.line))
            .then(a.selection_range.column.cmp(&b.selection_range.column))
            .then(kind_priority(a.kind).cmp(&kind_priority(b.kind)))
    });

    let total = results.len();
    let end = args.offset.saturating_add(args.limit).min(total);
    let results = if args.offset >= total {
        Vec::new()
    } else {
        results
            .into_iter()
            .skip(args.offset)
            .take(end - args.offset)
            .collect()
    };

    print_symbols(
        &results,
        "definition",
        &args.format,
        Some(total),
        args.offset,
        args.limit,
    )
}

fn kind_priority(kind: SymbolKind) -> u8 {
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
