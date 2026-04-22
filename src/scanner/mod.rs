pub mod framework;
pub mod language;
pub mod tool_config;

#[cfg(test)]
mod language_tests;

use crate::models::ProjectContext;
use crate::utils::Result;

pub fn scan_project(path: &std::path::Path) -> Result<ProjectContext> {
    let languages = language::detect(path)?;
    let frameworks = framework::detect(path)?;
    let existing_tools = tool_config::detect(path)?;

    Ok(ProjectContext {
        languages,
        frameworks,
        existing_tools,
    })
}
