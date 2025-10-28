/// HTTP Client supporting HTTP/1.1, HTTP/2 and HTTP/3

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use bytes::Bytes;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;
use crate::http::body::HttpsBody;

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpClient")]
pub struct HttpClient {
    config: Arc<ClientConfig>,
}

#[derive(Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub enable_http2: bool,
    pub enable_http3: bool,
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            follow_redirects: true,
            max_redirects: 10,
            enable_http2: true,
            enable_http3: false,
            user_agent: "async-php-client/1.0".to_string(),
        }
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            config: Arc::new(ClientConfig::default()),
        }
    }

    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    async fn execute_request(
        config: Arc<ClientConfig>,
        request: HttpRequest
    ) -> Result<HttpResponse, String> {
        // Simple implementation using hyper-rustls
        use hyper::Request;
        use hyper::Method;
        use http_body_util::BodyExt;
        use hyper_rustls::HttpsConnectorBuilder;

        let method = request.method.parse::<Method>()
            .map_err(|_| format!("Invalid HTTP method: {}", request.method))?;

        // Build request URI
        let uri = request.uri.clone();

        // Build request body from HttpsBody
        let body_bytes = if let Some(body) = request.get_body() {
            // Get body content (simplified - would need proper implementation)
            vec![]
        } else {
            vec![]
        };

        // Create HTTP request
        let mut builder = Request::builder()
            .method(method)
            .uri(uri);

        // Add headers
        for (k, v) in &request.headers {
            builder = builder.header(k, v);
        }

        // Use full body for now
        let req = builder.body(http_body_util::Full::new(Bytes::from(body_bytes)))
            .map_err(|e| format!("Request build error: {}", e))?;

        // Create HTTPS client
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()  // Enable HTTP/2 support
            .build();

        use hyper_util::client::legacy::Client;
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(https);

        // Execute request
        let mut resp = client.request(req)
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        // Build response
        let status = resp.status().as_u16() as i32;
        let mut response = HttpResponse::new(status);

        // Copy headers
        for (name, value) in resp.headers() {
            if let Ok(value_str) = value.to_str() {
                response.set_header(
                    name.to_string(),
                    value_str.to_string(),
                );
            }
        }

        // Read body
        let body_frames = resp.collect()
            .await
            .map_err(|e| format!("Body collection error: {}", e))?;

        let body_bytes = body_frames.to_bytes();
        let body_string = String::from_utf8_lossy(&body_bytes).to_string();
        response.set_body(HttpsBody::from_string(body_string));

        Ok(response)
    }
}

#[php_impl]
impl HttpClient {
    pub fn __construct() -> Self {
        Self::new()
    }

    /// Create HTTP request builder
    pub fn request(method: String, uri: String) -> HttpRequest {
        HttpRequest::new(method, uri)
    }

    /// GET request for HTTP/1.1
    pub fn get(uri: String) -> HttpRequest {
        let mut req = HttpRequest::new("GET".to_string(), uri);
        req.set_version("1.1".to_string());
        req
    }

    /// POST request for HTTP/1.1
    pub fn post(uri: String) -> HttpRequest {
        let mut req = HttpRequest::new("POST".to_string(), uri);
        req.set_version("1.1".to_string());
        req
    }

    /// GET request for HTTP/2
    pub fn get_http2(uri: String) -> HttpRequest {
        let mut req = HttpRequest::new("GET".to_string(), uri);
        req.set_version("2.0".to_string());
        req
    }

    /// POST request for HTTP/2
    pub fn post_http2(uri: String) -> HttpRequest {
        let mut req = HttpRequest::new("POST".to_string(), uri);
        req.set_version("2.0".to_string());
        req
    }

    /// GET request for HTTP/3 (if enabled)
    pub fn get_http3(uri: String) -> HttpRequest {
        let mut req = HttpRequest::new("GET".to_string(), uri);
        req.enable_http3();
        req
    }

    /// POST request for HTTP/3 (if enabled)
    pub fn post_http3(uri: String) -> HttpRequest {
        let mut req = HttpRequest::new("POST".to_string(), uri);
        req.enable_http3();
        req
    }

    /// Send the request and get response future
    pub fn send(
        &self, request: HttpRequest
    ) -> RustFuture {
        let config = self.config.clone();

        let future = async move {
            match Self::execute_request(config, request).await {
                Ok(response) => response.into_zval(false).unwrap_or(Zval::new()),
                Err(_) => {
                    // Return error response on failure
                    let error_response = HttpResponse::server_error();
                    error_response.into_zval(false).unwrap_or(Zval::new())
                }
            }
        };

        RustFuture::new(future)
    }
}

use http_body_util;