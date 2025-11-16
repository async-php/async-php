/// HTTP Request Builder - wraps reqwest::RequestBuilder
///
/// Provides fluent API for building HTTP requests with full reqwest capabilities:
/// - Headers manipulation
/// - Query parameters
/// - Request body (text, JSON, form data, multipart, stream)
/// - Basic/Bearer authentication
/// - Timeout override

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;

use reqwest::{Method, RequestBuilder};
use std::collections::HashMap;
use std::time::Duration;

use crate::future::RustFuture;
use crate::util::Shared;

/// HTTP Request Builder
///
/// This class wraps reqwest::RequestBuilder and provides a fluent API
/// for building HTTP requests. All methods return self for method chaining.
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpRequest")]
pub struct HttpRequest {
    builder: Shared<Option<RequestBuilder>>,
}

unsafe impl Send for HttpRequest {}
unsafe impl Sync for HttpRequest {}

impl HttpRequest {
    /// Internal constructor
    pub(crate) fn new(client: std::sync::Arc<reqwest::Client>, method: &str, url: String) -> Self {
        let method = method.parse::<Method>().unwrap_or(Method::GET);
        let builder = client.request(method, url);
        Self {
            builder: Shared::new(Some(builder)),
        }
    }
}

#[php_impl]
impl HttpRequest {
    /// Set a header
    #[php]
    pub fn header(&mut self, name: String, value: String) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.header(name, value));
        }
        Ok(())
    }

    /// Set multiple headers
    #[php]
    pub fn headers(&mut self, headers: HashMap<String, String>) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(mut builder) = builder_ref.take() {
            for (key, value) in headers {
                builder = builder.header(key, value);
            }
            *builder_ref = Some(builder);
        }
        Ok(())
    }

    /// Add a query parameter
    #[php]
    pub fn query(&mut self, name: String, value: String) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.query(&[(name, value)]));
        }
        Ok(())
    }

    /// Add multiple query parameters
    #[php]
    pub fn query_params(&mut self, params: HashMap<String, String>) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.query(&params));
        }
        Ok(())
    }

    /// Set Basic Authentication
    #[php]
    pub fn basic_auth(&mut self, username: String, password: Option<String>) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.basic_auth(username, password));
        }
        Ok(())
    }

    /// Set Bearer token authentication
    #[php]
    pub fn bearer_auth(&mut self, token: String) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.bearer_auth(token));
        }
        Ok(())
    }

    /// Set request timeout (overrides client timeout)
    #[php]
    pub fn timeout(&mut self, seconds: f64) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.timeout(Duration::from_secs_f64(seconds)));
        }
        Ok(())
    }

    /// Set text body with optional content type
    #[php]
    pub fn body_text(&mut self, text: String, content_type: Option<String>) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(mut builder) = builder_ref.take() {
            if let Some(ct) = content_type {
                builder = builder.header("Content-Type", ct);
            }
            *builder_ref = Some(builder.body(text));
        }
        Ok(())
    }

    /// Set JSON body (automatically sets Content-Type: application/json)
    #[php]
    pub fn body_json(&mut self, json: String) -> PhpResult<()> {
        let json_value: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.json(&json_value));
        }
        Ok(())
    }

    /// Set form body (application/x-www-form-urlencoded)
    #[php]
    pub fn body_form(&mut self, form: HashMap<String, String>) -> PhpResult<()> {
        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.form(&form));
        }
        Ok(())
    }

    /// Set binary/raw body
    #[php]
    pub fn body_bytes(&mut self, data: &Zval) -> PhpResult<()> {
        let bytes = if let Some(bin) = data.binary() {
            bin.to_vec()
        } else if let Some(s) = data.str() {
            s.as_bytes().to_vec()
        } else {
            return Err(PhpException::default("Body must be string or binary".to_string()));
        };

        let builder_ref = self.builder.get_mut();
        if let Some(builder) = builder_ref.take() {
            *builder_ref = Some(builder.body(bytes));
        }
        Ok(())
    }

    /// Set streaming body from AsyncReader
    ///
    /// The reader must implement the AsyncReader interface from io.rs
    ///
    /// NOTE: This method uses fast-path optimization when the reader is a Rust-native type
    /// (AsyncFileHandle, AsyncTcpStream, etc.), avoiding PHP FFI overhead.
    #[php]
    pub fn body_stream(&mut self, reader: &Zval, _content_length: Option<i64>) -> PhpResult<()> {
        use tokio_util::io::ReaderStream;
        use reqwest::Body;
        use crate::io::try_extract_native_reader;

        // Try fast path: extract native Rust IO type directly
        if let Some(async_reader) = try_extract_native_reader(reader) {
            // Fast path: use native Rust async reader without going through PHP FFI
            // The async_reader is already an AsyncReader PHP class wrapping the Shared trait object
            use crate::io::SharedAsyncRead;
            let wrapper = SharedAsyncRead::new(async_reader.get_inner());
            let stream = ReaderStream::new(wrapper);
            let body = Body::wrap_stream(stream);

            let builder_ref = self.builder.get_mut();
            if let Some(builder) = builder_ref.take() {
                *builder_ref = Some(builder.body(body));
            }
            return Ok(());
        }

        // Slow path: PHP custom implementation (TODO: implement using channel + fiber)
        // For now, return an error
        Err(PhpException::default("Streaming body is only supported for native Rust IO types (AsyncFileHandle, AsyncTcpStream, etc.)".to_string()))
    }

    /// Send the request and return a Future that resolves to HttpResponse
    #[php]
    pub fn send(&mut self) -> RustFuture {
        let builder_opt = self.builder.get_mut().take();

        RustFuture::new(async move {
            let builder = builder_opt
                .ok_or_else(|| "Request already sent or invalidated".to_string())?;

            let response = builder
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            let response_obj = crate::http::HttpResponse::new_internal(response);
            response_obj
                .into_zval(false)
                .map_err(|e| format!("Failed to convert response: {:?}", e))
        })
    }
}
