use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    PlanningAndWorkflows,
    Development,
    DesignAndMedia,
    Documentation,
    DataAndResearch,
    TestingAndDebugging,
    DeploymentAndInfrastructure,
    IntegrationsAndTools,
    CollaborationAndCommunication,
    SecurityAndPrivacy,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::PlanningAndWorkflows => "planning-and-workflows",
            Domain::Development => "development",
            Domain::DesignAndMedia => "design-and-media",
            Domain::Documentation => "documentation",
            Domain::DataAndResearch => "data-and-research",
            Domain::TestingAndDebugging => "testing-and-debugging",
            Domain::DeploymentAndInfrastructure => "deployment-and-infrastructure",
            Domain::IntegrationsAndTools => "integrations-and-tools",
            Domain::CollaborationAndCommunication => "collaboration-and-communication",
            Domain::SecurityAndPrivacy => "security-and-privacy",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Domain::PlanningAndWorkflows,
            Domain::Development,
            Domain::DesignAndMedia,
            Domain::Documentation,
            Domain::DataAndResearch,
            Domain::TestingAndDebugging,
            Domain::DeploymentAndInfrastructure,
            Domain::IntegrationsAndTools,
            Domain::CollaborationAndCommunication,
            Domain::SecurityAndPrivacy,
        ]
    }

    pub fn default_fallback() -> Self {
        Domain::Development
    }

    pub fn domain_list_string() -> String {
        Domain::all()
            .iter()
            .map(|d| format!("- {}", d.as_str()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Domain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "planning-and-workflows" => Ok(Domain::PlanningAndWorkflows),
            "development" => Ok(Domain::Development),
            "design-and-media" => Ok(Domain::DesignAndMedia),
            "documentation" => Ok(Domain::Documentation),
            "data-and-research" => Ok(Domain::DataAndResearch),
            "testing-and-debugging" => Ok(Domain::TestingAndDebugging),
            "deployment-and-infrastructure" => Ok(Domain::DeploymentAndInfrastructure),
            "integrations-and-tools" => Ok(Domain::IntegrationsAndTools),
            "collaboration-and-communication" => Ok(Domain::CollaborationAndCommunication),
            "security-and-privacy" => Ok(Domain::SecurityAndPrivacy),
            _ => Err(format!("Invalid domain: {}", s)),
        }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Domain::default_fallback()
    }
}
