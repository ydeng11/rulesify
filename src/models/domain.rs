use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    PlanningAndOrchestration,
    Development,
    Design,
    Documentation,
    Data,
    Testing,
    Deployment,
    Integrations,
    Collaboration,
    Security,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::PlanningAndOrchestration => "planning-and-orchestration",
            Domain::Development => "development",
            Domain::Design => "design",
            Domain::Documentation => "documentation",
            Domain::Data => "data",
            Domain::Testing => "testing",
            Domain::Deployment => "deployment",
            Domain::Integrations => "integrations",
            Domain::Collaboration => "collaboration",
            Domain::Security => "security",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Domain::PlanningAndOrchestration,
            Domain::Development,
            Domain::Design,
            Domain::Documentation,
            Domain::Data,
            Domain::Testing,
            Domain::Deployment,
            Domain::Integrations,
            Domain::Collaboration,
            Domain::Security,
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
            "planning-and-orchestration" => Ok(Domain::PlanningAndOrchestration),
            "development" => Ok(Domain::Development),
            "design" => Ok(Domain::Design),
            "documentation" => Ok(Domain::Documentation),
            "data" => Ok(Domain::Data),
            "testing" => Ok(Domain::Testing),
            "deployment" => Ok(Domain::Deployment),
            "integrations" => Ok(Domain::Integrations),
            "collaboration" => Ok(Domain::Collaboration),
            "security" => Ok(Domain::Security),
            _ => Err(format!("Invalid domain: {}", s)),
        }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Domain::default_fallback()
    }
}
