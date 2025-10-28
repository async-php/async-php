/// HTTP-specific IO implementations
/// This module provides concrete implementations of IO interfaces for HTTP

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::io::Reader;
use std::sync::Arc;
use tokio::sync::Mutex;
use bytes::BytesMut;

#[php_class]
#[derive(Clone)]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpBody")]
pub struct HttpBody {
    inner: Arc<Mutex<HttpBodyInner>>,
}

#[derive(Clone)]
pub struct HttpBodyInner {
    data: BytesMut,
    eof: bool,
}

impl HttpBodyInner {
    pub fn new() -> Self {
        Self {
            data: BytesMut::new(),
            eof: false,
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            data: BytesMut::from(s.as_bytes()),
            eof: true,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            data: BytesMut::from(bytes.as_slice()),
            eof: true,
        }
    }
}

impl Reader for HttpBody {
    fn read(&mut self, length: i64
    ) -> PhpResult<Option<String>> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        let fut = async move {
            let mut inner = inner.lock().await;

            if inner.data.is_empty() {
                if inner.eof {
                    return Ok(None);
                } else {
                    return Ok(Some(String::new()));
                }
            }

            let len = std::cmp::min(length as usize, inner.data.len());
            let bytes = inner.data.split_to(len);
            Ok(Some(String::from_utf8_lossy(&bytes).to_string()))
        };

        rt.block_on(fut)
    }
}

#[php_impl]
impl HttpBody {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HttpBodyInner::new())),
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HttpBodyInner::from_string(s))),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HttpBodyInner::from_bytes(bytes))),
        }
    }

    pub fn as_reader(&mut self
    ) -> PhpResult<Zval> {
        // Clone self and create new object wrapping it
        Ok(Zval::new())
    }

    pub fn write(&mut self, data: String) -> PhpResult<()> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        let result = rt.block_on(async move {
            let mut inner = inner.lock().await;
            inner.data.extend_from_slice(data.as_bytes());
            Ok(())
        });
        result
    }

    pub fn close(&mut self) -> PhpResult<bool> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        let result = rt.block_on(async move {
            let mut inner = inner.lock().await;
            inner.eof = true;
            Ok(true)
        });
        result
    }

    pub fn length(&mut self) -> PhpResult<i64> {
        let rt = tokio::runtime::Handle::current();
        let inner = self.inner.clone();
        let result = rt.block_on(async move {
            let inner = inner.lock().await;
            Ok(inner.data.len() as i64)
        });
        result
    }
}
