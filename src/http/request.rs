use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::collections::HashMap;

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
    headers: HashMap<String, String>,
    /// Request body as an IO ReadCloser (allows streaming)
    body: Zval,
}

// SAFETY: Safe because runtime is single-threaded
unsafe impl Send for HttpRequest {}
unsafe impl Sync for HttpRequest {}

#[php_impl]
impl HttpRequest {
    /// Create a new HTTP request
    #[php(constructor)]
    pub fn __construct(method: String, uri: String) -> Self {
        Self {
            method,
            uri,
            version: "1.1".to_string(),
            headers: HashMap::new(),
            body: Zval::null(),
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
    pub fn get_headers(&self) -> HashMap<String, String> {
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
        self.headers.insert(name, value);
    }

    /// Remove a header
    pub fn remove_header(&mut self, name: String) {
        self.headers.remove(&name);
    }

    /// Get the request body as an IO reader
    pub fn get_body(&self) -> Zval {
        self.body.shallow_clone()
    }

    /// Set the request body using an IO ReadCloser
    /// body参数应该是实现了Reader和Closer接口的对象
    pub fn set_body(&mut self, body: &Zval) -> PhpResult<()> {
        self.body = body.shallow_clone();
        Ok(())
    }

    /// Clone the request
    pub fn clone(&self) -> Self {
        Self {
            method: self.method.clone(),
            uri: self.uri.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            body: Zval::null(),
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
            body: Zval::null(),
            headers: HashMap::new(),
        })
    }
}
