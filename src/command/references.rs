use crate::cli::ReferencesArgs;
use crate::core::cache::{refresh_cache, SymbolLoad};
use crate::core::discover::{discover_files, has_ignored_path_match};
use crate::core::output::print_references;
use crate::core::query::{self, Reference};
use crate::core::repo::find_repo_root;
use crate::core::symbol::{Symbol, SymbolKind};
use anyhow::{Context, Result};
use std::fs;

/// Pick the most specific kind from the definition set so we can use a
/// narrower reference query.  Priority: concrete types > callables > rest.
fn primary_kind(definitions: &[&Symbol]) -> Option<SymbolKind> {
    use SymbolKind::*;
    const PRIORITY: &[SymbolKind] = &[
        Struct, Enum, Trait, Interface, Class, TypeAlias, Function, Method, Const, Static,
        Variable, Module, Impl,
    ];
    PRIORITY
        .iter()
        .copied()
        .find(|k| definitions.iter().any(|d| d.kind == *k))
}

pub fn run(args: ReferencesArgs) -> Result<()> {
    let repo_root = find_repo_root(&std::env::current_dir()?)?;

    // First, find definitions to understand what we're looking for
    let symbol_index = refresh_cache(&repo_root, SymbolLoad::All)?
        .symbol_index
        .context("missing repository symbol index")?;
    let definitions_owned = symbol_index.exact_name_matches(&args.name);
    let definitions: Vec<&Symbol> = definitions_owned.iter().collect();

    // Filter definitions by kind if specified
    let definitions: Vec<&Symbol> = if let Some(kind) = args.kind {
        definitions.into_iter().filter(|s| s.kind == kind).collect()
    } else {
        definitions
    };

    // Discover files to search
    let source_files = discover_files(&repo_root)?;

    // Filter by language
    let source_files: Vec<_> = if let Some(lang) = args.lang {
        source_files
            .into_iter()
            .filter(|sf| sf.language == lang)
            .collect()
    } else {
        source_files
    };

    // Filter by path
    let source_files: Vec<_> = if let Some(ref path_pattern) = args.path {
        use crate::core::discover::path_matches;
        let filtered: Vec<_> = source_files
            .into_iter()
            .filter(|sf| {
                let rel = sf.rel_path.to_string_lossy().replace('\\', "/");
                path_matches(&rel, path_pattern)
            })
            .collect();
        if filtered.is_empty() && has_ignored_path_match(&repo_root, path_pattern) {
            eprintln!(
                "warning: '--path {}' matches files that are excluded by ignore rules",
                path_pattern
            );
        }
        filtered
    } else {
        source_files
    };

    let warning: Option<String> = if definitions.is_empty() {
        Some(format!(
            "no definition found for '{}' in cache; results may be incomplete (using broad query)",
            args.name
        ))
    } else {
        None
    };

    let kind = primary_kind(&definitions);
    let mut all_refs: Vec<Reference> = Vec::new();

    for sf in &source_files {
        let abs_path = repo_root.join(&sf.rel_path);
        let source = match fs::read_to_string(&abs_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let rel_str = sf.rel_path.to_string_lossy().replace('\\', "/");

        let refs = query::find_references(&source, sf.language, &args.name, &rel_str, kind)?;

        // Filter out definition locations (unless --include-def)
        if !args.include_def {
            let filtered: Vec<Reference> = refs
                .into_iter()
                .filter(|r| {
                    !definitions.iter().any(|d| {
                        d.path == r.path
                            && d.selection_range.line == r.line
                            && d.selection_range.column == r.column
                    })
                })
                .collect();
            all_refs.extend(filtered);
        } else {
            all_refs.extend(refs);
        }
    }

    // Sort by path, then line
    all_refs.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));

    // Deduplicate
    all_refs.dedup_by(|a, b| a.path == b.path && a.line == b.line && a.column == b.column);

    let total = all_refs.len();
    let end = args.offset.saturating_add(args.limit).min(total);
    let all_refs = if args.offset >= total {
        Vec::new()
    } else {
        all_refs
            .into_iter()
            .skip(args.offset)
            .take(end - args.offset)
            .collect()
    };

    print_references(
        &all_refs,
        &args.format,
        warning.as_deref(),
        Some(total),
        args.offset,
        args.limit,
    )
}
