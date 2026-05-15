use crate::index_repository;
use anyhow::Result;
use std::time::Instant;

pub fn run() -> Result<()> {
    let t = Instant::now();
    let result = index_repository(std::env::current_dir()?)?;
    let elapsed = t.elapsed().as_secs_f64();
    println!(
        "Indexed {} symbols across {} files in {:.2}s  ({} cached, {} updated)",
        result.total_symbols,
        result.cached_files + result.updated_files,
        elapsed,
        result.cached_files,
        result.updated_files,
    );
    Ok(())
}
