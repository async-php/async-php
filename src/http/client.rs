/// HTTP Client implementation

use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;

use std::time::Duration;
use http_body_util::BodyExt;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use hyper_rustls::HttpsConnectorBuilder;
use bytes::Bytes;
use hyper::body::Frame;
use http_body_util::StreamBody;
use futures::StreamExt;

use crate::http::{HttpRequest, HttpResponse, HttpResponseBody, PhpReaderAdapter};
use crate::future::RustFuture;

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
    #[php]
    pub fn send(&self, request: &HttpRequest) -> RustFuture {
        let timeout = self.timeout;
        let method = request.get_method();
        let uri = request.get_uri();
        let headers = request.get_headers();
        let body_zval = request.get_body();

        RustFuture::new(async move {
            // Build HTTPS client
            let https = HttpsConnectorBuilder::new()
                .with_native_roots()
                .map_err(|e| format!("Failed to build HTTPS connector: {}", e))?
                .https_or_http()
                .enable_http1()
                .build();

            let client = Client::builder(TokioExecutor::new()).build(https);

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
                    .map(|result| result.map(Frame::data));
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
        })
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
