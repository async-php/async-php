/// HTTP Response - wraps reqwest::Response
///
/// Provides multiple ways to read response data:
/// - Text (UTF-8 string)
/// - JSON (parsed automatically)
/// - Bytes (raw binary)
/// - Stream (via HttpResponseBody for chunked reading)

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

use reqwest::Response;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::future::RustFuture;
use crate::http::HttpResponseBody;

/// HTTP Response
///
/// Wraps reqwest::Response and provides methods to:
/// - Access status code, headers, version
/// - Read body in various formats
/// - Stream body data
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponse")]
pub struct HttpResponse {
    // Store response in Arc<Mutex> to allow multiple method calls
    response: Arc<Mutex<Option<Response>>>,
    // Cache status and headers on creation
    status_code: u16,
    headers: HashMap<String, String>,
    version: String,
}

unsafe impl Send for HttpResponse {}
unsafe impl Sync for HttpResponse {}

impl HttpResponse {
    /// Internal constructor from reqwest::Response
    pub(crate) fn new_internal(response: Response) -> Self {
        let status_code = response.status().as_u16();
        let version = format!("{:?}", response.version());

        // Extract headers (convert to HashMap)
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.as_str().to_lowercase(), value_str.to_string());
            }
        }

        Self {
            response: Arc::new(Mutex::new(Some(response))),
            status_code,
            headers,
            version,
        }
    }
}

#[php_impl]
impl HttpResponse {
    /// Get HTTP status code (e.g., 200, 404, 500)
    #[php]
    pub fn status(&self) -> u16 {
        self.status_code
    }

    /// Check if status is 2xx (success)
    #[php]
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Check if status is 4xx (client error)
    #[php]
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// Check if status is 5xx (server error)
    #[php]
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500 && self.status_code < 600
    }

    /// Get HTTP version string (e.g., "HTTP/1.1", "HTTP/2.0")
    #[php]
    pub fn version(&self) -> String {
        self.version.clone()
    }

    /// Get all response headers as associative array
    #[php]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    /// Get a specific header value (case-insensitive)
    #[php]
    pub fn header(&self, name: String) -> Option<String> {
        self.headers.get(&name.to_lowercase()).cloned()
    }

    /// Get Content-Type header
    #[php]
    pub fn content_type(&self) -> Option<String> {
        self.header("content-type".to_string())
    }

    /// Get Content-Length header
    #[php]
    pub fn content_length(&self) -> Option<i64> {
        self.header("content-length".to_string())
            .and_then(|s| s.parse().ok())
    }

    /// Read entire response body as text (UTF-8)
    ///
    /// Returns a Future that resolves to the text string.
    /// Note: This consumes the response body.
    #[php]
    pub fn text(&self) -> RustFuture {
        let response_opt = {
            let mut guard = self.response.lock().unwrap();
            guard.take()
        };

        RustFuture::new(async move {
            let response = response_opt
                .ok_or_else(|| "Response body already consumed".to_string())?;

            let text = response
                .text()
                .await
                .map_err(|e| format!("Failed to read text: {}", e))?;

            let mut zval = Zval::new();
            zval.set_string(&text, false)
                .map_err(|e| format!("Failed to set string: {:?}", e))?;
            Ok::<Zval, String>(zval)
        })
    }

    /// Read entire response body as JSON and parse it
    ///
    /// Returns a Future that resolves to the parsed JSON string.
    /// Note: This consumes the response body.
    #[php]
    pub fn json(&self) -> RustFuture {
        let response_opt = {
            let mut guard = self.response.lock().unwrap();
            guard.take()
        };

        RustFuture::new(async move {
            let response = response_opt
                .ok_or_else(|| "Response body already consumed".to_string())?;

            let json_value: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            let json_str = serde_json::to_string(&json_value)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

            let mut zval = Zval::new();
            zval.set_string(&json_str, false)
                .map_err(|e| format!("Failed to set string: {:?}", e))?;
            Ok::<Zval, String>(zval)
        })
    }

    /// Read entire response body as bytes
    ///
    /// Returns a Future that resolves to binary data.
    /// Note: This consumes the response body.
    #[php]
    pub fn bytes(&self) -> RustFuture {
        let response_opt = {
            let mut guard = self.response.lock().unwrap();
            guard.take()
        };

        RustFuture::new(async move {
            let response = response_opt
                .ok_or_else(|| "Response body already consumed".to_string())?;

            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read bytes: {}", e))?;

            let mut zval = Zval::new();
            zval.set_binary(bytes.to_vec());
            Ok::<Zval, String>(zval)
        })
    }

    /// Get streaming body reader for chunked reading
    ///
    /// Returns HttpResponseBody which implements AsyncReader interface.
    /// This is the best option for large responses or streaming data.
    /// Note: This consumes the response.
    #[php]
    pub fn stream(&self) -> PhpResult<HttpResponseBody> {
        let response_opt = {
            let mut guard = self.response.lock().unwrap();
            guard.take()
        };

        let response = response_opt
            .ok_or_else(|| "Response body already consumed".to_string())?;

        // Get content encoding first (before moving response)
        let content_encoding = response
            .headers()
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Convert reqwest response to streaming body
        Ok(HttpResponseBody::from_reqwest(response, content_encoding.as_deref()))
    }
}
