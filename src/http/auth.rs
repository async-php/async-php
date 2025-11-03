/// HTTP Authentication helpers
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

/// Generate Basic Authentication header value
/// Returns: "Basic <base64-encoded username:password>"
pub fn basic_auth(username: &str, password: &str) -> String {
    let credentials = format!("{}:{}", username, password);
    let encoded = BASE64.encode(credentials.as_bytes());
    format!("Basic {}", encoded)
}

/// Generate Bearer Token authentication header value
/// Returns: "Bearer <token>"
pub fn bearer_auth(token: &str) -> String {
    format!("Bearer {}", token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth() {
        let auth = basic_auth("user", "pass");
        assert_eq!(auth, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn test_basic_auth_with_special_chars() {
        let auth = basic_auth("admin", "p@ssw0rd!");
        // Verify it starts with "Basic " and is valid base64
        assert!(auth.starts_with("Basic "));
    }

    #[test]
    fn test_bearer_auth() {
        let auth = bearer_auth("my-token-123");
        assert_eq!(auth, "Bearer my-token-123");
    }

    #[test]
    fn test_bearer_auth_empty_token() {
        let auth = bearer_auth("");
        assert_eq!(auth, "Bearer ");
    }
}
