/// HTTP Client implementation

use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;

use std::time::Duration;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::Semaphore;
use http_body_util::BodyExt;
use hyper_util::client::legacy::Client;
use hyper_rustls::HttpsConnectorBuilder;
use rustls::RootCertStore;
use rustls_pki_types::CertificateDer;
use bytes::Bytes;
use hyper::body::Frame;
use http_body_util::StreamBody;
use futures::StreamExt;

use crate::http::{HttpRequest, HttpResponse, HttpResponseBody, PhpReaderAdapter};
use crate::http::auth;
use crate::http::cookies::CookieJar;
use crate::http::retry::RetryConfig;
use crate::http::metrics::{RequestMetrics, MetricsCollector};
use crate::future::RustFuture;

type HyperClient = Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, http_body_util::combinators::BoxBody<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>>;

/// Local executor for single-threaded async runtime
#[derive(Clone, Copy)]
struct LocalExecutor;

impl<F> hyper::rt::Executor<F> for LocalExecutor
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}

/// HTTP Client for making HTTP requests
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpClient")]
pub struct HttpClient {
    /// Shared hyper client for connection pooling (wrapped in Mutex for lazy rebuild)
    client: Option<Arc<HyperClient>>,
    /// Default timeout for requests
    timeout: Option<Duration>,
    /// Follow redirects (3xx responses)
    follow_redirects: bool,
    /// Maximum number of redirects to follow
    max_redirects: u32,
    /// Custom CA certificates (PEM format)
    custom_ca_certs: Option<String>,
    /// Client certificate (PEM format)
    client_cert: Option<String>,
    /// Client private key (PEM format)
    client_key: Option<String>,
    /// Pre-computed Authorization header
    auth_header: Option<String>,
    /// Cookie jar for session management
    cookie_jar: Option<CookieJar>,
    /// Retry configuration
    retry_config: Option<RetryConfig>,
    /// Automatic response decompression
    auto_decompress: bool,
    /// Collect performance metrics
    collect_metrics: bool,
    /// Concurrent request limiter (semaphore)
    concurrency_limiter: Option<Arc<Semaphore>>,
}

#[php_impl]
impl HttpClient {
    /// Create a new HTTP client
    #[php(constructor)]
    pub fn __construct() -> Self {
        Self {
            client: None,
            timeout: Some(Duration::from_secs(30)),
            follow_redirects: true,
            max_redirects: 10,
            custom_ca_certs: None,
            client_cert: None,
            client_key: None,
            auth_header: None,
            cookie_jar: None,
            retry_config: None,
            auto_decompress: true, // Enable by default for better performance
            collect_metrics: false, // Disabled by default for performance
            concurrency_limiter: None, // No limit by default
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

    /// Set custom CA certificates in PEM format
    /// Client will be rebuilt on next request
    #[php]
    pub fn set_ca_cert(&mut self, ca_cert_pem: String) {
        self.custom_ca_certs = Some(ca_cert_pem);
    }

    /// Set client certificate and key in PEM format for mutual TLS
    /// Client will be rebuilt on next request
    #[php]
    pub fn set_client_cert(&mut self, cert_pem: String, key_pem: String) {
        self.client_cert = Some(cert_pem);
        self.client_key = Some(key_pem);
    }

    /// Set Basic Authentication
    /// Automatically adds "Authorization: Basic <base64>" header to all requests
    #[php]
    pub fn set_basic_auth(&mut self, username: String, password: String) {
        self.auth_header = Some(auth::basic_auth(&username, &password));
    }

    /// Set Bearer Token authentication
    /// Automatically adds "Authorization: Bearer <token>" header to all requests
    #[php]
    pub fn set_bearer_token(&mut self, token: String) {
        self.auth_header = Some(auth::bearer_auth(&token));
    }

    /// Clear authentication
    #[php]
    pub fn clear_auth(&mut self) {
        self.auth_header = None;
    }

    /// Enable cookie management
    /// Creates a new cookie jar to store cookies automatically
    #[php]
    pub fn enable_cookies(&mut self) {
        self.cookie_jar = Some(CookieJar::new());
    }

    /// Disable cookie management
    #[php]
    pub fn disable_cookies(&mut self) {
        self.cookie_jar = None;
    }

    /// Clear all stored cookies
    #[php]
    pub fn clear_cookies(&mut self) {
        if let Some(ref jar) = self.cookie_jar {
            jar.clear();
        }
    }

    /// Enable request retry with default configuration
    /// Default: 3 retries, exponential backoff starting at 1s
    #[php]
    pub fn enable_retry(&mut self) {
        self.retry_config = Some(RetryConfig::default());
    }

    /// Set custom retry configuration
    /// max_retries: Maximum number of retry attempts
    /// initial_backoff_secs: Initial backoff duration in seconds
    /// max_backoff_secs: Maximum backoff duration in seconds
    pub fn set_retry_config(&mut self, max_retries: u32, initial_backoff_secs: f64, max_backoff_secs: f64) {
        self.retry_config = Some(RetryConfig {
            max_retries,
            initial_backoff: Duration::from_secs_f64(initial_backoff_secs),
            max_backoff: Duration::from_secs_f64(max_backoff_secs),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_status_codes: vec![429, 500, 502, 503, 504],
        });
    }

    /// Disable request retry
    #[php]
    pub fn disable_retry(&mut self) {
        self.retry_config = None;
    }

    /// Enable automatic response decompression (gzip, deflate)
    /// Enabled by default
    #[php]
    pub fn set_auto_decompress(&mut self, enabled: bool) {
        self.auto_decompress = enabled;
    }

    /// Check if automatic decompression is enabled
    #[php]
    pub fn get_auto_decompress(&self) -> bool {
        self.auto_decompress
    }

    /// Enable performance metrics collection
    /// Disabled by default for better performance
    #[php]
    pub fn set_collect_metrics(&mut self, enabled: bool) {
        self.collect_metrics = enabled;
    }

    /// Check if metrics collection is enabled
    #[php]
    pub fn get_collect_metrics(&self) -> bool {
        self.collect_metrics
    }

    /// Set maximum concurrent requests
    /// Setting to 0 or less disables the limit
    #[php]
    pub fn set_max_concurrent_requests(&mut self, max: i64) {
        if max > 0 {
            self.concurrency_limiter = Some(Arc::new(Semaphore::new(max as usize)));
        } else {
            self.concurrency_limiter = None;
        }
    }

    /// Send a request and return a future that resolves to HttpResponse
    #[php]
    pub fn send(&mut self, request: &HttpRequest) -> PhpResult<RustFuture> {
        // Check if client needs to be rebuilt
        if self.client.is_none() {
            let client = Self::build_client(
                self.custom_ca_certs.as_deref(),
                self.client_cert.as_deref(),
                self.client_key.as_deref(),
            )?;

            self.client = Some(Arc::new(client));
        }

        let timeout = self.timeout;
        let follow_redirects = self.follow_redirects;
        let max_redirects = self.max_redirects;
        let auto_decompress = self.auto_decompress;
        let method = request.get_method();
        let uri = request.get_uri();
        let mut headers = request.get_headers();
        let body_zval = request.get_body();
        let client = self.client.as_ref().unwrap().clone();
        let cookie_jar = self.cookie_jar.clone();
        let concurrency_limiter = self.concurrency_limiter.clone();

        // Add authentication header if configured
        if let Some(ref auth) = self.auth_header {
            headers.insert("Authorization".to_string(), auth.clone());
        }

        // Add Accept-Encoding header if auto-decompression is enabled
        if auto_decompress && !headers.contains_key("Accept-Encoding") {
            headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
        }

        Ok(RustFuture::new(async move {
            // Acquire concurrency permit if limiter is set
            let _permit = if let Some(ref limiter) = concurrency_limiter {
                Some(limiter.acquire().await.map_err(|e| format!("Failed to acquire concurrency permit: {}", e))?)
            } else {
                None
            };
            // Permit is automatically released when _permit is dropped

            let mut current_uri = uri.clone();
            let mut current_method = method.clone();
            let mut current_body_zval = body_zval.shallow_clone();
            let mut visited_urls: HashSet<String> = HashSet::new();
            let mut redirect_count = 0u32;

            loop {
                // Prevent redirect loops by tracking visited URLs
                if visited_urls.contains(&current_uri) {
                    return Err("Redirect loop detected".to_string());
                }
                visited_urls.insert(current_uri.clone());

                // Parse method and URI
                let http_method = current_method.parse::<hyper::Method>()
                    .map_err(|e| format!("Invalid HTTP method '{}': {}", current_method, e))?;
                let http_uri = current_uri.parse::<hyper::Uri>()
                    .map_err(|e| format!("Invalid URI '{}': {}", current_uri, e))?;

                // Build request with headers
                let mut req_builder = hyper::Request::builder()
                    .method(http_method)
                    .uri(http_uri);

                // Add cookies from cookie jar if enabled
                if let Some(ref jar) = cookie_jar {
                    if let Some(cookie_header) = jar.get_cookies_for_url(&current_uri) {
                        req_builder = req_builder.header("Cookie", cookie_header);
                    }
                }

                for (key, value) in &headers {
                    req_builder = req_builder.header(key, value);
                }

                // Build streaming body
                let hyper_body = if current_body_zval.is_null() || redirect_count > 0 {
                    // For redirects, we don't send the body (except for 307/308)
                    BodyExt::boxed(
                        http_body_util::Empty::<Bytes>::new()
                            .map_err(|never| match never {})
                    )
                } else {
                    let php_reader = PhpReaderAdapter::new(current_body_zval.shallow_clone());
                    let stream = tokio_util::io::ReaderStream::new(php_reader)
                        .map(|result| result.map(Frame::data).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>));
                    BodyExt::boxed(StreamBody::new(stream))
                };

                let hyper_request = req_builder.body(hyper_body)
                    .map_err(|e| format!("Failed to build request: {}", e))?;

                // Execute request with optional timeout
                let response = if let Some(timeout_duration) = timeout {
                    tokio::time::timeout(timeout_duration, client.request(hyper_request))
                        .await
                        .map_err(|_| "Request timeout".to_string())?
                        .map_err(|e| format!("Request failed: {}", e))?
                } else {
                    client.request(hyper_request)
                        .await
                        .map_err(|e| format!("Request failed: {}", e))?
                };

                let (parts, body) = response.into_parts();
                let status_code = parts.status.as_u16();

                // Store cookies from Set-Cookie headers if cookie jar is enabled
                if let Some(ref jar) = cookie_jar {
                    let set_cookie_headers: Vec<&str> = parts.headers
                        .get_all("set-cookie")
                        .iter()
                        .filter_map(|v| v.to_str().ok())
                        .collect();

                    if !set_cookie_headers.is_empty() {
                        jar.store_cookies_from_response(&current_uri, set_cookie_headers);
                    }
                }

                // Check if we should follow redirects
                let is_redirect = matches!(status_code, 301 | 302 | 303 | 307 | 308);

                if is_redirect && follow_redirects && redirect_count < max_redirects {
                    // Extract Location header
                    if let Some(location) = parts.headers.get("location") {
                        if let Ok(location_str) = location.to_str() {
                            redirect_count += 1;

                            // Handle relative vs absolute URLs
                            current_uri = if location_str.starts_with("http://") || location_str.starts_with("https://") {
                                location_str.to_string()
                            } else {
                                // Parse current URI to get base URL
                                let base_uri = current_uri.parse::<hyper::Uri>()
                                    .map_err(|e| format!("Failed to parse base URI: {}", e))?;

                                let scheme = base_uri.scheme_str().unwrap_or("https");
                                let authority = base_uri.authority()
                                    .ok_or_else(|| "Missing authority in redirect base URL".to_string())?;

                                if location_str.starts_with('/') {
                                    // Absolute path
                                    format!("{}://{}{}", scheme, authority, location_str)
                                } else {
                                    // Relative path
                                    let base_path = base_uri.path();
                                    let last_slash = base_path.rfind('/').unwrap_or(0);
                                    let base_dir = &base_path[..=last_slash];
                                    format!("{}://{}{}{}", scheme, authority, base_dir, location_str)
                                }
                            };

                            // Handle method change for certain status codes
                            // 303 always changes to GET
                            // 301, 302 change POST to GET
                            match status_code {
                                303 => {
                                    current_method = "GET".to_string();
                                    current_body_zval = ext_php_rs::types::Zval::null();
                                }
                                301 | 302 if current_method == "POST" => {
                                    current_method = "GET".to_string();
                                    current_body_zval = ext_php_rs::types::Zval::null();
                                }
                                _ => {
                                    // 307, 308 keep method and body
                                }
                            }

                            continue; // Follow redirect
                        }
                    }
                }

                // Build response (no more redirects or redirect limit reached)
                let mut http_response = HttpResponse::__construct(status_code as i32);

                http_response.set_version(Self::version_to_string(parts.version).to_string());

                for (key, value) in &parts.headers {
                    if let Ok(value_str) = value.to_str() {
                        http_response.set_header(key.to_string(), value_str.to_string());
                    }
                }

                // Wrap response body
                let response_body = HttpResponseBody::new_internal(body);
                let body_zval = ext_php_rs::types::ZendClassObject::new(response_body)
                    .into_zval(false)
                    .map_err(|e| format!("Failed to create response body: {:?}", e))?;

                http_response.set_body(&body_zval)
                    .map_err(|e| format!("Failed to set response body: {:?}", e))?;

                return ext_php_rs::types::ZendClassObject::new(http_response)
                    .into_zval(false)
                    .map_err(|e| format!("Failed to convert HttpResponse to Zval: {:?}", e));
            }
        }))
    }
}

impl HttpClient {
    /// Convert hyper version to string
    fn version_to_string(version: hyper::Version) -> &'static str {
        match version {
            hyper::Version::HTTP_09 => "0.9",
            hyper::Version::HTTP_10 => "1.0",
            hyper::Version::HTTP_11 => "1.1",
            hyper::Version::HTTP_2 => "2.0",
            hyper::Version::HTTP_3 => "3.0",
            _ => "1.1",
        }
    }

    /// Build a hyper client with optional custom certificates
    fn build_client(
        custom_ca_certs: Option<&str>,
        client_cert: Option<&str>,
        client_key: Option<&str>,
    ) -> Result<HyperClient, String> {
        let mut root_store = RootCertStore::empty();

        // Add custom CA certificates if provided
        if let Some(ca_pem) = custom_ca_certs {
            let mut cursor = std::io::Cursor::new(ca_pem.as_bytes());
            let certs = rustls_pemfile::certs(&mut cursor)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to parse CA certificates: {}", e))?;

            for cert in certs {
                root_store.add(cert)
                    .map_err(|e| format!("Failed to add CA certificate: {}", e))?;
            }
        } else {
            // Use native system certificates
            root_store = webpki_roots::TLS_SERVER_ROOTS.iter()
                .map(|ta| ta.to_owned())
                .collect();
        }

        let config_builder = rustls::ClientConfig::builder()
            .with_root_certificates(root_store);

        // Add client certificate if provided
        let config = if let (Some(cert_pem), Some(key_pem)) = (client_cert, client_key) {
            // Parse client certificate
            let mut cert_cursor = std::io::Cursor::new(cert_pem.as_bytes());
            let certs = rustls_pemfile::certs(&mut cert_cursor)
                .collect::<Result<Vec<CertificateDer>, _>>()
                .map_err(|e| format!("Failed to parse client certificate: {}", e))?;

            // Parse private key
            let mut key_cursor = std::io::Cursor::new(key_pem.as_bytes());
            let key = rustls_pemfile::private_key(&mut key_cursor)
                .map_err(|e| format!("Failed to parse private key: {}", e))?
                .ok_or_else(|| "No private key found in PEM".to_string())?;

            config_builder
                .with_client_auth_cert(certs, key)
                .map_err(|e| format!("Failed to configure client certificate: {}", e))?
        } else {
            config_builder.with_no_client_auth()
        };

        let https = HttpsConnectorBuilder::new()
            .with_tls_config(config)
            .https_or_http()
            .enable_http1()
            .build();

        Ok(Client::builder(LocalExecutor).build(https))
    }

    /// Create a new client with shared configuration
    /// This allows for connection pooling, cookie persistence, etc.
    pub fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            timeout: self.timeout,
            follow_redirects: self.follow_redirects,
            max_redirects: self.max_redirects,
            custom_ca_certs: self.custom_ca_certs.clone(),
            client_cert: self.client_cert.clone(),
            client_key: self.client_key.clone(),
            auth_header: self.auth_header.clone(),
            cookie_jar: self.cookie_jar.clone(),
            retry_config: self.retry_config.clone(),
            auto_decompress: self.auto_decompress,
            collect_metrics: self.collect_metrics,
            concurrency_limiter: self.concurrency_limiter.clone(),
        }
    }
}
