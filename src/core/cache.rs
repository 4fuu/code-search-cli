use crate::core::symbol::Symbol;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub files: HashMap<String, FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub cache_key: String,
    pub size: u64,
    pub mtime_ms: u128,
    #[serde(default)]
    pub symbol_count: Option<usize>,
}

pub struct Fingerprint {
    pub size: u64,
    pub mtime_ms: u128,
}

impl Fingerprint {
    pub fn from_path(path: &Path) -> Result<Self> {
        let meta = fs::metadata(path)?;
        let mtime = meta
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Ok(Fingerprint {
            size: meta.len(),
            mtime_ms: mtime.as_millis(),
        })
    }

    pub fn matches(&self, entry: &FileEntry) -> bool {
        self.size == entry.size && self.mtime_ms == entry.mtime_ms
    }
}

/// Initialize the `.code-search/` directory and `.gitignore` if needed.
pub fn ensure_cache_dir(repo_root: &Path) -> Result<()> {
    let cache = repo_root.join(".code-search");
    fs::create_dir_all(cache.join("files"))?;
    let gitignore = cache.join(".gitignore");
    if !gitignore.exists() {
        fs::write(&gitignore, "*\n")?;
    }
    Ok(())
}

pub fn manifest_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".code-search").join("manifest.json")
}

fn lock_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".code-search.lock")
}

pub fn load_manifest(repo_root: &Path) -> Result<Manifest> {
    let path = manifest_path(repo_root);
    if !path.exists() {
        return Ok(Manifest {
            files: HashMap::new(),
        });
    }
    let data = fs::read_to_string(&path)?;
    let manifest: Manifest =
        serde_json::from_str(&data).context("failed to parse manifest.json")?;
    Ok(manifest)
}

pub fn save_manifest(repo_root: &Path, manifest: &Manifest) -> Result<()> {
    ensure_cache_dir(repo_root)?;
    let path = manifest_path(repo_root);
    let data = serde_json::to_string(manifest)?;
    atomic_write(path, data.as_bytes())?;
    Ok(())
}

fn cache_key(rel_path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    rel_path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn cache_file_path(repo_root: &Path, key: &str) -> PathBuf {
    let prefix = &key[..2.min(key.len())];
    repo_root
        .join(".code-search")
        .join("files")
        .join(prefix)
        .join(format!("{}.bin", key))
}

fn repo_index_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".code-search").join("symbols.bin")
}

pub fn read_cached_symbols(repo_root: &Path, key: &str) -> Result<Vec<Symbol>> {
    let path = cache_file_path(repo_root, key);
    let data = fs::read(&path)?;
    let symbols: Vec<Symbol> = bincode::deserialize(&data)?;
    Ok(symbols)
}

pub fn write_cached_symbols(repo_root: &Path, key: &str, symbols: &[Symbol]) -> Result<()> {
    let path = cache_file_path(repo_root, key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = bincode::serialize(symbols)?;
    atomic_write(path, &data)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolIndex {
    symbols: Vec<Symbol>,
    exact_name: HashMap<String, Vec<usize>>,
    trigram_index: HashMap<String, Vec<usize>>,
}

impl SymbolIndex {
    pub fn from_symbols(symbols: Vec<Symbol>) -> Self {
        let mut exact_name: HashMap<String, Vec<usize>> = HashMap::new();
        let mut trigram_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, symbol) in symbols.iter().enumerate() {
            let lower = symbol.name.to_lowercase();
            exact_name.entry(lower.clone()).or_default().push(idx);
            for trigram in unique_trigrams(&lower) {
                trigram_index.entry(trigram).or_default().push(idx);
            }
        }
        Self {
            symbols,
            exact_name,
            trigram_index,
        }
    }

    pub fn into_symbols(self) -> Vec<Symbol> {
        self.symbols
    }

    pub fn exact_name_matches(&self, name: &str) -> Vec<Symbol> {
        let lower = name.to_lowercase();
        self.exact_name
            .get(&lower)
            .into_iter()
            .flat_map(|indexes| indexes.iter().map(|idx| self.symbols[*idx].clone()))
            .collect()
    }

    pub fn substring_name_matches(&self, name: &str) -> Vec<Symbol> {
        let lower = name.to_lowercase();
        if lower.is_empty() {
            return self.symbols.clone();
        }

        if lower.chars().count() < 3 {
            return self
                .symbols
                .iter()
                .filter(|symbol| symbol.name.to_lowercase().contains(&lower))
                .cloned()
                .collect();
        }

        let trigrams = unique_trigrams(&lower);
        let mut counts: HashMap<usize, usize> = HashMap::new();
        for trigram in &trigrams {
            let Some(indexes) = self.trigram_index.get(trigram) else {
                return Vec::new();
            };
            for idx in indexes {
                *counts.entry(*idx).or_insert(0) += 1;
            }
        }

        counts
            .into_iter()
            .filter(|(_, matched)| *matched == trigrams.len())
            .map(|(idx, _)| idx)
            .filter(|idx| self.symbols[*idx].name.to_lowercase().contains(&lower))
            .map(|idx| self.symbols[idx].clone())
            .collect()
    }
}

fn read_repo_index(repo_root: &Path) -> Result<SymbolIndex> {
    let data = fs::read(repo_index_path(repo_root))?;
    Ok(bincode::deserialize(&data)?)
}

fn write_repo_index(repo_root: &Path, index: &SymbolIndex) -> Result<()> {
    ensure_cache_dir(repo_root)?;
    let data = bincode::serialize(index)?;
    atomic_write(repo_index_path(repo_root), &data)?;
    Ok(())
}

pub struct RefreshStats {
    pub cached: usize,
    pub updated: usize,
}

struct CachedFile {
    rel_str: String,
    cache_key: String,
}

struct StaleFile {
    rel_str: String,
    key: String,
    abs_path: PathBuf,
    language: crate::core::language::Language,
}

struct StaleResult {
    rel_str: String,
    entry: FileEntry,
    symbols: Vec<Symbol>,
}

pub enum SymbolLoad {
    None,
    All,
}

pub struct RefreshResult {
    pub symbol_index: Option<SymbolIndex>,
    pub stats: RefreshStats,
    pub total_symbols: usize,
}

/// Incrementally update the cache, optionally returning the repository symbol index.
pub fn refresh_cache(repo_root: &Path, load_symbols: SymbolLoad) -> Result<RefreshResult> {
    with_cache_lock(repo_root, || refresh_cache_locked(repo_root, load_symbols))
}

pub fn with_cache_lock<T>(repo_root: &Path, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let _guard = CacheLockGuard::acquire(repo_root)?;
    f()
}

fn refresh_cache_locked(repo_root: &Path, load_symbols: SymbolLoad) -> Result<RefreshResult> {
    use crate::core::discover::discover_files;
    use crate::core::parser::parse_file;
    use rayon::prelude::*;

    ensure_cache_dir(repo_root)?;
    let mut manifest = load_manifest(repo_root)?;
    let source_files = discover_files(repo_root)?;

    let mut current_keys: HashSet<String> = HashSet::with_capacity(source_files.len());
    let mut cached_files: Vec<CachedFile> = Vec::new();
    let mut stale_files: Vec<StaleFile> = Vec::new();

    for sf in &source_files {
        let rel_str = sf.rel_path.to_string_lossy().replace('\\', "/");
        let key = cache_key(&rel_str);
        current_keys.insert(rel_str.clone());

        let abs_path = repo_root.join(&sf.rel_path);
        let mut is_cached = false;

        if let Some(entry) = manifest.files.get(&rel_str) {
            if let Ok(fp) = Fingerprint::from_path(&abs_path) {
                if fp.matches(entry) && entry.symbol_count.is_some() {
                    cached_files.push(CachedFile {
                        rel_str: rel_str.clone(),
                        cache_key: entry.cache_key.clone(),
                    });
                    is_cached = true;
                }
            }
        }

        if !is_cached {
            stale_files.push(StaleFile {
                rel_str,
                key,
                abs_path,
                language: sf.language,
            });
        }
    }

    let cached_count = cached_files.len();

    let stale_results: Vec<StaleResult> = stale_files
        .par_iter()
        .filter_map(|sf| {
            let fp = Fingerprint::from_path(&sf.abs_path).ok()?;
            let source = fs::read_to_string(&sf.abs_path).ok()?;
            let symbols =
                parse_file(std::path::Path::new(&sf.rel_str), &source, sf.language).ok()?;
            write_cached_symbols(repo_root, &sf.key, &symbols).ok()?;
            Some(StaleResult {
                rel_str: sf.rel_str.clone(),
                entry: FileEntry {
                    cache_key: sf.key.clone(),
                    size: fp.size,
                    mtime_ms: fp.mtime_ms,
                    symbol_count: Some(symbols.len()),
                },
                symbols,
            })
        })
        .collect();

    let updated_count = stale_results.len();

    // Remove entries for deleted files
    let removed: Vec<String> = manifest
        .files
        .keys()
        .filter(|k| !current_keys.contains(k.as_str()))
        .cloned()
        .collect();
    for key in &removed {
        if let Some(entry) = manifest.files.remove(key) {
            let path = cache_file_path(repo_root, &entry.cache_key);
            let _ = fs::remove_file(path);
        }
    }

    for result in &stale_results {
        manifest
            .files
            .insert(result.rel_str.clone(), result.entry.clone());
    }

    let total_symbols = manifest
        .files
        .values()
        .map(|entry| entry.symbol_count.unwrap_or(0))
        .sum();

    let repo_index_exists = repo_index_path(repo_root).exists();
    let repo_index_needs_update =
        !stale_results.is_empty() || !removed.is_empty() || !repo_index_exists;
    let symbol_index = if repo_index_needs_update {
        Some(update_repo_index(
            repo_root,
            repo_index_exists,
            &cached_files,
            &stale_results,
            &removed,
        )?)
    } else if matches!(load_symbols, SymbolLoad::All) {
        Some(read_repo_index(repo_root)?)
    } else {
        None
    };

    save_manifest(repo_root, &manifest)?;
    Ok(RefreshResult {
        symbol_index: if matches!(load_symbols, SymbolLoad::All) {
            symbol_index
        } else {
            None
        },
        stats: RefreshStats {
            cached: cached_count,
            updated: updated_count,
        },
        total_symbols,
    })
}

struct CacheLockGuard {
    path: PathBuf,
}

impl CacheLockGuard {
    fn acquire(repo_root: &Path) -> Result<Self> {
        let path = lock_path(repo_root);
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return Ok(Self { path }),
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                    if lock_is_stale(&path)? {
                        let _ = fs::remove_file(&path);
                        continue;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(err) => return Err(err.into()),
            }
        }
    }
}

impl Drop for CacheLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn lock_is_stale(path: &Path) -> Result<bool> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err.into()),
    };
    let modified = match meta.modified() {
        Ok(modified) => modified,
        Err(_) => return Ok(false),
    };
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    Ok(age > Duration::from_secs(60 * 60))
}

fn atomic_write(path: PathBuf, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("atomic write target must have a parent directory")?;
    fs::create_dir_all(parent)?;
    let unique = format!(
        ".tmp-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let tmp_path = parent.join(unique);
    fs::write(&tmp_path, bytes)?;
    match fs::rename(&tmp_path, &path) {
        Ok(()) => {}
        Err(err) => {
            if path.exists() {
                let _ = fs::remove_file(&path);
                fs::rename(&tmp_path, &path).map_err(|_| err)?;
            } else {
                let _ = fs::remove_file(&tmp_path);
                return Err(err.into());
            }
        }
    }
    Ok(())
}

fn unique_trigrams(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 3 {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for window in chars.windows(3) {
        let trigram: String = window.iter().collect();
        if seen.insert(trigram.clone()) {
            out.push(trigram);
        }
    }
    out
}

fn update_repo_index(
    repo_root: &Path,
    repo_index_exists: bool,
    cached_files: &[CachedFile],
    stale_results: &[StaleResult],
    removed: &[String],
) -> Result<SymbolIndex> {
    use rayon::prelude::*;

    let dirty_paths: HashSet<&str> = stale_results
        .iter()
        .map(|result| result.rel_str.as_str())
        .chain(removed.iter().map(String::as_str))
        .collect();

    let mut all_symbols: Vec<Symbol> = if repo_index_exists {
        match read_repo_index(repo_root) {
            Ok(index) => index
                .into_symbols()
                .into_iter()
                .filter(|symbol| !dirty_paths.contains(symbol.path.as_str()))
                .collect(),
            Err(_) => cached_files
                .par_iter()
                .filter_map(|cf| {
                    let _ = &cf.rel_str;
                    read_cached_symbols(repo_root, &cf.cache_key).ok()
                })
                .flatten()
                .collect(),
        }
    } else {
        cached_files
            .par_iter()
            .filter_map(|cf| {
                let _ = &cf.rel_str;
                read_cached_symbols(repo_root, &cf.cache_key).ok()
            })
            .flatten()
            .collect()
    };

    all_symbols.extend(
        stale_results
            .iter()
            .flat_map(|result| result.symbols.iter().cloned()),
    );

    let index = SymbolIndex::from_symbols(all_symbols);
    write_repo_index(repo_root, &index)?;
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_serde_roundtrip() {
        let mut manifest = Manifest {
            files: HashMap::new(),
        };
        manifest.files.insert(
            "src/main.rs".to_string(),
            FileEntry {
                cache_key: "abc123".to_string(),
                size: 1024,
                mtime_ms: 1700000000000,
                symbol_count: Some(3),
            },
        );
        let json = serde_json::to_string(&manifest).unwrap();
        let deser: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.files.len(), 1);
        let entry = deser.files.get("src/main.rs").unwrap();
        assert_eq!(entry.cache_key, "abc123");
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.symbol_count, Some(3));
    }

    #[test]
    fn fingerprint_matches() {
        let fp = Fingerprint {
            size: 100,
            mtime_ms: 5000,
        };
        let entry = FileEntry {
            cache_key: "x".to_string(),
            size: 100,
            mtime_ms: 5000,
            symbol_count: Some(0),
        };
        assert!(fp.matches(&entry));
    }

    #[test]
    fn fingerprint_mismatch_size() {
        let fp = Fingerprint {
            size: 200,
            mtime_ms: 5000,
        };
        let entry = FileEntry {
            cache_key: "x".to_string(),
            size: 100,
            mtime_ms: 5000,
            symbol_count: Some(0),
        };
        assert!(!fp.matches(&entry));
    }

    #[test]
    fn fingerprint_mismatch_mtime() {
        let fp = Fingerprint {
            size: 100,
            mtime_ms: 6000,
        };
        let entry = FileEntry {
            cache_key: "x".to_string(),
            size: 100,
            mtime_ms: 5000,
            symbol_count: Some(0),
        };
        assert!(!fp.matches(&entry));
    }

    #[test]
    fn cache_key_deterministic() {
        let k1 = cache_key("src/main.rs");
        let k2 = cache_key("src/main.rs");
        assert_eq!(k1, k2);
        let k3 = cache_key("src/lib.rs");
        assert_ne!(k1, k3);
    }

    #[test]
    fn cache_file_path_structure() {
        let repo = Path::new("/repo");
        let key = "abcdef0123456789";
        let path = cache_file_path(repo, key);
        assert!(path.to_string_lossy().contains(".code-search/files/ab/"));
        assert!(path.to_string_lossy().ends_with(".bin"));
    }

    #[test]
    fn symbol_index_exact_name_lookup() {
        let index = SymbolIndex::from_symbols(vec![
            Symbol {
                name: "Parser".to_string(),
                kind: crate::core::symbol::SymbolKind::Struct,
                language: crate::core::language::Language::Rust,
                path: "src/lib.rs".to_string(),
                container_name: None,
                signature: None,
                range: crate::core::symbol::TextRange {
                    line: 1,
                    column: 0,
                    end_line: 1,
                    end_column: 6,
                },
                selection_range: crate::core::symbol::TextRange {
                    line: 1,
                    column: 0,
                    end_line: 1,
                    end_column: 6,
                },
                exported: true,
            },
            Symbol {
                name: "parser".to_string(),
                kind: crate::core::symbol::SymbolKind::Function,
                language: crate::core::language::Language::Rust,
                path: "src/main.rs".to_string(),
                container_name: None,
                signature: None,
                range: crate::core::symbol::TextRange {
                    line: 2,
                    column: 0,
                    end_line: 2,
                    end_column: 6,
                },
                selection_range: crate::core::symbol::TextRange {
                    line: 2,
                    column: 0,
                    end_line: 2,
                    end_column: 6,
                },
                exported: true,
            },
        ]);

        let matches = index.exact_name_matches("PARSER");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn symbol_index_substring_lookup_uses_trigrams() {
        let index = SymbolIndex::from_symbols(vec![
            Symbol {
                name: "createLogger".to_string(),
                kind: crate::core::symbol::SymbolKind::Function,
                language: crate::core::language::Language::TypeScript,
                path: "src/logger.ts".to_string(),
                container_name: None,
                signature: None,
                range: crate::core::symbol::TextRange {
                    line: 1,
                    column: 0,
                    end_line: 1,
                    end_column: 12,
                },
                selection_range: crate::core::symbol::TextRange {
                    line: 1,
                    column: 0,
                    end_line: 1,
                    end_column: 12,
                },
                exported: true,
            },
            Symbol {
                name: "parse".to_string(),
                kind: crate::core::symbol::SymbolKind::Function,
                language: crate::core::language::Language::Rust,
                path: "src/lib.rs".to_string(),
                container_name: None,
                signature: None,
                range: crate::core::symbol::TextRange {
                    line: 2,
                    column: 0,
                    end_line: 2,
                    end_column: 5,
                },
                selection_range: crate::core::symbol::TextRange {
                    line: 2,
                    column: 0,
                    end_line: 2,
                    end_column: 5,
                },
                exported: true,
            },
        ]);

        let matches = index.substring_name_matches("logger");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "createLogger");
    }

    #[test]
    fn unique_trigrams_deduplicates() {
        assert_eq!(unique_trigrams("ab"), Vec::<String>::new());
        assert_eq!(unique_trigrams("aaaa"), vec!["aaa".to_string()]);
    }
}
