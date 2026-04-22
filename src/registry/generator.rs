use crate::models::Registry;
use crate::utils::Result;
use chrono::Utc;
use std::collections::HashMap;

pub struct RegistryGenerator {
    version: u32,
}

impl RegistryGenerator {
    pub fn new(version: u32) -> Self {
        Self { version }
    }

    pub fn generate(&self, skills: HashMap<String, crate::models::Skill>) -> Registry {
        Registry {
            version: self.version,
            updated: Utc::now().format("%Y-%m-%d").to_string(),
            skills,
        }
    }

    pub fn to_toml(&self, registry: &Registry) -> String {
        toml::to_string_pretty(registry).unwrap_or_default()
    }

    pub fn write(&self, registry: &Registry, path: &std::path::Path) -> Result<()> {
        std::fs::write(path, self.to_toml(registry))?;
        Ok(())
    }
}
