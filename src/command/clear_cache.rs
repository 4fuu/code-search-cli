use crate::core::cache::with_cache_lock;
use crate::core::repo::{cache_dir, find_repo_root};
use anyhow::Result;
use std::fs;

pub fn run() -> Result<()> {
    let repo_root = find_repo_root(&std::env::current_dir()?)?;
    with_cache_lock(&repo_root, || {
        let dir = cache_dir(&repo_root);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
            println!("Removed {}", dir.display());
        } else {
            println!("No cache directory found");
        }
        Ok(())
    })
}
