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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::future::RustFuture;

/// HTTP Request Builder
///
/// This class wraps reqwest::RequestBuilder and provides a fluent API
/// for building HTTP requests. All methods return self for method chaining.
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpRequest")]
pub struct HttpRequest {
    builder: Arc<Mutex<Option<RequestBuilder>>>,
}

unsafe impl Send for HttpRequest {}
unsafe impl Sync for HttpRequest {}

impl HttpRequest {
    /// Internal constructor
    pub(crate) fn new(client: Arc<reqwest::Client>, method: &str, url: String) -> Self {
        let method = method.parse::<Method>().unwrap_or(Method::GET);
        let builder = client.request(method, url);
        Self {
            builder: Arc::new(Mutex::new(Some(builder))),
        }
    }
}

#[php_impl]
impl HttpRequest {
    /// Set a header
    #[php]
    pub fn header(&mut self, name: String, value: String) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.header(name, value));
        }
        Ok(())
    }

    /// Set multiple headers
    #[php]
    pub fn headers(&mut self, headers: HashMap<String, String>) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(mut builder) = guard.take() {
            for (key, value) in headers {
                builder = builder.header(key, value);
            }
            *guard = Some(builder);
        }
        Ok(())
    }

    /// Add a query parameter
    #[php]
    pub fn query(&mut self, name: String, value: String) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.query(&[(name, value)]));
        }
        Ok(())
    }

    /// Add multiple query parameters
    #[php]
    pub fn query_params(&mut self, params: HashMap<String, String>) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.query(&params));
        }
        Ok(())
    }

    /// Set Basic Authentication
    #[php]
    pub fn basic_auth(&mut self, username: String, password: Option<String>) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.basic_auth(username, password));
        }
        Ok(())
    }

    /// Set Bearer token authentication
    #[php]
    pub fn bearer_auth(&mut self, token: String) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.bearer_auth(token));
        }
        Ok(())
    }

    /// Set request timeout (overrides client timeout)
    #[php]
    pub fn timeout(&mut self, seconds: f64) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.timeout(Duration::from_secs_f64(seconds)));
        }
        Ok(())
    }

    /// Set text body with optional content type
    #[php]
    pub fn body_text(&mut self, text: String, content_type: Option<String>) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(mut builder) = guard.take() {
            if let Some(ct) = content_type {
                builder = builder.header("Content-Type", ct);
            }
            *guard = Some(builder.body(text));
        }
        Ok(())
    }

    /// Set JSON body (automatically sets Content-Type: application/json)
    #[php]
    pub fn body_json(&mut self, json: String) -> PhpResult<()> {
        let json_value: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.json(&json_value));
        }
        Ok(())
    }

    /// Set form body (application/x-www-form-urlencoded)
    #[php]
    pub fn body_form(&mut self, form: HashMap<String, String>) -> PhpResult<()> {
        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.form(&form));
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

        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.body(bytes));
        }
        Ok(())
    }

    /// Set streaming body from AsyncReader
    ///
    /// The reader must implement the AsyncReader interface from io.rs
    #[php]
    pub fn body_stream(&mut self, reader: &Zval, _content_length: Option<i64>) -> PhpResult<()> {
        use tokio_util::io::ReaderStream;
        use reqwest::Body;

        // Wrap PHP reader in AsyncRead adapter
        let php_reader = PhpBodyReader::new(reader.shallow_clone());
        let stream = ReaderStream::new(php_reader);

        // Note: reqwest::Body::wrap_stream doesn't support with_length
        // Content-Length must be set via header if needed
        let body = Body::wrap_stream(stream);

        let mut guard = self.builder.lock().unwrap();
        if let Some(builder) = guard.take() {
            *guard = Some(builder.body(body));
        }
        Ok(())
    }

    /// Send the request and return a Future that resolves to HttpResponse
    #[php]
    pub fn send(&mut self) -> RustFuture {
        let builder_opt = {
            let mut guard = self.builder.lock().unwrap();
            guard.take()
        };

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

// ==================== Internal Adapters ====================

/// Adapter to read from PHP AsyncReader as tokio::io::AsyncRead
struct PhpBodyReader {
    reader: Zval,
    chunk_size: usize,
}

impl PhpBodyReader {
    fn new(reader: Zval) -> Self {
        Self {
            reader,
            chunk_size: 8192,
        }
    }
}

impl tokio::io::AsyncRead for PhpBodyReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let to_read = std::cmp::min(buf.remaining(), this.chunk_size) as i64;

        let mut length_zval = Zval::new();
        length_zval.set_long(to_read);

        let data_zval = this
            .reader
            .try_call_method("read", vec![&length_zval])
            .map_err(|e| std::io::Error::other(format!("PHP read failed: {:?}", e)))?;

        if data_zval.is_null() {
            return std::task::Poll::Ready(Ok(()));
        }

        if let Some(bytes) = data_zval.binary() {
            buf.put_slice(&bytes);
        } else if let Some(s) = data_zval.str() {
            buf.put_slice(s.as_bytes());
        } else {
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Read did not return string or binary",
            )));
        }

        std::task::Poll::Ready(Ok(()))
    }
}

unsafe impl Send for PhpBodyReader {}
unsafe impl Sync for PhpBodyReader {}
