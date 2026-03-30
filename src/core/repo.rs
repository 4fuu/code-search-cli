use crate::core::error::AppError;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Walk up from `start` to find the nearest `.git` directory, returning the repo root.
pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let mut current = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir()?.join(start)
    };
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            return Err(AppError::NotInGitRepo.into());
        }
    }
}

/// Return the `.code-search/` directory path for the given repo root.
pub fn cache_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".code-search")
}
