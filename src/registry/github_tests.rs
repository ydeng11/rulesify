#[cfg(test)]
mod tests {
    use crate::registry::GitHubClient;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new();
        assert!(client.token.is_none());

        let client_with_token = GitHubClient::with_token(Some("test".to_string()));
        assert!(client_with_token.token.is_some());
    }
}
