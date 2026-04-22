use crate::models::Domain;
use std::str::FromStr;

#[test]
fn test_domain_as_str() {
    assert_eq!(
        Domain::PlanningAndOrchestration.as_str(),
        "planning-and-orchestration"
    );
    assert_eq!(Domain::Development.as_str(), "development");
    assert_eq!(Domain::Design.as_str(), "design");
    assert_eq!(Domain::Documentation.as_str(), "documentation");
    assert_eq!(Domain::Data.as_str(), "data");
    assert_eq!(Domain::Testing.as_str(), "testing");
    assert_eq!(Domain::Deployment.as_str(), "deployment");
    assert_eq!(Domain::Integrations.as_str(), "integrations");
    assert_eq!(Domain::Collaboration.as_str(), "collaboration");
    assert_eq!(Domain::Security.as_str(), "security");
}

#[test]
fn test_domain_display() {
    assert_eq!(
        format!("{}", Domain::PlanningAndOrchestration),
        "planning-and-orchestration"
    );
    assert_eq!(format!("{}", Domain::Development), "development");
}

#[test]
fn test_domain_from_str_valid() {
    assert_eq!(
        Domain::from_str("planning-and-orchestration").unwrap(),
        Domain::PlanningAndOrchestration
    );
    assert_eq!(
        Domain::from_str("development").unwrap(),
        Domain::Development
    );
    assert_eq!(Domain::from_str("design").unwrap(), Domain::Design);
}

#[test]
fn test_domain_from_str_case_insensitive() {
    assert_eq!(
        Domain::from_str("DEVELOPMENT").unwrap(),
        Domain::Development
    );
    assert_eq!(
        Domain::from_str("  development  ").unwrap(),
        Domain::Development
    );
}

#[test]
fn test_domain_from_str_invalid() {
    assert!(Domain::from_str("invalid-domain").is_err());
    assert!(Domain::from_str("unknown").is_err());
}

#[test]
fn test_domain_all() {
    let all = Domain::all();
    assert_eq!(all.len(), 10);
}

#[test]
fn test_domain_default_fallback() {
    assert_eq!(Domain::default_fallback(), Domain::Development);
}

#[test]
fn test_domain_default() {
    assert_eq!(Domain::default(), Domain::Development);
}

#[test]
fn test_domain_serialization() {
    let domain = Domain::Deployment;
    let json = serde_json::to_string(&domain).unwrap();
    assert_eq!(json, "\"deployment\"");
}

#[test]
fn test_domain_deserialization() {
    let domain: Domain = serde_json::from_str("\"deployment\"").unwrap();
    assert_eq!(domain, Domain::Deployment);
}
