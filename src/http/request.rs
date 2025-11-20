use bytes::Bytes;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use http::header::{HeaderName, HeaderValue};
use http::{Request, Uri};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full, StreamBody};
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::time::Duration;
use url::Url;

use crate::util::Shared;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestTimeout(pub Duration);

#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestSent;

pub(crate) fn empty_body() -> BoxBody<Bytes, Box<dyn Error + Send>> {
    Empty::<Bytes>::new()
        .map_err(|err: Infallible| match err {})
        .boxed()
}

fn full_body(bytes: Bytes) -> BoxBody<Bytes, Box<dyn Error + Send>> {
    Full::new(bytes)
        .map_err(|err: Infallible| match err {})
        .boxed()
}

fn parse_uri(uri: &str) -> Result<Uri, String> {
    uri.parse::<Uri>()
        .map_err(|e| format!("Invalid URI: {e}"))
}

fn set_header(request: &mut Request<BoxBody<Bytes, Box<dyn Error + Send>>>, name: String, value: String) -> Result<(), String> {
    let name = HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| format!("Invalid header name: {e}"))?;
    let value = HeaderValue::from_str(&value)
        .map_err(|e| format!("Invalid header value: {e}"))?;
    request.headers_mut().insert(name, value);
    Ok(())
}

/// HTTP Request (protocol-level, client-agnostic)
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpRequest")]
pub struct HttpRequest {
    pub(crate) inner: Shared<Request<BoxBody<Bytes, Box<dyn Error + Send>>>>,
}

unsafe impl Send for HttpRequest {}
unsafe impl Sync for HttpRequest {}

impl HttpRequest {
    pub(crate) fn new(method: &str, url: String) -> PhpResult<Self> {
        let request = Request::builder()
            .method(method)
            .uri(parse_uri(&url)?)
            .body(empty_body())
            .map_err(|e| format!("Failed to build request: {e}"))?;
        Ok(Self {
            inner: Shared::new(request),
        })
    }
}

#[php_impl]
impl HttpRequest {
    #[php(constructor)]
    pub fn __construct(method: String, url: String) -> PhpResult<Self> {
        Self::new(&method, url)
    }

    #[php]
    pub fn header(&mut self, name: String, value: String) -> PhpResult<()> {
        set_header(self.inner.get_mut(), name, value)?;
        Ok(())
    }

    #[php]
    pub fn headers(&mut self, headers: HashMap<String, String>) -> PhpResult<()> {
        let request = self.inner.get_mut();
        for (name, value) in headers {
            set_header(request, name, value)?;
        }
        Ok(())
    }

    #[php]
    pub fn query(&mut self, name: String, value: String) -> PhpResult<()> {
        let request = self.inner.get_mut();
        let mut url = Url::parse(&request.uri().to_string())
            .map_err(|e| format!("Invalid URL: {e}"))?;
        url.query_pairs_mut().append_pair(&name, &value);
        *request.uri_mut() = parse_uri(url.as_str())?;
        Ok(())
    }

    #[php]
    pub fn query_params(&mut self, params: HashMap<String, String>) -> PhpResult<()> {
        let request = self.inner.get_mut();
        let mut url = Url::parse(&request.uri().to_string())
            .map_err(|e| format!("Invalid URL: {e}"))?;
        {
            let mut qp = url.query_pairs_mut();
            for (k, v) in params {
                qp.append_pair(&k, &v);
            }
        }
        *request.uri_mut() = parse_uri(url.as_str())?;
        Ok(())
    }

    #[php]
    pub fn timeout(&mut self, seconds: f64) -> PhpResult<()> {
        let secs = if seconds.is_sign_negative() { 0.0 } else { seconds };
        let req = self.inner.get_mut();
        req.extensions_mut()
            .insert(RequestTimeout(Duration::from_secs_f64(secs)));
        Ok(())
    }

    #[php]
    pub fn body_text(&mut self, text: String, content_type: Option<String>) -> PhpResult<()> {
        let req = self.inner.get_mut();
        if let Some(ct) = content_type {
            set_header(req, "Content-Type".to_string(), ct)?;
        }
        *req.body_mut() = full_body(Bytes::from(text));
        Ok(())
    }

    #[php]
    pub fn body_json(&mut self, json: String) -> PhpResult<()> {
        let _: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| format!("Invalid JSON: {e}"))?;
        let req = self.inner.get_mut();
        set_header(req, "Content-Type".to_string(), "application/json".to_string())?;
        *req.body_mut() = full_body(Bytes::from(json));
        Ok(())
    }

    #[php]
    pub fn body_form(&mut self, form: HashMap<String, String>) -> PhpResult<()> {
        let req = self.inner.get_mut();
        set_header(req, "Content-Type".to_string(), "application/x-www-form-urlencoded".to_string())?;
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in form {
            serializer.append_pair(&k, &v);
        }
        let encoded = serializer.finish();
        *req.body_mut() = full_body(Bytes::from(encoded));
        Ok(())
    }

    #[php]
    pub fn body_bytes(&mut self, data: &Zval) -> PhpResult<()> {
        let bytes = if let Some(bin) = data.binary() {
            Bytes::from(bin.to_vec())
        } else if let Some(s) = data.str() {
            Bytes::from(s.as_bytes().to_vec())
        } else {
            return Err(PhpException::default("Body must be string or binary".to_string()));
        };

        let req = self.inner.get_mut();
        *req.body_mut() = full_body(bytes);
        Ok(())
    }

    #[php]
    pub fn body_stream(&mut self, reader: &crate::io::AsyncReader) -> PhpResult<()> {
        use futures::TryStreamExt;
        use http_body::Frame;
        use sync_wrapper::SyncStream;
        use tokio_util::io::ReaderStream;

        let stream = ReaderStream::new(reader.get_inner())
            .map_ok(Frame::data)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>);
        let body = StreamBody::new(SyncStream::new(stream)).boxed();

        let req = self.inner.get_mut();
        *req.body_mut() = body;
        Ok(())
    }
}
