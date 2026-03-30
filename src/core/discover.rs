use crate::core::language::Language;
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// A discovered source file with its detected language.
pub struct SourceFile {
    /// Path relative to repo root.
    pub rel_path: PathBuf,
    pub language: Language,
}

const EXTRA_IGNORES: &[&str] = &["node_modules", "dist", "build", "target", ".venv", "vendor"];

fn is_extra_ignored_path(path: &Path) -> bool {
    path.components().any(|comp| {
        let s = comp.as_os_str().to_string_lossy();
        EXTRA_IGNORES.iter().any(|ig| s == *ig)
    })
}

/// Scan the repository for source files, respecting `.gitignore` and extra ignores.
pub fn discover_files(repo_root: &Path) -> Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    let mut builder = WalkBuilder::new(repo_root);
    builder.hidden(true).git_ignore(true).git_global(true);
    let root_for_filter = repo_root.to_path_buf();
    builder.filter_entry(move |entry| {
        let rel = entry
            .path()
            .strip_prefix(&root_for_filter)
            .unwrap_or(entry.path());
        !is_extra_ignored_path(rel)
    });

    for entry in builder.build() {
        let entry = entry?;
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(repo_root).unwrap_or(path);

        if let Some(lang) = Language::from_path(path) {
            files.push(SourceFile {
                rel_path: rel.to_path_buf(),
                language: lang,
            });
        }
    }

    Ok(files)
}

/// Substring / simple-glob path filter. `*` matches any sequence of characters.
pub fn path_matches(path: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        let mut remaining = path;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if let Some(pos) = remaining.find(part) {
                if i == 0 && pos != 0 {
                    return false;
                }
                remaining = &remaining[pos + part.len()..];
            } else {
                return false;
            }
        }
        true
    } else {
        path.contains(pattern)
    }
}

/// Return true if `pattern` matches at least one supported-language file that
/// exists on disk but is excluded by ignore rules (gitignore, etc.).
/// Only called when a `--path` filter already produced 0 results, so the
/// extra walk is acceptable.
pub fn has_ignored_path_match(repo_root: &Path, pattern: &str) -> bool {
    let mut builder = WalkBuilder::new(repo_root);
    builder
        .hidden(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false);

    for entry in builder.build().flatten() {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(repo_root).unwrap_or(path);

        // Still skip the hard-coded extra-ignore dirs
        if is_extra_ignored_path(rel) {
            continue;
        }

        if Language::from_path(path).is_none() {
            continue;
        }

        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if path_matches(&rel_str, pattern) {
            return true;
        }
    }
    false
}
