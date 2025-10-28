/// HTTP Response implementation with IO support

use std::collections::HashMap;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::http::body::HttpsBody;

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponse")]
pub struct HttpResponse {
    #[php(prop)]
    status: i32,

    #[php(prop)]
    version: String,

    #[php(prop)]
    headers: HashMap<String, String>,

    body: Option<HttpsBody>,

    // Related request ID for tracking (removed to avoid circular dependency)

    // Whether response is from HTTP/2 server push
    server_pushed: bool,
}

impl HttpResponse {
    pub fn new(status: i32) -> Self {
        let mut headers = HashMap::new();
        headers.insert("server".to_string(), "async-php/1.0".to_string());

        Self {
            status,
            version: "1.1".to_string(),
            headers,
            body: None,
            server_pushed: false,
        }
    }

    pub fn ok() -> Self {
        Self::new(200)
    }

    pub fn not_found() -> Self {
        let mut resp = Self::new(404);
        resp.set_body(HttpsBody::from_string("404 Not Found".to_string()));
        resp
    }

    pub fn server_error() -> Self {
        let mut resp = Self::new(500);
        resp.set_body(HttpsBody::from_string("500 Internal Server Error".to_string()));
        resp
    }
}

#[php_impl]
impl HttpResponse {
    pub fn __construct(status: i32) -> Self {
        Self::new(status)
    }

    pub fn with_status(&mut self, status: i32
    ) -> &mut Self {
        self.status = status;
        self
    }

    pub fn set_status(&mut self, status: i32) -> &mut Self {
        self.status = status;
        self
    }

    pub fn with_version(&mut self, version: String
    ) -> &mut Self {
        self.version = version;
        self
    }

    pub fn with_header(&mut self, name: String, value: String
    ) -> &mut Self {
        self.set_header(name, value);
        self
    }

    pub fn set_header(&mut self, name: String, value: String) -> &mut Self {
        self.headers.insert(name.to_lowercase(), value);
        self
    }

    pub fn remove_header(&mut self, name: &str) -> &mut Self {
        self.headers.remove(&name.to_lowercase());
        self
    }

    pub fn get_header(&self, name: String) -> Option<String> {
        self.headers.get(&name.to_lowercase()).cloned()
    }

    pub fn has_header(&self, name: String) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    pub fn content_type(&self) -> Option<String> {
        self.get_header("content-type".to_string())
    }

    pub fn set_content_type(&mut self, content_type: String) -> &mut Self {
        self.set_header("content-type".to_string(), content_type)
    }

    pub fn content_length(&self) -> Option<i64> {
        self.get_header("content-length".to_string())
            .and_then(|len| len.parse().ok())
    }

    pub fn set_content_length(&mut self, length: i64) -> &mut Self {
        self.set_header("content-length".to_string(), length.to_string())
    }

    pub fn with_body(&mut self, body: HttpsBody
    ) -> &mut Self {
        self.body = Some(body);

        if self.content_length().is_none() {
            if let Ok(len) = body.length() {
                self.set_content_length(len);
            }
        }

        self
    }

    pub fn set_body(&mut self, body: HttpsBody) -> &mut Self {
        self.body = Some(body);

        if self.content_length().is_none() {
            if let Ok(len) = body.length() {
                self.set_content_length(len);
            }
        }

        self
    }

    pub fn is_redirect(&self) -> bool {
        matches!(self.status, 301..=399)
    }

    pub fn redirect(&mut self, location: String) -> &mut Self {
        self.status = 302;
        self.set_header("location".to_string(), location);
        self
    }

    pub fn is_success(&self) -> bool {
        matches!(self.status, 200..=299)
    }

    pub fn is_client_error(&self) -> bool {
        matches!(self.status, 400..=499)
    }

    pub fn is_server_error(&self) -> bool {
        matches!(self.status, 500..=599)
    }

    pub fn is_error(&self) -> bool {
        self.status >= 400
    }

    pub fn error_message(&self) -> Option<String> {
        match self.status {
            400 => Some("Bad Request".to_string()),
            401 => Some("Unauthorized".to_string()),
            403 => Some("Forbidden".to_string()),
            404 => Some("Not Found".to_string()),
            500 => Some("Internal Server Error".to_string()),
            502 => Some("Bad Gateway".to_string()),
            503 => Some("Service Unavailable".to_string()),
            _ => None,
        }
    }

    pub fn get_body(&self) -> Option<&HttpsBody> {
        self.body.as_ref()
    }

    pub fn take_body(&mut self) -> Option<HttpsBody> {
        self.body.take()
    }

    pub fn body_string(&self) -> String {
        String::new()
    }

    // Store request info for tracking (simplified)
    pub fn associate_request(&mut self
    ) -> &mut Self {
        // Track request association here
        self
    }

    pub fn version_string(&self) -> String {
        format!("HTTP/{}", self.version)
    }

    pub fn is_server_push_enabled(&self) -> bool {
        self.server_pushed
    }

    pub fn enable_server_push(&mut self) -> &mut Self {
        self.server_pushed = true;
        self
    }

    pub fn to_array(&self) -> HashMap<String, Zval> {
        let mut result = HashMap::new();

        result.insert("status".to_string(), (self.status as i64).into_zval(false).unwrap());
        result.insert("version".to_string(), self.version.clone().into_zval(false).unwrap());
        result.insert("headers".to_string(), self.headers.clone().into_zval(false).unwrap());

        if self.server_pushed {
            result.insert("server_pushed".to_string(), true.into_zval(false).unwrap());
        }

        result
    }

    pub fn from_array(data: HashMap<String, Zval>) -> PhpResult<Self> {
        let status = data.get("status")
            .and_then(|z| z.long())
            .unwrap_or(200) as i32;

        let version = data.get("version")
            .and_then(|z| z.string())
            .unwrap_or("1.1")
            .to_string();

        let mut resp = Self::new(status);
        resp.version = version;

        if let Some(headers_zval) = data.get("headers") {
            if let Ok(headers_ht) = headers_zval.array() {
                for (k, v) in headers_ht {
                    if let Some(key) = k.string() {
                        if let Some(value) = v.string() {
                            resp.set_header(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        Ok(resp)
    }
}

/// Standard HTTP status codes
#[allow(non_snake_case, dead_code)]
impl HttpResponse {
    pub fn Continue() -> Self { Self::new(100) }
    pub fn SwitchingProtocols() -> Self { Self::new(101) }

    pub fn OK() -> Self { Self::new(200) }
    pub fn Created() -> Self { Self::new(201) }
    pub fn Accepted() -> Self { Self::new(202) }

    pub fn MultipleChoices() -> Self { Self::new(300) }
    pub fn MovedPermanently() -> Self { Self::new(301) }
    pub fn Found() -> Self { Self::new(302) }
    pub fn SeeOther() -> Self { Self::new(303) }
    pub fn NotModified() -> Self { Self::new(304) }
    pub fn TemporaryRedirect() -> Self { Self::new(307) }
    pub fn PermanentRedirect() -> Self { Self::new(308) }

    pub fn BadRequest() -> Self { Self::new(400) }
    pub fn Unauthorized() -> Self { Self::new(401) }
    pub fn Forbidden() -> Self { Self::new(403) }
    pub fn NotFound() -> Self { Self::new(404) }
    pub fn MethodNotAllowed() -> Self { Self::new(405) }
    pub fn RequestTimeout() -> Self { Self::new(408) }
    pub fn Conflict() -> Self { Self::new(409) }
    pub fn Gone() -> Self { Self::new(410) }
    pub fn LengthRequired() -> Self { Self::new(411) }
    pub fn PayloadTooLarge() -> Self { Self::new(413) }
    pub fn URITooLong() -> Self { Self::new(414) }
    pub fn UnsupportedMediaType() -> Self { Self::new(415) }
    pub fn RangeNotSatisfiable() -> Self { Self::new(416) }
    pub fn UnprocessableEntity() -> Self { Self::new(422) }
    pub fn TooManyRequests() -> Self { Self::new(429) }

    pub fn InternalServerError() -> Self { Self::new(500) }
    pub fn NotImplemented() -> Self { Self::new(501) }
    pub fn BadGateway() -> Self { Self::new(502) }
    pub fn ServiceUnavailable() -> Self { Self::new(503) }
    pub fn GatewayTimeout() -> Self { Self::new(504) }
    pub fn HTTPVersionNotSupported() -> Self { Self::new(505) }
}