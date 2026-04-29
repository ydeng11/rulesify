use crate::models::Registry;
use crate::utils::{Result, RulesifyError};

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/ydeng11/rulesify/main/registry.toml";

pub async fn fetch_registry() -> Result<Registry> {
    let client = reqwest::Client::new();
    let response = client
        .get(REGISTRY_URL)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| RulesifyError::RegistryFetch(e.to_string()))?;

    if !response.status().is_success() {
        return Err(RulesifyError::RegistryFetch(format!("HTTP {}", response.status())).into());
    }

    let content = response.text().await?;
    let registry: Registry = toml::from_str(&content)?;
    Ok(registry)
}
