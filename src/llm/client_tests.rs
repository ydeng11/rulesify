use crate::llm::{Classifier, OpenRouterClient};

#[tokio::test]
async fn test_classification_request_error_leaves_skills_for_registry_fallback() {
    let client = OpenRouterClient {
        client: reqwest::Client::new(),
        api_key: "invalid\nheader".to_string(),
        model: "test-model".to_string(),
    };

    assert!(client.classify_batch("system", "user").await.is_err());

    let classifier = Classifier::new(client);
    let result = classifier
        .classify(vec![(
            "unclassified-skill".to_string(),
            "A skill needing classification".to_string(),
        )])
        .await
        .expect("Classification failures must not abort the registry update");

    assert!(result.is_empty());
}
