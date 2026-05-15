mod cli;
mod command;
mod core;
mod lang;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub use cli::{
    Cli, Command, DefinitionArgs, OutputFormat, OverviewArgs, ReferencesArgs, SkillCommand,
    SkillInstallArgs, SkillTarget, SymbolsArgs, DEFAULT_LIMIT,
};
pub use core::error::AppError;
pub use core::language::Language;
pub use core::query::Reference;
pub use core::symbol::{Symbol, SymbolKind, TextRange};

use core::cache::{refresh_cache, with_cache_lock, SymbolLoad};
use core::discover::{discover_files, has_ignored_path_match, path_matches};
use core::parser::parse_file;
use core::repo::{cache_dir, find_repo_root};
use core::symbol::SymbolKind::{
    Class, Const, Enum, Function, Impl, Interface, Method, Module, Static, Struct, Trait,
    TypeAlias, Variable,
};

#[derive(Debug, Clone)]
pub struct OverviewRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SymbolSearchRequest {
    pub repo_path: PathBuf,
    pub name: Option<String>,
    pub kind: Option<SymbolKind>,
    pub language: Option<Language>,
    pub path_pattern: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct DefinitionSearchRequest {
    pub repo_path: PathBuf,
    pub name: String,
    pub kind: Option<SymbolKind>,
    pub language: Option<Language>,
    pub path_pattern: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct ReferenceSearchRequest {
    pub repo_path: PathBuf,
    pub name: String,
    pub kind: Option<SymbolKind>,
    pub language: Option<Language>,
    pub path_pattern: Option<String>,
    pub include_definition: bool,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexResult {
    pub total_symbols: usize,
    pub cached_files: usize,
    pub updated_files: usize,
}

#[derive(Debug, Clone)]
pub struct ClearCacheResult {
    pub cache_dir: PathBuf,
    pub removed: bool,
}

pub fn overview(request: OverviewRequest) -> Result<Vec<Symbol>> {
    let path = request.path;
    if !path.exists() {
        return Err(AppError::FileNotFound(path.display().to_string()).into());
    }
    let language = Language::from_path(&path)
        .ok_or_else(|| AppError::UnsupportedLanguage(path.display().to_string()))?;
    let source = fs::read_to_string(&path)?;
    parse_file(&path, &source, language)
}

pub fn search_symbols(request: SymbolSearchRequest) -> Result<SearchResult<Symbol>> {
    let repo_root = find_repo_root(&request.repo_path)?;
    let symbol_index = refresh_cache(&repo_root, SymbolLoad::All)?
        .symbol_index
        .context("missing repository symbol index")?;
    let mut symbols = if let Some(ref name) = request.name {
        symbol_index.substring_name_matches(name)
    } else {
        symbol_index.into_symbols()
    };

    if let Some(ref name) = request.name {
        let lower = name.to_lowercase();
        symbols.retain(|s| s.name.to_lowercase().contains(&lower));
    }

    if let Some(kind) = request.kind {
        symbols.retain(|s| s.kind == kind);
    }

    if let Some(language) = request.language {
        symbols.retain(|s| s.language == language);
    }

    if let Some(ref path_pattern) = request.path_pattern {
        symbols.retain(|s| path_matches(&s.path, path_pattern));
    }

    if let Some(ref name) = request.name {
        let lower = name.to_lowercase();
        symbols.sort_by(|a, b| {
            let a_lower = a.name.to_lowercase();
            let b_lower = b.name.to_lowercase();

            let a_exact = a_lower == lower;
            let b_exact = b_lower == lower;
            if a_exact != b_exact {
                return b_exact.cmp(&a_exact);
            }

            let a_prefix = a_lower.starts_with(&lower);
            let b_prefix = b_lower.starts_with(&lower);
            if a_prefix != b_prefix {
                return b_prefix.cmp(&a_prefix);
            }

            let a_prio = kind_priority(a.kind);
            let b_prio = kind_priority(b.kind);
            if a_prio != b_prio {
                return a_prio.cmp(&b_prio);
            }

            a.path.cmp(&b.path)
        });
    } else {
        symbols.sort_by(|a, b| {
            let a_prio = kind_priority(a.kind);
            let b_prio = kind_priority(b.kind);
            if a_prio != b_prio {
                return a_prio.cmp(&b_prio);
            }
            a.path.cmp(&b.path).then(a.name.cmp(&b.name))
        });
    }

    let warnings = empty_result_warnings(
        symbols.is_empty(),
        &repo_root,
        request.kind,
        request.language,
        request.path_pattern.as_deref(),
    );
    let total = symbols.len();

    Ok(SearchResult {
        items: paginate(symbols, request.offset, request.limit),
        total,
        warnings,
    })
}

pub fn search_definitions(request: DefinitionSearchRequest) -> Result<SearchResult<Symbol>> {
    let repo_root = find_repo_root(&request.repo_path)?;
    let mut results = refresh_cache(&repo_root, SymbolLoad::All)?
        .symbol_index
        .context("missing repository symbol index")?
        .exact_name_matches(&request.name);

    if let Some(kind) = request.kind {
        results.retain(|s| s.kind == kind);
    }

    if let Some(language) = request.language {
        results.retain(|s| s.language == language);
    }

    if let Some(ref path_pattern) = request.path_pattern {
        results.retain(|s| path_matches(&s.path, path_pattern));
    }

    let warnings = empty_result_warnings(
        results.is_empty(),
        &repo_root,
        request.kind,
        request.language,
        request.path_pattern.as_deref(),
    );

    results.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.selection_range.line.cmp(&b.selection_range.line))
            .then(a.selection_range.column.cmp(&b.selection_range.column))
            .then(kind_priority(a.kind).cmp(&kind_priority(b.kind)))
    });

    let total = results.len();
    Ok(SearchResult {
        items: paginate(results, request.offset, request.limit),
        total,
        warnings,
    })
}

pub fn search_references(request: ReferenceSearchRequest) -> Result<SearchResult<Reference>> {
    let repo_root = find_repo_root(&request.repo_path)?;

    let symbol_index = refresh_cache(&repo_root, SymbolLoad::All)?
        .symbol_index
        .context("missing repository symbol index")?;
    let definitions_owned = symbol_index.exact_name_matches(&request.name);
    let definitions: Vec<&Symbol> = definitions_owned.iter().collect();
    let definitions: Vec<&Symbol> = if let Some(kind) = request.kind {
        definitions.into_iter().filter(|s| s.kind == kind).collect()
    } else {
        definitions
    };

    let source_files = discover_files(&repo_root)?;
    let source_files: Vec<_> = if let Some(language) = request.language {
        source_files
            .into_iter()
            .filter(|sf| sf.language == language)
            .collect()
    } else {
        source_files
    };

    let mut warnings = Vec::new();
    let source_files: Vec<_> = if let Some(ref path_pattern) = request.path_pattern {
        let filtered: Vec<_> = source_files
            .into_iter()
            .filter(|sf| {
                let rel = sf.rel_path.to_string_lossy().replace('\\', "/");
                path_matches(&rel, path_pattern)
            })
            .collect();
        if filtered.is_empty() && has_ignored_path_match(&repo_root, path_pattern) {
            warnings.push(format!(
                "'--path {path_pattern}' matches files that are excluded by ignore rules"
            ));
        }
        filtered
    } else {
        source_files
    };

    if definitions.is_empty() {
        warnings.push(format!(
            "no definition found for '{}' in cache; results may be incomplete (using broad query)",
            request.name
        ));
    }

    let kind = primary_kind(&definitions);
    let mut references = Vec::new();

    for source_file in &source_files {
        let abs_path = repo_root.join(&source_file.rel_path);
        let source = match fs::read_to_string(&abs_path) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let rel_path = source_file.rel_path.to_string_lossy().replace('\\', "/");

        let refs = core::query::find_references(
            &source,
            source_file.language,
            &request.name,
            &rel_path,
            kind,
        )?;

        if request.include_definition {
            references.extend(refs);
        } else {
            references.extend(refs.into_iter().filter(|reference| {
                !definitions.iter().any(|definition| {
                    definition.path == reference.path
                        && definition.selection_range.line == reference.line
                        && definition.selection_range.column == reference.column
                })
            }));
        }
    }

    references.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    references.dedup_by(|a, b| a.path == b.path && a.line == b.line && a.column == b.column);

    let total = references.len();
    Ok(SearchResult {
        items: paginate(references, request.offset, request.limit),
        total,
        warnings,
    })
}

pub fn index_repository(repo_path: impl AsRef<Path>) -> Result<IndexResult> {
    let repo_root = find_repo_root(repo_path.as_ref())?;
    let result = refresh_cache(&repo_root, SymbolLoad::None)?;
    Ok(IndexResult {
        total_symbols: result.total_symbols,
        cached_files: result.stats.cached,
        updated_files: result.stats.updated,
    })
}

pub fn clear_cache_directory(repo_path: impl AsRef<Path>) -> Result<ClearCacheResult> {
    let repo_root = find_repo_root(repo_path.as_ref())?;
    with_cache_lock(&repo_root, || {
        let dir = cache_dir(&repo_root);
        let removed = if dir.exists() {
            fs::remove_dir_all(&dir)?;
            true
        } else {
            false
        };
        Ok(ClearCacheResult {
            cache_dir: dir,
            removed,
        })
    })
}

pub fn run_cli(cli: Cli) -> Result<()> {
    initialize_thread_pool(cli.threads)?;
    match cli.command {
        Command::Overview(args) => command::overview::run(args),
        Command::Symbols(args) => command::symbols::run(args),
        Command::Definition(args) => command::definition::run(args),
        Command::References(args) => command::references::run(args),
        Command::Index => command::index::run(),
        Command::ClearCache => command::clear_cache::run(),
        Command::Skill(sub) => command::skill::run(sub),
    }
}

pub fn command_output_format(cmd: &Command) -> OutputFormat {
    match cmd {
        Command::Overview(args) => args.format.clone(),
        Command::Symbols(args) => args.format.clone(),
        Command::Definition(args) => args.format.clone(),
        Command::References(args) => args.format.clone(),
        Command::Index | Command::ClearCache | Command::Skill(_) => OutputFormat::Text,
    }
}

pub fn print_error(err: &AppError, format: &OutputFormat) {
    core::output::print_error(err, format);
}

fn initialize_thread_pool(threads: Option<usize>) -> Result<()> {
    let threads = threads.unwrap_or_else(default_thread_count);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .or_else(|err| {
            if err
                .to_string()
                .contains("The global thread pool has already been initialized")
            {
                Ok(())
            } else {
                Err(err)
            }
        })?;
    Ok(())
}

fn default_thread_count() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    (cpus / 2).max(1)
}

fn empty_result_warnings(
    is_empty: bool,
    repo_root: &Path,
    kind: Option<SymbolKind>,
    language: Option<Language>,
    path_pattern: Option<&str>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if !is_empty {
        return warnings;
    }

    if let (Some(kind), Some(language)) = (kind, language) {
        if !lang::language_supports_kind(language, kind) {
            warnings.push(format!("{language} does not produce '{kind}' symbols"));
        }
    }

    if let Some(path_pattern) = path_pattern {
        if has_ignored_path_match(repo_root, path_pattern) {
            warnings.push(format!(
                "'--path {path_pattern}' matches files that are excluded by ignore rules"
            ));
        }
    }

    warnings
}

fn paginate<T>(items: Vec<T>, offset: usize, limit: usize) -> Vec<T> {
    let total = items.len();
    let end = offset.saturating_add(limit).min(total);
    if offset >= total {
        Vec::new()
    } else {
        items.into_iter().skip(offset).take(end - offset).collect()
    }
}

fn kind_priority(kind: SymbolKind) -> u8 {
    match kind {
        Class | Struct | Interface | Trait => 0,
        Enum => 1,
        Function | Method => 2,
        TypeAlias => 3,
        Const | Static | Variable => 4,
        Impl => 5,
        Module => 6,
    }
}

fn primary_kind(definitions: &[&Symbol]) -> Option<SymbolKind> {
    const PRIORITY: &[SymbolKind] = &[
        Struct, Enum, Trait, Interface, Class, TypeAlias, Function, Method, Const, Static,
        Variable, Module, Impl,
    ];
    PRIORITY.iter().copied().find(|kind| {
        definitions
            .iter()
            .any(|definition| definition.kind == *kind)
    })
}
