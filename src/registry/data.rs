use crate::models::Registry;
use crate::utils::Result;

pub fn load_builtin() -> Result<Registry> {
    let content = include_str!("../../registry.toml");
    let registry: Registry = toml::from_str(content)?;
    Ok(registry)
}