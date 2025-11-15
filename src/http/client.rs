/// Minimal HTTP Client - directly exposes hyper-util capabilities
/// Business logic (redirect, retry, auth, cookies) implemented in PHP layer

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;

use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use rustls::RootCertStore;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

use bytes::Bytes;
use futures::StreamExt;
use http_body_util::BodyExt;
use http_body_util::StreamBody;
use hyper::body::Frame;

use crate::future::RustFuture;
use crate::http::{HttpResponse, HttpResponseBody};

type HyperClient = Client<
    hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>,
>;

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

/// Internal adapter for PHP Reader interface (for request bodies)
struct PhpBodyReader {
    reader: Zval,
    chunk_size: usize,
}

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

        let mut length_zval = Zval::new();
        length_zval.set_long(to_read);

        let data_zval = this
            .reader
            .try_call_method("read", vec![&length_zval])
            .map_err(|e| std::io::Error::other(format!("PHP read failed: {:?}", e)))?;

        if data_zval.is_null() {
            return std::task::Poll::Ready(Ok(()));
        }

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

/// Minimal HTTP Client - executes HTTP requests
///
/// This client ONLY handles the HTTP protocol layer.
/// All application logic should be implemented in PHP:
/// - Redirects (follow Location headers)
/// - Retries (handle timeouts/errors)
/// - Authentication (add Authorization headers)
/// - Cookies (manage Cookie/Set-Cookie headers)
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpClient")]
pub struct HttpClient {
    client: Arc<HyperClient>,
}

unsafe impl Send for HttpClient {}
unsafe impl Sync for HttpClient {}

#[php_impl]
impl HttpClient {
    /// Create a new HTTP client
    ///
    /// # Arguments (all optional)
    /// * `ca_cert` - PEM-encoded CA certificate for custom root trust
    /// * `client_cert` - PEM-encoded client certificate for mTLS
    /// * `client_key` - PEM-encoded private key for mTLS
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

    /// Execute a single HTTP request
    ///
    /// This method ONLY executes the request. It does NOT:
    /// - Follow redirects (check status code and Location header in PHP)
    /// - Retry on failures (handle errors in PHP)
    /// - Add authentication (pass Authorization header explicitly)
    /// - Manage cookies (pass Cookie header and parse Set-Cookie in PHP)
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
    /// * `uri` - Full URI (https://example.com/path?query)
    /// * `headers` - Associative array of headers
    /// * `body` - Request body (null, string, or AsyncReader object)
    /// * `timeout` - Optional timeout in seconds (null = no timeout)
    ///
    /// # Returns
    /// Future that resolves to HttpResponse
    #[php]
    pub fn request(
        &self,
        method: String,
        uri: String,
        headers: HashMap<String, String>,
        body: Option<&Zval>,
        timeout: Option<f64>,
    ) -> RustFuture {
        let client = self.client.clone();
        let body_zval = body.map(|z| z.shallow_clone());
        let timeout_duration = timeout.map(Duration::from_secs_f64);

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

            for (key, value) in headers {
                req_builder = req_builder.header(key, value);
            }

            // Build body
            let hyper_body = if body_zval.as_ref().map_or(true, |z| z.is_null()) {
                // Empty body
                BodyExt::boxed(
                    http_body_util::Empty::<Bytes>::new()
                        .map_err(|never| match never {}),
                )
            } else if let Some(zval) = body_zval.as_ref().filter(|z| z.is_string()) {
                // String body
                let bytes = if let Some(bin) = zval.binary() {
                    Bytes::from(bin)
                } else if let Some(s) = zval.str() {
                    Bytes::from(s.as_bytes().to_vec())
                } else {
                    Bytes::new()
                };
                BodyExt::boxed(
                    http_body_util::Full::new(bytes)
                        .map_err(|never| match never {}),
                )
            } else if let Some(zval) = body_zval {
                // AsyncReader body
                let php_reader = PhpBodyReader::new(zval);
                let stream = tokio_util::io::ReaderStream::new(php_reader).map(|result| {
                    result
                        .map(Frame::data)
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                });
                BodyExt::boxed(StreamBody::new(stream))
            } else {
                // Fallback empty
                BodyExt::boxed(
                    http_body_util::Empty::<Bytes>::new()
                        .map_err(|never| match never {}),
                )
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

            // Convert to HttpResponse and return as Zval
            Self::convert_response(response).await
        })
    }
}

// Internal implementation
impl HttpClient {
    /// Build hyper client with optional custom certificates
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
            let mut cert_cursor = std::io::Cursor::new(cert_pem.as_bytes());
            let certs = rustls_pemfile::certs(&mut cert_cursor)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to parse client certificate: {}", e))?;

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

        // Get Content-Encoding for automatic decompression
        let content_encoding = parts
            .headers
            .get("content-encoding")
            .and_then(|v| v.to_str().ok());

        // Create response body with optional decompression
        let response_body = HttpResponseBody::new_internal(body, content_encoding);

        // Create HttpResponse
        let response_obj = HttpResponse::new_internal(
            parts.status.as_u16(),
            parts.version,
            parts.headers,
            response_body,
        );

        response_obj
            .into_zval(false)
            .map_err(|e| format!("Failed to convert response to Zval: {:?}", e))
    }
}
