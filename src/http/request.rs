/// HTTP Request implementation

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

use crate::http::HttpBody;

/// HTTP Request structure - rust侧仅提供最简内核实现
/// 应用层负责在PHP中实现具体的body逻辑
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpRequest")]
pub struct HttpRequest {
    /// The HTTP method (GET, POST, PUT, DELETE, etc.)
    method: String,
    /// The request URI
    uri: String,
    /// HTTP version (1.0, 1.1, 2.0, etc.)
    version: String,
    /// Request headers as key-value pairs
    headers: Vec<(String, String)>,
    /// Request body as an IO ReadCloser (allows streaming)
    body: Option<HttpBody>,
}

#[php_impl]
impl HttpRequest {
    /// Create a new HTTP request
    #[php(constructor)]
    pub fn __construct(method: String, uri: String) -> Self {
        Self {
            method,
            uri,
            version: "1.1".to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Get the HTTP method
    pub fn get_method(&self) -> String {
        self.method.clone()
    }

    /// Set the HTTP method
    pub fn set_method(&mut self, method: String) {
        self.method = method;
    }

    /// Get the request URI
    pub fn get_uri(&self) -> String {
        self.uri.clone()
    }

    /// Set the request URI
    pub fn set_uri(&mut self, uri: String) {
        self.uri = uri;
    }

    /// Get the HTTP version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Set the HTTP version
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Get all headers as an associative array
    pub fn get_headers(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }

    /// Get a specific header value
    pub fn get_header(&self, name: String) -> Option<String> {
        self.headers.iter()
            .find(|(key, _)| key.to_lowercase() == name.to_lowercase())
            .map(|(_, value)| value.clone())
    }

    /// Set a header (replaces if exists)
    pub fn set_header(&mut self, name: String, value: String) {
        // Remove existing header with same name (case-insensitive)
        self.headers.retain(|(key, _)| key.to_lowercase() != name.to_lowercase());
        self.headers.push((name, value));
    }

    /// Add a header (allows multiple headers with same name)
    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push((name, value));
    }

    /// Remove a header
    pub fn remove_header(&mut self, name: String) {
        self.headers.retain(|(key, _)| key.to_lowercase() != name.to_lowercase());
    }

    /// Get the request body as an IO reader
    pub fn get_body(&self) -> Option<HttpBody> {
        self.body.clone()
    }

    /// Set the request body using an IO ReadCloser
    /// body参数应该是实现了Reader和Closer接口的对象
    pub fn set_body(&mut self, body: Option<Zval>) -> PhpResult<()> {
        self.body = HttpBody::from_optional(body)?;
        Ok(())
    }

    /// Clone the request
    pub fn clone(&self) -> Self {
        Self {
            method: self.method.clone(),
            uri: self.uri.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
        }
    }

    /// Create from server globals (utility method for server-side)
    /// This is a simplified implementation - server would populate this
    pub fn from_server_globals() -> PhpResult<HttpRequest> {
        // In a real implementation, this would parse $_SERVER, $_GET, $_POST, etc.
        Ok(Self {
            method: "GET".to_string(),
            uri: "/".to_string(),
            version: "1.1".to_string(),
            headers: Vec::new(),
            body: None,
        })
    }
}
