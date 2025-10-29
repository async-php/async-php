/// HTTP Client implementation

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::http::{HttpRequest, HttpResponse};
use crate::future::RustFuture;
use futures::FutureExt;
use std::time::Duration;

/// HTTP Client for making HTTP requests
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpClient")]
pub struct HttpClient {
    /// Default timeout for requests
    timeout: Option<Duration>,
    /// Default headers to send with all requests
    default_headers: Vec<(String, String)>,
    /// Follow redirects (3xx responses)
    follow_redirects: bool,
    /// Maximum number of redirects to follow
    max_redirects: u32,
}

#[php_impl]
impl HttpClient {
    /// Create a new HTTP client
    #[php(constructor)]
    pub fn __construct() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            default_headers: Vec::new(),
            follow_redirects: true,
            max_redirects: 10,
        }
    }

    /// Set the default timeout for requests
    pub fn set_timeout(&mut self, seconds: i64) {
        if seconds > 0 {
            self.timeout = Some(Duration::from_secs(seconds as u64));
        } else {
            self.timeout = None;
        }
    }

    /// Get the default timeout
    pub fn get_timeout(&self) -> Option<i64> {
        self.timeout.map(|d| d.as_secs() as i64)
    }

    /// Enable or disable following redirects
    pub fn set_follow_redirects(&mut self, follow: bool) {
        self.follow_redirects = follow;
    }

    /// Check if redirects are followed
    pub fn get_follow_redirects(&self) -> bool {
        self.follow_redirects
    }

    /// Set maximum number of redirects to follow
    pub fn set_max_redirects(&mut self, max: u32) {
        self.max_redirects = max;
    }

    /// Get maximum number of redirects
    pub fn get_max_redirects(&self) -> u32 {
        self.max_redirects
    }

    /// Add a default header to all requests
    pub fn add_default_header(&mut self, name: String, value: String) {
        self.default_headers.retain(|(key, _)| key.to_lowercase() != name.to_lowercase());
        self.default_headers.push((name, value));
    }

    /// Remove a default header
    pub fn remove_default_header(&mut self, name: String) {
        self.default_headers.retain(|(key, _)| key.to_lowercase() != name.to_lowercase());
    }

    /// Get all default headers
    pub fn get_default_headers(&self) -> Vec<(String, String)> {
        self.default_headers.clone()
    }

    /// Send a request and return a future that resolves to HttpResponse
    /// This is the main method for sending HTTP requests
    #[php]
    pub fn send(&self, request: &HttpRequest) -> PhpResult<Zval> {
        let client = self.clone();
        let req = request.clone();

        let fut = async move {
            // This is where the actual HTTP request would be made
            // For now, we'll return a mock response
            // In a real implementation, this would use hyper, reqwest, or similar

            // Simulate some async work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Create a mock response
            let mut response = HttpResponse::__construct(200);
            response.set_header("Content-Type".to_string(), "text/plain".to_string());
            response.set_body_string("OK".to_string());

            // The actual implementation would:
            // 1. Parse the request
            // 2. Set up the connection (with streaming if body is a Reader)
            // 3. Send headers
            // 4. Stream body if present (reading from request.get_body())
            // 5. Read response headers
            // 6. Create response object with a Reader body for streaming
            // 7. Return the response

            Ok(response)
        };

        // Convert to RustFuture for PHP consumption
        let rust_fut = RustFuture::new(fut.boxed());
        rust_fut.into_zval(false)
    }

    /// Convenience method to send a GET request
    #[php]
    pub fn get(&self, uri: String, headers: Option<Vec<(String, String)>>) -> PhpResult<Zval> {
        let mut request = HttpRequest::__construct("GET".to_string(), uri);

        // Add default headers
        for (name, value) in &self.default_headers {
            request.set_header(name.clone(), value.clone());
        }

        // Add custom headers
        if let Some(hdrs) = headers {
            for (name, value) in hdrs {
                request.set_header(name, value);
            }
        }

        self.send(&request)
    }

    /// Convenience method to send a POST request
    #[php]
    pub fn post(&self, uri: String, body: Option<Zval>, headers: Option<Vec<(String, String)>>) -> PhpResult<Zval> {
        let mut request = HttpRequest::__construct("POST".to_string(), uri);

        // Set body if provided
        if let Some(b) = body {
            request.set_body(Some(b))?;
        }

        // Add default headers
        for (name, value) in &self.default_headers {
            request.set_header(name.clone(), value.clone());
        }

        // Add custom headers
        if let Some(hdrs) = headers {
            for (name, value) in hdrs {
                request.set_header(name, value);
            }
        }

        self.send(&request)
    }

    /// Convenience method to send a PUT request
    #[php]
    pub fn put(&self, uri: String, body: Option<Zval>, headers: Option<Vec<(String, String)>>) -> PhpResult<Zval> {
        let mut request = HttpRequest::__construct("PUT".to_string(), uri);

        // Set body if provided
        if let Some(b) = body {
            request.set_body(Some(b))?;
        }

        // Add default headers
        for (name, value) in &self.default_headers {
            request.set_header(name.clone(), value.clone());
        }

        // Add custom headers
        if let Some(hdrs) = headers {
            for (name, value) in hdrs {
                request.set_header(name, value);
            }
        }

        self.send(&request)
    }

    /// Convenience method to send a DELETE request
    #[php]
    pub fn delete(&self, uri: String, headers: Option<Vec<(String, String)>>) -> PhpResult<Zval> {
        let mut request = HttpRequest::__construct("DELETE".to_string(), uri);

        // Add default headers
        for (name, value) in &self.default_headers {
            request.set_header(name.clone(), value.clone());
        }

        // Add custom headers
        if let Some(hdrs) = headers {
            for (name, value) in hdrs {
                request.set_header(name, value);
            }
        }

        self.send(&request)
    }

    /// Create a new client with shared configuration
    /// This allows for connection pooling, cookie persistence, etc.
    pub fn clone(&self) -> Self {
        Self {
            timeout: self.timeout,
            default_headers: self.default_headers.clone(),
            follow_redirects: self.follow_redirects,
            max_redirects: self.max_redirects,
        }
    }
}
