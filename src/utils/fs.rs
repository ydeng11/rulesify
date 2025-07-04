use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

pub fn copy_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        ensure_dir_exists(parent)?;
    }
    
    fs::copy(from, to)
        .with_context(|| format!("Failed to copy {} to {}", from.display(), to.display()))?;
    
    Ok(())
} 