use thiserror::Error;

#[derive(Error, Debug)]
pub enum RulesifyError {
    #[error("Registry fetch failed: {0}")]
    RegistryFetch(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("No skills match the current filters")]
    NoMatchingSkills,

    #[error("Project scan failed: {0}")]
    ScanFailed(String),

    #[error("Config file error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),
}

pub type Result<T> = anyhow::Result<T>;
