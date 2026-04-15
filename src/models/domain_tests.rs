use crate::models::Domain;
use std::str::FromStr;

#[test]
fn test_domain_as_str() {
    assert_eq!(
        Domain::PlanningAndWorkflows.as_str(),
        "planning-and-workflows"
    );
    assert_eq!(Domain::Development.as_str(), "development");
    assert_eq!(Domain::DesignAndMedia.as_str(), "design-and-media");
    assert_eq!(Domain::Documentation.as_str(), "documentation");
    assert_eq!(Domain::DataAndResearch.as_str(), "data-and-research");
    assert_eq!(
        Domain::TestingAndDebugging.as_str(),
        "testing-and-debugging"
    );
    assert_eq!(
        Domain::DeploymentAndInfrastructure.as_str(),
        "deployment-and-infrastructure"
    );
    assert_eq!(
        Domain::IntegrationsAndTools.as_str(),
        "integrations-and-tools"
    );
    assert_eq!(
        Domain::CollaborationAndCommunication.as_str(),
        "collaboration-and-communication"
    );
    assert_eq!(Domain::SecurityAndPrivacy.as_str(), "security-and-privacy");
}

#[test]
fn test_domain_display() {
    assert_eq!(
        format!("{}", Domain::PlanningAndWorkflows),
        "planning-and-workflows"
    );
    assert_eq!(format!("{}", Domain::Development), "development");
}

#[test]
fn test_domain_from_str_valid() {
    assert_eq!(
        Domain::from_str("planning-and-workflows").unwrap(),
        Domain::PlanningAndWorkflows
    );
    assert_eq!(
        Domain::from_str("development").unwrap(),
        Domain::Development
    );
    assert_eq!(
        Domain::from_str("design-and-media").unwrap(),
        Domain::DesignAndMedia
    );
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
    let domain = Domain::DeploymentAndInfrastructure;
    let json = serde_json::to_string(&domain).unwrap();
    assert_eq!(json, "\"deployment-and-infrastructure\"");
}

#[test]
fn test_domain_deserialization() {
    let domain: Domain = serde_json::from_str("\"deployment-and-infrastructure\"").unwrap();
    assert_eq!(domain, Domain::DeploymentAndInfrastructure);
}
