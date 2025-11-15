/// Minimal HTTP Transport - executes single HTTP requests
/// Inspired by Go's http.RoundTripper interface

use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::Zval;

use bytes::Bytes;
use futures::StreamExt;
use http_body_util::BodyExt;
use http_body_util::StreamBody;
use hyper::body::Frame;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use rustls::RootCertStore;
use rustls_pki_types::CertificateDer;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::future::RustFuture;
use crate::http::{HttpRequest, HttpResponse, HttpResponseBody};

type HyperClient = Client<
    hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    http_body_util::combinators::BoxBody<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>,
>;

/// Local executor for single-threaded async runtime
#[derive(Clone, Copy)]
struct LocalExecutor;

/// Internal adapter for PHP Reader interface
/// This is used for streaming request bodies from PHP objects
///
/// SAFETY: This performs synchronous calls in poll_read, which is safe
/// because the PHP extension runs in a single-threaded async runtime.
struct PhpBodyReader {
    reader: Zval,
    chunk_size: usize,
}

// SAFETY: Safe because runtime is single-threaded
unsafe impl Send for PhpBodyReader {}
unsafe impl Sync for PhpBodyReader {}

impl PhpBodyReader {
    fn new(reader: Zval) -> Self {
        Self {
            reader,
            chunk_size: 8192,
        }
    }
}

impl tokio::io::AsyncRead for PhpBodyReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let to_read = std::cmp::min(buf.remaining(), this.chunk_size) as i64;

        // Call PHP's read method
        let mut length_zval = Zval::new();
        length_zval.set_long(to_read);

        let data_zval = this
            .reader
            .try_call_method("read", vec![&length_zval])
            .map_err(|e| std::io::Error::other(format!("PHP read failed: {:?}", e)))?;

        // Check for EOF
        if data_zval.is_null() {
            return std::task::Poll::Ready(Ok(()));
        }

        // Extract data (binary-safe)
        if let Some(bytes) = data_zval.binary() {
            buf.put_slice(&bytes);
        } else if let Some(s) = data_zval.str() {
            buf.put_slice(s.as_bytes());
        } else {
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Read did not return string or binary",
            )));
        }

        std::task::Poll::Ready(Ok(()))
    }
}

impl<F> hyper::rt::Executor<F> for LocalExecutor
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}

/// Pure HTTP transport - executes exactly one request without any business logic
/// All features like redirects, cookies, retry, auth must be implemented in PHP layer
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\Transport")]
pub struct HttpTransport {
    /// Shared hyper client for connection pooling
    client: Arc<HyperClient>,
}

// SAFETY: Safe because runtime is single-threaded
unsafe impl Send for HttpTransport {}
unsafe impl Sync for HttpTransport {}

#[php_impl]
impl HttpTransport {
    /// Create transport with optional TLS configuration
    ///
    /// # Arguments
    /// * `ca_cert` - Optional PEM-encoded CA certificate(s) for custom root trust
    /// * `client_cert` - Optional PEM-encoded client certificate for mTLS
    /// * `client_key` - Optional PEM-encoded private key for mTLS
    #[php(constructor)]
    pub fn __construct(
        ca_cert: Option<String>,
        client_cert: Option<String>,
        client_key: Option<String>,
    ) -> PhpResult<Self> {
        let client = Self::build_client(
            ca_cert.as_deref(),
            client_cert.as_deref(),
            client_key.as_deref(),
        )
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Execute single HTTP request and return response
    ///
    /// This is the core transport method - it ONLY executes the HTTP request.
    /// It does NOT handle:
    /// - Redirects (PHP must follow Location headers)
    /// - Cookies (PHP must manage Cookie/Set-Cookie headers)
    /// - Retry (PHP must retry on failures)
    /// - Auth (PHP must inject Authorization headers)
    ///
    /// # Arguments
    /// * `request` - The HTTP request to execute
    /// * `timeout` - Optional timeout in seconds (null = no timeout)
    ///
    /// # Returns
    /// RustFuture that resolves to HttpResponse (as Zval)
    #[php]
    pub fn execute(&self, request: &HttpRequest, timeout: Option<f64>) -> RustFuture {
        let client = self.client.clone();
        let method = request.get_method();
        let uri = request.get_uri();
        let headers = request.get_headers();
        let body_zval = request.get_body();
        let timeout_duration = timeout.map(|s| Duration::from_secs_f64(s));

        RustFuture::new(async move {
            // Parse method and URI
            let http_method = method
                .parse::<hyper::Method>()
                .map_err(|e| format!("Invalid HTTP method '{}': {}", method, e))?;
            let http_uri = uri
                .parse::<hyper::Uri>()
                .map_err(|e| format!("Invalid URI '{}': {}", uri, e))?;

            // Build request with headers
            let mut req_builder = hyper::Request::builder()
                .method(http_method)
                .uri(http_uri);

            for (key, value) in &headers {
                req_builder = req_builder.header(key, value);
            }

            // Build streaming body from PHP Reader
            let hyper_body = if body_zval.is_null() {
                BodyExt::boxed(
                    http_body_util::Empty::<Bytes>::new()
                        .map_err(|never| match never {}),
                )
            } else {
                let php_reader = PhpBodyReader::new(body_zval);
                let stream = tokio_util::io::ReaderStream::new(php_reader).map(|result| {
                    result
                        .map(Frame::data)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                });
                BodyExt::boxed(StreamBody::new(stream))
            };

            let hyper_request = req_builder
                .body(hyper_body)
                .map_err(|e| format!("Failed to build request: {}", e))?;

            // Execute request with optional timeout
            let response = if let Some(duration) = timeout_duration {
                tokio::time::timeout(duration, client.request(hyper_request))
                    .await
                    .map_err(|_| "Request timeout".to_string())?
                    .map_err(|e| format!("Request failed: {}", e))?
            } else {
                client
                    .request(hyper_request)
                    .await
                    .map_err(|e| format!("Request failed: {}", e))?
            };

            // Convert hyper response to HttpResponse (Zval)
            Self::convert_response(response).await
        })
    }
}

// Non-exported helpers
impl HttpTransport {
    /// Build a hyper client with optional custom certificates
    fn build_client(
        ca_cert: Option<&str>,
        client_cert: Option<&str>,
        client_key: Option<&str>,
    ) -> Result<HyperClient, String> {
        let mut root_store = RootCertStore::empty();

        // Add custom CA certificates if provided
        if let Some(ca_pem) = ca_cert {
            let mut cursor = std::io::Cursor::new(ca_pem.as_bytes());
            let certs = rustls_pemfile::certs(&mut cursor)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to parse CA certificates: {}", e))?;

        for cert in certs {
                root_store
                    .add(cert)
                    .map_err(|e| format!("Failed to add CA certificate: {}", e))?;
            }
        } else {
            // Use native system certificates
            root_store = webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .map(|ta| ta.to_owned())
                .collect();
        }

        let config_builder = rustls::ClientConfig::builder().with_root_certificates(root_store);

        // Add client certificate if provided (mTLS)
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

    /// Convert hyper response to HttpResponse (Zval)
    async fn convert_response(
        response: hyper::Response<hyper::body::Incoming>,
    ) -> Result<Zval, String> {
        let (parts, body) = response.into_parts();

        // Convert status code
        let status_code = parts.status.as_u16();
        let reason_phrase = parts
            .status
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        // Convert HTTP version
        let version = match parts.version {
            hyper::Version::HTTP_09 => "0.9",
            hyper::Version::HTTP_10 => "1.0",
            hyper::Version::HTTP_11 => "1.1",
            hyper::Version::HTTP_2 => "2.0",
            hyper::Version::HTTP_3 => "3.0",
            _ => "1.1",
        }
        .to_string();

        // Convert headers
        let mut headers = HashMap::new();
        for (name, value) in parts.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.to_string(), value_str.to_string());
            }
        }

        // Get Content-Encoding for automatic decompression
        let content_encoding = headers.get("content-encoding").map(|s| s.as_str());

        // Wrap body in streaming reader with optional decompression
        let response_body = HttpResponseBody::new_internal(body, content_encoding);

        // Create HttpResponse and convert to Zval
        let response_obj = HttpResponse::new_internal(
            status_code,
            reason_phrase,
            version,
            headers,
            response_body,
        );

        response_obj
            .into_zval(false)
            .map_err(|e| format!("Failed to convert response to Zval: {:?}", e))
    }
}
