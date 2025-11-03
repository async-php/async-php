/// Cookie management for HTTP client
use cookie_store::CookieStore;
use cookie::Cookie as RawCookie;
use std::sync::{Arc, Mutex};
use url::Url;

/// Thread-safe cookie jar wrapper
#[derive(Clone)]
pub struct CookieJar {
    store: Arc<Mutex<CookieStore>>,
}

impl CookieJar {
    /// Create a new cookie jar
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(CookieStore::default())),
        }
    }

    /// Get cookies for a URL as a Cookie header value
    /// Returns None if no cookies match
    pub fn get_cookies_for_url(&self, url: &str) -> Option<String> {
        let parsed_url = Url::parse(url).ok()?;
        let store = self.store.lock().ok()?;

        let cookies: Vec<String> = store
            .get_request_values(&parsed_url)
            .map(|(name, value)| format!("{}={}", name, value))
            .collect();

        if cookies.is_empty() {
            None
        } else {
            Some(cookies.join("; "))
        }
    }

    /// Store cookies from Set-Cookie headers
    /// url: The URL from which the cookies originated
    /// set_cookie_headers: Vec of Set-Cookie header values
    pub fn store_cookies_from_response(&self, url: &str, set_cookie_headers: Vec<&str>) {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Ok(mut store) = self.store.lock() {
                for cookie_str in set_cookie_headers {
                    // Parse using cookie crate's Cookie type
                    if let Ok(raw_cookie) = RawCookie::parse(cookie_str) {
                        // Convert to owned cookie for storage
                        let owned_cookie = raw_cookie.into_owned();
                        let _ = store.insert_raw(&owned_cookie, &parsed_url);
                    }
                }
            }
        }
    }

    /// Clear all cookies
    pub fn clear(&self) {
        if let Ok(mut store) = self.store.lock() {
            *store = CookieStore::default();
        }
    }

    /// Get all cookies as a debug string
    pub fn debug_cookies(&self) -> String {
        if let Ok(store) = self.store.lock() {
            let mut result = Vec::new();
            for cookie in store.iter_any() {
                result.push(format!("{}={} (domain={:?}, path={:?})",
                    cookie.name(),
                    cookie.value(),
                    cookie.domain(),
                    cookie.path()
                ));
            }
            result.join("\n")
        } else {
            "Error: Could not lock cookie store".to_string()
        }
    }
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_jar_basic() {
        let jar = CookieJar::new();

        // Store a cookie
        jar.store_cookies_from_response(
            "https://example.com/path",
            vec!["session=abc123; Path=/; HttpOnly"]
        );

        // Retrieve cookies for the same URL
        let cookies = jar.get_cookies_for_url("https://example.com/path");
        assert!(cookies.is_some());
        assert!(cookies.unwrap().contains("session=abc123"));
    }

    #[test]
    fn test_cookie_jar_domain_matching() {
        let jar = CookieJar::new();

        // Store cookie for subdomain
        jar.store_cookies_from_response(
            "https://api.example.com/",
            vec!["token=xyz; Domain=example.com"]
        );

        // Should be available for main domain
        let cookies = jar.get_cookies_for_url("https://example.com/");
        assert!(cookies.is_some());
    }

    #[test]
    fn test_cookie_jar_clear() {
        let jar = CookieJar::new();

        jar.store_cookies_from_response(
            "https://example.com/",
            vec!["session=abc"]
        );

        jar.clear();

        let cookies = jar.get_cookies_for_url("https://example.com/");
        assert!(cookies.is_none());
    }
}
