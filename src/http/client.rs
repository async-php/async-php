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

    /// Send a request and return a future that resolves to HttpResponse
    /// This is the main method for sending HTTP requests
    #[php]
    pub fn send(&self, request: &HttpRequest) -> RustFuture {
        let client = self.clone();
        let req = request.clone();

        RustFuture::new(async move {

        })
    }

    /// Create a new client with shared configuration
    /// This allows for connection pooling, cookie persistence, etc.
    pub fn clone(&self) -> Self {
        Self {
            timeout: self.timeout,
            follow_redirects: self.follow_redirects,
            max_redirects: self.max_redirects,
        }
    }
}
