/// Minimal HTTP Response - wraps hyper response

use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::Zval;
use std::collections::HashMap;

use crate::http::HttpResponseBody;

/// HTTP Response
///
/// Simple wrapper around HTTP response data from hyper.
/// All response processing logic should be in PHP.
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponse")]
pub struct HttpResponse {
    /// HTTP status code (e.g., 200, 404, 500)
    status_code: u16,
    /// HTTP version (e.g., HTTP/1.1, HTTP/2)
    version: String,
    /// Response headers (lowercase keys)
    headers: HashMap<String, String>,
    /// Response body (HttpResponseBody with optional decompression)
    body: Zval,
}

unsafe impl Send for HttpResponse {}
unsafe impl Sync for HttpResponse {}

#[php_impl]
impl HttpResponse {
    /// Get the HTTP status code
    pub fn get_status(&self) -> u16 {
        self.status_code
    }

    /// Get the HTTP version string
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Get all response headers
    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    /// Get a specific header value (case-insensitive)
    pub fn get_header(&self, name: String) -> Option<String> {
        let name_lower = name.to_lowercase();
        self.headers.get(&name_lower).cloned()
    }

    /// Get the response body
    pub fn get_body(&self) -> Zval {
        self.body.shallow_clone()
    }
}

// Internal constructor (used by HttpClient)
impl HttpResponse {
    pub(crate) fn new_internal(
        status_code: u16,
        version: hyper::Version,
        headers: hyper::HeaderMap,
        body: HttpResponseBody,
    ) -> Self {
        // Convert version
        let version_str = match version {
            hyper::Version::HTTP_09 => "HTTP/0.9",
            hyper::Version::HTTP_10 => "HTTP/1.0",
            hyper::Version::HTTP_11 => "HTTP/1.1",
            hyper::Version::HTTP_2 => "HTTP/2",
            hyper::Version::HTTP_3 => "HTTP/3",
            _ => "HTTP/1.1",
        }
        .to_string();

        // Convert headers (normalize to lowercase keys)
        let mut headers_map = HashMap::new();
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                headers_map.insert(name.as_str().to_lowercase(), value_str.to_string());
            }
        }

        // Convert body to Zval
        let body_zval = body.into_zval(false).unwrap_or_else(|_| Zval::null());

        Self {
            status_code,
            version: version_str,
            headers: headers_map,
            body: body_zval,
        }
    }
}
