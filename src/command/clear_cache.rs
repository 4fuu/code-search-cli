use crate::clear_cache_directory;
use anyhow::Result;

pub fn run() -> Result<()> {
    let result = clear_cache_directory(std::env::current_dir()?)?;
    if result.removed {
        println!("Removed {}", result.cache_dir.display());
    } else {
        println!("No cache directory found");
    }
    Ok(())
}
