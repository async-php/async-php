/// HTTP Response implementation

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::http::types::StatusCodes;
use crate::http::HttpBody;

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
    headers: Vec<(String, String)>,
    /// Response body as an IO ReadCloser (allows streaming)
    body: Option<HttpBody>,
}

#[php_impl]
impl HttpResponse {
    /// Create a new HTTP response
    #[php(constructor)]
    pub fn __construct(status_code: i32) -> Self {
        let reason = Self::get_default_reason_phrase(status_code);
        Self {
            status_code,
            reason_phrase: reason,
            version: "1.1".to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Create a new response with a body
    pub fn create(status_code: i32, body: Option<Zval>) -> PhpResult<Self> {
        let reason = Self::get_default_reason_phrase(status_code);
        Ok(Self {
            status_code,
            reason_phrase: reason,
            version: "1.1".to_string(),
            headers: Vec::new(),
            body: HttpBody::from_optional(body)?,
        })
    }

    /// Get the status code
    pub fn get_status_code(&self) -> i32 {
        self.status_code
    }

    /// Set the status code
    pub fn set_status_code(&mut self, status_code: i32) {
        self.status_code = status_code;
        self.reason_phrase = Self::get_default_reason_phrase(status_code);
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
    pub fn get_headers(&self) -> Vec<(String, String)> {
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

    /// Get the response body
    pub fn get_body(&self) -> Option<HttpBody> {
        self.body.clone()
    }

    /// Set the response body using an IO ReadCloser
    pub fn set_body(&mut self, body: Option<Zval>) -> PhpResult<()> {
        self.body = HttpBody::from_optional(body)?;
        Ok(())
    }

    /// Set the response body from a static string buffer
    pub fn set_body_string(&mut self, body: String) {
        self.body = Some(HttpBody::from_string(body));
    }

    /// Initialize an empty streaming body (buffer backed)
    pub fn init_stream(&mut self) {
        if self.body.is_none() {
            self.body = Some(HttpBody::from_string(String::new()));
        } else if let Some(ref mut body) = self.body {
            body.rewind();
        }
    }

    /// Write data to the buffered response body (not available for external streams)
    pub fn write(&mut self, data: String) -> PhpResult<i64> {
        if self.body.is_none() {
            self.body = Some(HttpBody::from_string(String::new()));
        }

        if let Some(ref mut body) = self.body {
            body.append(data.clone())?;
            Ok(data.len() as i64)
        } else {
            Ok(0)
        }
    }

    /// Close/finish the response body
    pub fn end(&mut self) -> PhpResult<()> {
        if let Some(ref mut body) = self.body {
            body.close()?;
        }
        Ok(())
    }

    /// Clone the response
    pub fn clone(&self) -> Self {
        Self {
            status_code: self.status_code,
            reason_phrase: self.reason_phrase.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
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

    /// Get default reason phrase for status code
    fn get_default_reason_phrase(status_code: i32) -> String {
        match status_code {
            100 => "Continue".to_string(),
            101 => "Switching Protocols".to_string(),

            200 => "OK".to_string(),
            201 => "Created".to_string(),
            202 => "Accepted".to_string(),
            204 => "No Content".to_string(),

            301 => "Moved Permanently".to_string(),
            302 => "Found".to_string(),
            303 => "See Other".to_string(),
            304 => "Not Modified".to_string(),
            307 => "Temporary Redirect".to_string(),
            308 => "Permanent Redirect".to_string(),

            400 => "Bad Request".to_string(),
            401 => "Unauthorized".to_string(),
            403 => "Forbidden".to_string(),
            404 => "Not Found".to_string(),
            405 => "Method Not Allowed".to_string(),
            406 => "Not Acceptable".to_string(),
            408 => "Request Timeout".to_string(),
            409 => "Conflict".to_string(),
            410 => "Gone".to_string(),
            411 => "Length Required".to_string(),
            413 => "Payload Too Large".to_string(),
            414 => "URI Too Long".to_string(),
            415 => "Unsupported Media Type".to_string(),
            416 => "Range Not Satisfiable".to_string(),
            422 => "Unprocessable Entity".to_string(),
            429 => "Too Many Requests".to_string(),

            500 => "Internal Server Error".to_string(),
            501 => "Not Implemented".to_string(),
            502 => "Bad Gateway".to_string(),
            503 => "Service Unavailable".to_string(),
            504 => "Gateway Timeout".to_string(),

            _ => "Unknown".to_string(),
        }
    }
}
