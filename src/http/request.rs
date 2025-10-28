/// HTTP Request implementation with IO support

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use std::collections::HashMap;
use crate::http::body::HttpsBody;

#[php_class]
#[derive(Clone)]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpRequest")]
pub struct HttpRequest {
    #[php(prop)]
    pub method: String,
    #[php(prop)]
    pub uri: String,
    #[php(prop)]
    pub version: String,
    #[php(prop)]
    pub headers: HashMap<String, String>,

    pub body: Option<HttpsBody>,

    // Response is set after the request is made through client (will be stored separately)
}

impl HttpRequest {
    pub fn new(method: String, uri: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "async-php-client/1.0".to_string());

        Self {
            method,
            uri,
            version: "1.1".to_string(),
            headers,
            body: None,
        }
    }

    pub fn version_string(&self
    ) -> String {
        format!("HTTP/{}", self.version)
    }
}

#[php_impl]
impl HttpRequest {
    pub fn __construct(method: String, uri: String) -> Self {
        Self::new(method, uri)
    }

    pub fn with_body(&mut self, body: &HttpsBody
    ) -> Self {
        self.body = Some(body.clone());
        self.clone()
    }

    pub fn set_version(&mut self, version: String
    ) -> Self {
        self.version = version;
        self.clone()
    }

    pub fn with_header(&mut self, name: String, value: String
    ) -> Self {
        self.headers.insert(
            name.to_lowercase(),
            value,
        );
        self.clone()
    }

    pub fn without_header(&mut self, name: &str
    ) -> Self {
        self.headers.remove(&name.to_lowercase());
        self.clone()
    }

    pub fn get_header(&self, name: String
    ) -> Option<String> {
        self.headers.get(&name.to_lowercase()).cloned()
    }

    pub fn has_header(&self, name: String
    ) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    pub fn content_type(&self
    ) -> Option<String> {
        self.get_header("content-type".to_string())
    }

    pub fn set_content_type(&mut self, content_type: String
    ) -> Self {
        self.with_header("content-type".to_string(), content_type)
    }

    pub fn set_content_length(&mut self, content_length: i64
    ) -> Self {
        self.with_header(
            "content-length".to_string(),
            content_length.to_string(),
        )
    }

    pub fn host(&self
    ) -> Option<String> {
        self.get_header("host".to_string())
    }

    pub fn is_json(&self
    ) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    pub fn is_form_data(&self
    ) -> bool {
        self.content_type()
            .map(|ct| ct.contains("multipart/form-data") || ct.contains("application/x-www-form-urlencoded"))
            .unwrap_or(false)
    }

    pub fn get_body(&self
    ) -> Option<HttpsBody> {
        self.body.clone()
    }

    pub fn take_body(&mut self
    ) -> Option<HttpsBody> {
        self.body.take()
    }

    pub fn body_string(&self
    ) -> String {
        self.body
            .as_ref()
            .and_then(|b| {
                let mut b_clone = b.clone();
                b_clone.as_string().ok()
            })
            .unwrap_or_default()
    }


    pub fn clone_without_body(&self
    ) -> Self {
        Self {
            method: self.method.clone(),
            uri: self.uri.clone(),
            version: self.version.clone(),
            headers: self.headers.clone(),
            body: None,
        }
    }

    pub fn is_http3_request(&self
    ) -> bool {
        #[cfg(feature = "http3")]
        {
            false
        }
        #[cfg(not(feature = "http3"))]
        {
            false
        }
    }

    pub fn to_array(&self
    ) -> HashMap<String, Zval> {
        let mut result = HashMap::new();

        result.insert("method".to_string(), self.method.clone().into_zval(false).unwrap());
        result.insert("uri".to_string(), self.uri.clone().into_zval(false).unwrap());
        result.insert("version".to_string(), self.version.clone().into_zval(false).unwrap());
        result.insert("headers".to_string(), self.headers.clone().into_zval(false).unwrap());

        if self.is_http3_request() {
            result.insert("http3".to_string(), true.into_zval(false).unwrap());
        }

        result
    }

    pub fn from_array(data: &Zval
    ) -> PhpResult<Self> {
        let arr = data.array().ok_or("Expected array")?;
        
        let method = arr.get("method")
            .and_then(|z| z.string())
            .unwrap_or("GET".to_string())
            .to_string();

        let uri = arr.get("uri")
            .and_then(|z| z.string())
            .unwrap_or("/".to_string())
            .to_string();

        let mut req = Self::new(method, uri);

        if let Some(headers_zval) = arr.get("headers") {
            if let Some(headers_ht) = headers_zval.array() {
                for (k, v) in headers_ht {
                    // Handle ArrayKey manually
                    use ext_php_rs::types::ArrayKey;
                    let key_str = match k {
                        ArrayKey::Long(i) => Some(i.to_string()),
                        ArrayKey::Str(s) => Some(s.to_string()),
                        _ => None,
                    };
                    
                    if let Some(key) = key_str {
                        if let Some(value) = v.string() {
                            req.with_header(key, value.to_string());
                        }
                    }
                }
            }
        }

        if let Some(version_zval) = arr.get("version") {
            if let Some(version) = version_zval.string() {
                req.set_version(version.to_string());
            }
        }

        Ok(req)
    }

    pub fn is_server_push_enabled(&self
    ) -> bool {
        self.get_header("server-push".to_string())
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    pub fn enable_server_push(&mut self
    ) -> Self {
        self.with_header("server-push".to_string(), "true".to_string())
    }

    pub fn disable_server_push(&mut self
    ) -> Self {
        self.without_header("server-push")
    }

    #[cfg(feature = "http3")]
    pub fn enable_http3(&mut self
    ) -> Self {
        self.version = "3".to_string();
        self.clone()
    }

    #[cfg(not(feature = "http3"))]
    #[cfg(feature = "http3")]
    pub fn enable_http3(&mut self
    ) -> Self {
        self.version = "3".to_string();
        self.clone()
    }

    #[cfg(not(feature = "http3"))]
    pub fn enable_http3(&mut self
    ) -> Self {
        self.clone()
    }
}
