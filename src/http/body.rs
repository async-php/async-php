/// Shared HTTP body type built on top of the Async PHP IO ReadCloser
/// The body can be backed by a user-provided ReadCloser (for streaming)
/// or an internal in-memory buffer (for simple string bodies).

use ext_php_rs::exception::PhpException;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

use crate::io::PhpInterfaceReadCloser;

/// Internal representation of the body
#[derive(Debug)]
enum BodyInner {
    /// External PHP object implementing Async\Kernel\IO\ReadCloser
    External(Zval),
    /// Simple in-memory buffer for small/static bodies
    Buffer {
        data: Vec<u8>,
        cursor: usize,
        closed: bool,
    },
}

impl Clone for BodyInner {
    fn clone(&self) -> Self {
        match self {
            BodyInner::External(zv) => BodyInner::External(zv.shallow_clone()),
            BodyInner::Buffer {
                data,
                cursor,
                closed,
            } => BodyInner::Buffer {
                data: data.clone(),
                cursor: *cursor,
                closed: *closed,
            },
        }
    }
}

/// HTTP body wrapper exposed to PHP
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpBody")]
#[derive(Clone)]
pub struct HttpBody {
    inner: BodyInner,
}

impl HttpBody {
    fn empty_buffer() -> Self {
        Self {
            inner: BodyInner::Buffer {
                data: Vec::new(),
                cursor: 0,
                closed: false,
            },
        }
    }

    fn from_readcloser_zval(zv: Zval) -> PhpResult<Self> {
        // Validate that the provided object implements ReadCloser
        if PhpInterfaceReadCloser::from_zval(&zv).is_none() {
            return Err(PhpException::default(
                "Body must implement Async\\Kernel\\IO\\ReadCloser".into(),
            ));
        }

        Ok(Self {
            inner: BodyInner::External(zv),
        })
    }

    fn as_readcloser_mut(&mut self) -> PhpResult<PhpInterfaceReadCloser> {
        match &mut self.inner {
            BodyInner::External(zv) => {
                PhpInterfaceReadCloser::from_zval_mut(zv).ok_or_else(|| {
                    PhpException::default(
                        "Body must implement Async\\Kernel\\IO\\ReadCloser".into(),
                    )
                })
            }
            BodyInner::Buffer { .. } => Err(PhpException::default(
                "Body is backed by in-memory buffer, not an external ReadCloser".into(),
            )),
        }
    }
}

#[php_impl]
impl HttpBody {
    /// Create a new HTTP body. If an object is provided it must implement
    /// Async\Kernel\IO\ReadCloser, otherwise an empty in-memory buffer is used.
    #[php(constructor)]
    pub fn __construct(inner: Option<Zval>) -> PhpResult<Self> {
        match inner {
            Some(zv) => Self::from_readcloser_zval(zv),
            None => Ok(Self::empty_buffer()),
        }
    }

    /// Create an in-memory body from string data
    #[php]
    pub fn from_string(data: String) -> Self {
        Self {
            inner: BodyInner::Buffer {
                data: data.into_bytes(),
                cursor: 0,
                closed: false,
            },
        }
    }

    /// Read from the body. Returns None on EOF.
    pub fn read(&mut self, length: i64) -> PhpResult<Option<String>> {
        if length <= 0 {
            return Ok(Some(String::new()));
        }

        match &mut self.inner {
            BodyInner::Buffer {
                data,
                cursor,
                closed,
            } => {
                if *closed && *cursor >= data.len() {
                    return Ok(None);
                }

                let end = (*cursor + length as usize).min(data.len());
                let chunk = data[*cursor..end].to_vec();
                *cursor = end;
                if *cursor >= data.len() {
                    *closed = true;
                }

                Ok(Some(String::from_utf8_lossy(&chunk).to_string()))
            }
            BodyInner::External(_) => {
                let mut rc = self.as_readcloser_mut()?;
                rc.read(length)
            }
        }
    }

    /// Close the body
    pub fn close(&mut self) -> PhpResult<bool> {
        match &mut self.inner {
            BodyInner::Buffer { closed, .. } => {
                *closed = true;
                Ok(true)
            }
            BodyInner::External(_) => {
                let mut rc = self.as_readcloser_mut()?;
                rc.close()
            }
        }
    }

    /// Append data into the in-memory buffer. This is only available when the
    /// body is not backed by an external ReadCloser.
    pub fn append(&mut self, data: String) -> PhpResult<()> {
        match &mut self.inner {
            BodyInner::Buffer { data: buf, closed, .. } => {
                if *closed {
                    return Err(PhpException::default("Cannot append to a closed body".into()));
                }
                buf.extend_from_slice(data.as_bytes());
                Ok(())
            }
            BodyInner::External(_) => Err(PhpException::default(
                "Body append is only available for in-memory buffers".into(),
            )),
        }
    }

    /// Reset cursor for buffered bodies (useful for reusing responses)
    pub fn rewind(&mut self) {
        if let BodyInner::Buffer { cursor, closed, .. } = &mut self.inner {
            *cursor = 0;
            *closed = false;
        }
    }

    /// Get the underlying PHP object (when provided)
    pub fn get_inner(&self) -> Option<Zval> {
        match &self.inner {
            BodyInner::External(zv) => Some(zv.shallow_clone()),
            BodyInner::Buffer { .. } => None,
        }
    }
}

impl HttpBody {
    /// Helper to build from Option<Zval>
    pub fn from_optional(inner: Option<Zval>) -> PhpResult<Option<Self>> {
        inner
            .map(Self::from_readcloser_zval)
            .transpose()
    }
}
