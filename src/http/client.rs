/// HTTP Client implementation

use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;

use std::time::Duration;
use std::sync::Arc;
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
        let method = request.get_method();
        let uri = request.get_uri();
        let headers = request.get_headers();
        let body_zval = request.get_body();
        let client = self.client.as_ref().unwrap().clone();

        Ok(RustFuture::new(async move {
            // Parse method and URI
            let http_method = method.parse::<hyper::Method>()
                .map_err(|e| format!("Invalid HTTP method '{}': {}", method, e))?;
            let http_uri = uri.parse::<hyper::Uri>()
                .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;

            // Build request with headers
            let mut req_builder = hyper::Request::builder()
                .method(http_method)
                .uri(http_uri);

            for (key, value) in headers {
                req_builder = req_builder.header(key, value);
            }

            // Build streaming body
            let hyper_body = if body_zval.is_null() {
                BodyExt::boxed(
                    http_body_util::Empty::<Bytes>::new()
                        .map_err(|never| match never {})
                )
            } else {
                let php_reader = PhpReaderAdapter::new(body_zval);
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

            // Build response
            let (parts, body) = response.into_parts();
            let mut http_response = HttpResponse::__construct(parts.status.as_u16() as i32);

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

            ext_php_rs::types::ZendClassObject::new(http_response)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert HttpResponse to Zval: {:?}", e))
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
        }
    }
}
