use std::collections::HashMap;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::http::types::StatusCodes;
use crate::io::get_read_closer_ce;

/// HTTP Response structure
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponse")]
pub struct HttpResponse {
    /// HTTP status code
    status_code: i32,
    /// Reason phrase (e.g., "OK" for status 200)
    reason_phrase: String,
    /// HTTP version
    version: String,
    /// Response headers
    headers: HashMap<String, String>,
    /// Response body as an IO ReadCloser (allows streaming)
    body: Zval,
}

// SAFETY: Safe because runtime is single-threaded
unsafe impl Send for HttpResponse {}
unsafe impl Sync for HttpResponse {}

#[php_impl]
impl HttpResponse {
    /// Create a new HTTP response
    #[php(constructor)]
    pub fn __construct(status_code: i32) -> Self {
        let reason = StatusCodes::get_default_reason_phrase(status_code);
        Self {
            status_code,
            reason_phrase: reason,
            version: "1.1".to_string(),
            headers: HashMap::new(),
            body: Zval::null(),
        }
    }

    /// Create a new response with a body
    pub fn create(status_code: i32, body: &Zval) -> PhpResult<Self> {
        let interface_ce = get_read_closer_ce();

        let object = body.object();
        if object.is_none() || !object.unwrap().instance_of(interface_ce) {
            return Err(PhpException::default("Body must implement ReadCloser".into()));
        }

        let reason = StatusCodes::get_default_reason_phrase(status_code);
        Ok(Self {
            status_code,
            reason_phrase: reason,
            version: "1.1".to_string(),
            headers: HashMap::new(),
            body: body.shallow_clone(),
        })
    }

    /// Get the status code
    pub fn get_status_code(&self) -> i32 {
        self.status_code
    }

    /// Set the status code
    pub fn set_status_code(&mut self, status_code: i32) {
        self.status_code = status_code;
        self.reason_phrase = StatusCodes::get_default_reason_phrase(status_code);
    }

    /// Get the reason phrase
    pub fn get_reason_phrase(&self) -> String {
        self.reason_phrase.clone()
    }

    /// Set a custom reason phrase
    pub fn set_reason_phrase(&mut self, reason_phrase: String) {
        self.reason_phrase = reason_phrase;
    }

    /// Get the HTTP version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Set the HTTP version
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Get all headers
    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    /// Get a specific header
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

    /// Get the response body
    pub fn get_body(&self) -> Zval {
        self.body.shallow_clone()
    }

    /// Set the response body
    pub fn set_body(&mut self, body: &Zval) -> PhpResult<()> {
        self.body = body.shallow_clone();
        Ok(())
    }

    /// Clone the response
    pub fn clone(&self) -> Self {
        Self {
            status_code: self.status_code,
            reason_phrase: self.reason_phrase.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            body: Zval::null(),
        }
    }

    /// Check if the response is successful (2xx)
    pub fn is_success(&self) -> bool {
        StatusCodes::is_success(self.status_code)
    }

    /// Check if the response is a redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        StatusCodes::is_redirect(self.status_code)
    }

    /// Check if the response is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        StatusCodes::is_client_error(self.status_code)
    }

    /// Check if the response is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        StatusCodes::is_server_error(self.status_code)
    }

    /// Check if the response is an error (4xx or 5xx)
    pub fn is_error(&self) -> bool {
        StatusCodes::is_error(self.status_code)
    }
}
