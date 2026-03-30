use crate::core::cache::{refresh_cache, SymbolLoad};
use crate::core::repo::find_repo_root;
use anyhow::Result;
use std::time::Instant;

pub fn run() -> Result<()> {
    let repo_root = find_repo_root(&std::env::current_dir()?)?;
    let t = Instant::now();
    let result = refresh_cache(&repo_root, SymbolLoad::None)?;
    let elapsed = t.elapsed().as_secs_f64();
    println!(
        "Indexed {} symbols across {} files in {:.2}s  ({} cached, {} updated)",
        result.total_symbols,
        result.stats.cached + result.stats.updated,
        elapsed,
        result.stats.cached,
        result.stats.updated,
    );
    Ok(())
}
