/// HTTP Client - wraps reqwest::Client with full feature exposure
///
/// This module exposes all reqwest capabilities to PHP:
/// - Connection pooling (automatic)
/// - Timeout configuration
/// - Redirect handling
/// - Cookie management
/// - Proxy support
/// - Custom TLS/SSL configuration
/// - HTTP/1.1 and HTTP/2 support

use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;

use reqwest::tls::Version as TlsVersion;
use std::sync::Arc;
use std::time::Duration;

use crate::future::RustFuture;
use crate::http::HttpRequest;

/// HTTP Client - full-featured HTTP client based on reqwest
///
/// This client exposes all reqwest capabilities including:
/// - Automatic connection pooling
/// - Configurable timeouts (connect, read, request)
/// - Redirect policy control
/// - Cookie jar integration
/// - Proxy configuration
/// - Custom TLS certificates and client auth
/// - HTTP/2 support
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpClient")]
pub struct HttpClient {
    client: Arc<reqwest::Client>,
}

unsafe impl Send for HttpClient {}
unsafe impl Sync for HttpClient {}

#[php_impl]
impl HttpClient {
    /// Create a new HTTP client with builder options
    ///
    /// # Arguments (all optional)
    /// * `timeout_secs` - Total request timeout in seconds
    /// * `connect_timeout_secs` - Connection timeout in seconds
    /// * `pool_idle_timeout_secs` - Connection pool idle timeout
    /// * `pool_max_idle_per_host` - Max idle connections per host
    /// * `max_redirects` - Maximum number of redirects (0 = no redirects)
    /// * `enable_cookies` - Enable automatic cookie handling
    /// * `enable_http2` - Enable HTTP/2 protocol
    /// * `ca_cert_pem` - Custom CA certificate in PEM format
    /// * `client_cert_pem` - Client certificate for mTLS
    /// * `client_key_pem` - Client private key for mTLS
    /// * `min_tls_version` - Minimum TLS version ("1.0", "1.1", "1.2", "1.3")
    /// * `accept_invalid_certs` - Accept invalid/self-signed certificates (DANGEROUS)
    #[php(constructor)]
    pub fn __construct(
        timeout_secs: Option<f64>,
        connect_timeout_secs: Option<f64>,
        pool_idle_timeout_secs: Option<f64>,
        pool_max_idle_per_host: Option<i64>,
        max_redirects: Option<i64>,
        enable_cookies: Option<bool>,
        enable_http2: Option<bool>,
        ca_cert_pem: Option<String>,
        client_cert_pem: Option<String>,
        client_key_pem: Option<String>,
        min_tls_version: Option<String>,
        accept_invalid_certs: Option<bool>,
    ) -> PhpResult<Self> {
        let mut builder = reqwest::Client::builder();

        // Timeouts
        if let Some(secs) = timeout_secs {
            builder = builder.timeout(Duration::from_secs_f64(secs));
        }
        if let Some(secs) = connect_timeout_secs {
            builder = builder.connect_timeout(Duration::from_secs_f64(secs));
        }

        // Connection pool
        if let Some(secs) = pool_idle_timeout_secs {
            builder = builder.pool_idle_timeout(Duration::from_secs_f64(secs));
        }
        if let Some(max) = pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(max as usize);
        }

        // Redirects
        if let Some(max) = max_redirects {
            if max == 0 {
                builder = builder.redirect(reqwest::redirect::Policy::none());
            } else {
                builder = builder.redirect(reqwest::redirect::Policy::limited(max as usize));
            }
        }

        // Cookies
        if enable_cookies.unwrap_or(false) {
            builder = builder.cookie_store(true);
        }

        // HTTP/2 is enabled by default in reqwest
        // Set http1_only if user explicitly disabled HTTP/2
        if let Some(h2) = enable_http2 {
            if !h2 {
                builder = builder.http1_only();
            }
        }

        // TLS configuration
        if let Some(min_ver) = min_tls_version {
            let tls_ver = match min_ver.as_str() {
                "1.0" => TlsVersion::TLS_1_0,
                "1.1" => TlsVersion::TLS_1_1,
                "1.2" => TlsVersion::TLS_1_2,
                "1.3" => TlsVersion::TLS_1_3,
                _ => return Err(PhpException::default(format!("Invalid TLS version: {}", min_ver))),
            };
            builder = builder.min_tls_version(tls_ver);
        }

        if accept_invalid_certs.unwrap_or(false) {
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Custom CA certificate
        if let Some(ca_pem) = ca_cert_pem {
            let cert = reqwest::Certificate::from_pem(ca_pem.as_bytes())
                .map_err(|e| format!("Failed to parse CA certificate: {}", e))?;
            builder = builder.add_root_certificate(cert);
        }

        // Client certificate (mTLS)
        if let (Some(cert_pem), Some(key_pem)) = (client_cert_pem, client_key_pem) {
            // Combine cert and key for reqwest
            let combined_pem = format!("{}{}", cert_pem, key_pem);
            let identity = reqwest::Identity::from_pem(combined_pem.as_bytes())
                .map_err(|e| format!("Failed to create client identity: {}", e))?;
            builder = builder.identity(identity);
        }

        // Build client
        let client = builder
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Create a simple HTTP client with default settings
    #[php]
    pub fn create_simple() -> PhpResult<Self> {
        let client = reqwest::Client::new();
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Start building a GET request
    #[php]
    pub fn get(&self, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), "GET", url))
    }

    /// Start building a POST request
    #[php]
    pub fn post(&self, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), "POST", url))
    }

    /// Start building a PUT request
    #[php]
    pub fn put(&self, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), "PUT", url))
    }

    /// Start building a PATCH request
    #[php]
    pub fn patch(&self, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), "PATCH", url))
    }

    /// Start building a DELETE request
    #[php]
    pub fn delete(&self, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), "DELETE", url))
    }

    /// Start building a HEAD request
    #[php]
    pub fn head(&self, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), "HEAD", url))
    }

    /// Start building a request with custom method
    #[php]
    pub fn request(&self, method: String, url: String) -> PhpResult<HttpRequest> {
        Ok(HttpRequest::new(self.client.clone(), &method, url))
    }

    /// Execute a simple GET request (convenience method)
    #[php]
    pub fn quick_get(&self, url: String) -> RustFuture {
        let client = self.client.clone();
        RustFuture::new(async move {
            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            let response_obj = crate::http::HttpResponse::new_internal(response);
            response_obj
                .into_zval(false)
                .map_err(|e| format!("Failed to convert response: {:?}", e))
        })
    }

    /// Execute a simple POST request with JSON body (convenience method)
    #[php]
    pub fn quick_post_json(&self, url: String, json: String) -> RustFuture {
        let client = self.client.clone();
        RustFuture::new(async move {
            let json_value: serde_json::Value = serde_json::from_str(&json)
                .map_err(|e| format!("Invalid JSON: {}", e))?;

            let response = client
                .post(&url)
                .json(&json_value)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            let response_obj = crate::http::HttpResponse::new_internal(response);
            response_obj
                .into_zval(false)
                .map_err(|e| format!("Failed to convert response: {:?}", e))
        })
    }
}
