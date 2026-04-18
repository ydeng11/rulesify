use crate::models::Registry;
use crate::utils::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RegistryCache {
    cache_path: PathBuf,
}

impl RegistryCache {
    pub fn new(project_path: &Path) -> Self {
        Self {
            cache_path: project_path.join(".rulesify").join("registry.toml"),
        }
    }

    pub fn load(&self) -> Result<Option<Registry>> {
        if !self.cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.cache_path)?;
        let registry: Registry = toml::from_str(&content)?;
        Ok(Some(registry))
    }

    pub fn save(&self, registry: &Registry) -> Result<()> {
        let parent = self.cache_path.parent().unwrap();
        fs::create_dir_all(parent)?;

        let content = toml::to_string_pretty(registry)?;
        fs::write(&self.cache_path, content)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path)?;
        }
        Ok(())
    }
}
