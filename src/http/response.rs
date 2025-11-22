use bytes::Bytes;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use http::Response;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty};
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;

use crate::future::RustFuture;
use crate::io::AsyncReader;
use crate::util::Shared;

#[derive(Clone, Copy, Debug)]
struct BodyConsumed;

fn empty_body() -> BoxBody<Bytes, Box<dyn Error + Send + Sync>> {
    Empty::<Bytes>::new()
        .map_err(|err: Infallible| match err {})
        .boxed()
}

/// HTTP Response (protocol-level)
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponse")]
pub struct HttpResponse {
    pub(crate) inner: Shared<Response<BoxBody<Bytes, Box<dyn Error + Send + Sync>>>>,
}

unsafe impl Send for HttpResponse {}
unsafe impl Sync for HttpResponse {}

impl HttpResponse {
    pub(crate) fn new_internal(response: Response<BoxBody<Bytes, Box<dyn Error + Send + Sync>>>) -> Self {
        Self {
            inner: Shared::new(response),
        }
    }

    pub(crate) fn from_reqwest(response: reqwest::Response) -> Self {
        use futures::TryStreamExt;
        use http_body::Frame;
        use http_body_util::StreamBody;
        use sync_wrapper::SyncStream;

        let status = response.status();
        let version = response.version();
        let headers = response.headers().clone();

        let stream = response
            .bytes_stream()
            .map_ok(Frame::data)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>);
        let body = StreamBody::new(SyncStream::new(stream)).boxed();

        let mut http_response = Response::new(body);
        *http_response.status_mut() = status;
        *http_response.version_mut() = version;
        *http_response.headers_mut() = headers;

        Self::new_internal(http_response)
    }

    pub(crate) fn take_body(&mut self) -> Result<BoxBody<Bytes, Box<dyn Error + Send + Sync>>, String> {
        let resp = self.inner.get_mut();
        if resp.extensions().get::<BodyConsumed>().is_some() {
            return Err("Response body already consumed".to_string());
        }
        resp.extensions_mut().insert(BodyConsumed);
        Ok(std::mem::replace(resp.body_mut(), empty_body()))
    }
}

#[php_impl]
impl HttpResponse {
    #[php]
    pub fn status(&self) -> u16 {
        self.inner.get_ref().status().as_u16()
    }

    #[php]
    pub fn is_success(&self) -> bool {
        self.inner.get_ref().status().is_success()
    }

    #[php]
    pub fn is_client_error(&self) -> bool {
        self.inner.get_ref().status().is_client_error()
    }

    #[php]
    pub fn is_server_error(&self) -> bool {
        self.inner.get_ref().status().is_server_error()
    }

    #[php]
    pub fn version(&self) -> String {
        format!("{:?}", self.inner.get_ref().version())
    }

    #[php]
    pub fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for (name, value) in self.inner.get_ref().headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(name.as_str().to_lowercase(), value_str.to_string());
            }
        }
        headers
    }

    #[php]
    pub fn header(&self, name: String) -> Option<String> {
        let name = name.to_lowercase();
        self.inner
            .get_ref()
            .headers()
            .get(name.as_str())
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    #[php]
    pub fn content_type(&self) -> Option<String> {
        self.header("content-type".to_string())
    }

    #[php]
    pub fn content_length(&self) -> Option<i64> {
        self.header("content-length".to_string())
            .and_then(|s| s.parse().ok())
    }

    #[php]
    pub fn text(&mut self) -> RustFuture {
        let body = match self.take_body() {
            Ok(b) => b,
            Err(e) => return RustFuture::new(async move { Err::<Zval, String>(e) }),
        };

        RustFuture::new(async move {
            let collected = body.collect().await.map_err(|e| e.to_string())?;
            let bytes = collected.to_bytes();
            let text = String::from_utf8(bytes.to_vec())
                .map_err(|e| format!("Failed to decode UTF-8: {e}"))?;

            let mut zval = Zval::new();
            zval.set_string(&text, false)
                .map_err(|e| format!("Failed to set string: {:?}", e))?;
            Ok::<Zval, String>(zval)
        })
    }

    #[php]
    pub fn json(&mut self) -> RustFuture {
        let body = match self.take_body() {
            Ok(b) => b,
            Err(e) => return RustFuture::new(async move { Err::<Zval, String>(e) }),
        };

        RustFuture::new(async move {
            let collected = body.collect().await.map_err(|e| e.to_string())?;
            let bytes = collected.to_bytes();
            let json_value: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to parse JSON: {e}"))?;
            let json_str = serde_json::to_string(&json_value)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?;

            let mut zval = Zval::new();
            zval.set_string(&json_str, false)
                .map_err(|e| format!("Failed to set string: {:?}", e))?;
            Ok::<Zval, String>(zval)
        })
    }

    #[php]
    pub fn bytes(&mut self) -> RustFuture {
        let body = match self.take_body() {
            Ok(b) => b,
            Err(e) => return RustFuture::new(async move { Err::<Zval, String>(e) }),
        };

        RustFuture::new(async move {
            let collected = body.collect().await.map_err(|e| e.to_string())?;
            let bytes = collected.to_bytes();

            let mut zval = Zval::new();
            zval.set_binary(bytes.to_vec());
            Ok::<Zval, String>(zval)
        })
    }

    #[php]
    pub fn stream(&mut self) -> PhpResult<AsyncReader> {
        let body = self.take_body()?;
        use std::io;
        use futures::StreamExt;

        let stream = body
            .into_data_stream()
            .map(|result| result.map_err(|e| io::Error::other(e.to_string())));
        let reader = tokio_util::io::StreamReader::new(stream);

        Ok(AsyncReader::new(reader))
    }

    // ==================== Server-side builder methods ====================

    /// Create a new HTTP response (for server-side use)
    #[php(constructor)]
    pub fn __construct() -> Self {
        let response = Response::new(empty_body());
        Self::new_internal(response)
    }

    /// Set the HTTP status code
    #[php]
    pub fn set_status(&mut self, status: u16) -> PhpResult<()> {
        let resp = self.inner.get_mut();
        *resp.status_mut() = http::StatusCode::from_u16(status)
            .map_err(|e| format!("Invalid status code: {e}"))?;
        Ok(())
    }

    /// Set a response header
    #[php]
    pub fn set_header(&mut self, name: String, value: String) -> PhpResult<()> {
        use http::header::{HeaderName, HeaderValue};

        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| format!("Invalid header name: {e}"))?;
        let value = HeaderValue::from_str(&value)
            .map_err(|e| format!("Invalid header value: {e}"))?;

        let resp = self.inner.get_mut();
        resp.headers_mut().insert(name, value);
        Ok(())
    }

    /// Set the response body from a string
    #[php]
    pub fn set_body(&mut self, body: String) -> PhpResult<()> {
        use bytes::Bytes;
        use http_body_util::Full;

        let full_body = Full::new(Bytes::from(body))
            .map_err(|err: Infallible| match err {})
            .boxed();

        let resp = self.inner.get_mut();
        *resp.body_mut() = full_body;
        Ok(())
    }
}
