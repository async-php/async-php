/// HTTP request retry logic with exponential backoff
use std::time::Duration;

/// Configuration for request retry behavior
#[derive(Clone, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_retries: u32,
    /// Initial backoff duration before first retry
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential backoff (default: 2.0)
    pub backoff_multiplier: f64,
    /// Whether to retry on timeout errors
    pub retry_on_timeout: bool,
    /// HTTP status codes that should trigger a retry (e.g., 429, 500, 502, 503, 504)
    #[allow(dead_code)]
    pub retry_status_codes: Vec<u16>,
}

impl RetryConfig {
    /// Create a new retry configuration with sensible defaults
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_status_codes: vec![429, 500, 502, 503, 504],
        }
    }

    /// Calculate backoff duration for a given retry attempt
    /// attempt: 0-based retry attempt number (0 = first retry)
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let backoff_secs = self.initial_backoff.as_secs_f64() * multiplier;
        let backoff = Duration::from_secs_f64(backoff_secs.min(self.max_backoff.as_secs_f64()));
        backoff
    }

    /// Check if a status code should trigger a retry
    #[allow(dead_code)]
    pub fn should_retry_status(&self, status_code: u16) -> bool {
        self.retry_status_codes.contains(&status_code)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff, Duration::from_secs(1));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.retry_on_timeout);
    }

    #[test]
    fn test_calculate_backoff_exponential() {
        let config = RetryConfig::new();

        // First retry: 1s * 2^0 = 1s
        assert_eq!(config.calculate_backoff(0), Duration::from_secs(1));

        // Second retry: 1s * 2^1 = 2s
        assert_eq!(config.calculate_backoff(1), Duration::from_secs(2));

        // Third retry: 1s * 2^2 = 4s
        assert_eq!(config.calculate_backoff(2), Duration::from_secs(4));

        // Fourth retry: 1s * 2^3 = 8s
        assert_eq!(config.calculate_backoff(3), Duration::from_secs(8));
    }

    #[test]
    fn test_calculate_backoff_max_limit() {
        let config = RetryConfig {
            max_retries: 10,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_status_codes: vec![],
        };

        // 1s * 2^10 = 1024s, but should be capped at 10s
        assert_eq!(config.calculate_backoff(10), Duration::from_secs(10));
    }

    #[test]
    fn test_should_retry_status() {
        let config = RetryConfig::new();

        assert!(config.should_retry_status(429)); // Too Many Requests
        assert!(config.should_retry_status(500)); // Internal Server Error
        assert!(config.should_retry_status(502)); // Bad Gateway
        assert!(config.should_retry_status(503)); // Service Unavailable
        assert!(config.should_retry_status(504)); // Gateway Timeout

        assert!(!config.should_retry_status(200)); // OK
        assert!(!config.should_retry_status(404)); // Not Found
        assert!(!config.should_retry_status(400)); // Bad Request
    }
}
