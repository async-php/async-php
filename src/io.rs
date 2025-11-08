/// Core IO utilities for async-php
/// This module provides Rust types that wrap tokio IO traits

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use crate::util::Shared;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeek, AsyncBufRead, AsyncReadExt, AsyncWriteExt, AsyncSeekExt, AsyncBufReadExt};
use std::io::SeekFrom;

/// AsyncReader wraps Shared<Box<dyn AsyncRead + Unpin>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReader")]
pub struct AsyncReader {
    inner: Shared<Box<dyn AsyncRead + Unpin>>,
}

impl AsyncReader {
    /// Create AsyncReader from a tokio AsyncRead type
    pub fn from_reader<R: AsyncRead + Unpin + 'static>(reader: R) -> Self {
        Self {
            inner: Shared::new(Box::new(reader)),
        }
    }

    /// Get a clone of the inner Shared for direct tokio usage
    pub fn into_tokio(self) -> Shared<Box<dyn AsyncRead + Unpin>> {
        self.inner
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn as_tokio(&self) -> Shared<Box<dyn AsyncRead + Unpin>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncReader {
    /// Read up to length bytes
    pub fn read(&mut self, length: i64) -> RustFuture {
        let inner = self.inner.clone();

        let future = async move {
            let mut buf = vec![0u8; length as usize];
            let n = inner.get_mut().read(&mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            buf.truncate(n);
            // Use set_binary to preserve all bytes (binary-safe)
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok(z)
        };

        RustFuture::new(future)
    }
}

/// AsyncWriter wraps Shared<Box<dyn AsyncWrite + Unpin>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncWriter")]
pub struct AsyncWriter {
    inner: Shared<Box<dyn AsyncWrite + Unpin>>,
}

impl AsyncWriter {
    /// Create AsyncWriter from a tokio AsyncWrite type
    pub fn from_writer<W: AsyncWrite + Unpin + 'static>(writer: W) -> Self {
        Self {
            inner: Shared::new(Box::new(writer)),
        }
    }

    /// Get a clone of the inner Shared for direct tokio usage
    pub fn into_tokio(self) -> Shared<Box<dyn AsyncWrite + Unpin>> {
        self.inner
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn as_tokio(&self) -> Shared<Box<dyn AsyncWrite + Unpin>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncWriter {
    /// Write data
    pub fn write(&mut self, data: String) -> RustFuture {
        let inner = self.inner.clone();

        let future = async move {
            let bytes = data.as_bytes();
            inner.get_mut().write_all(bytes).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(bytes.len() as i64);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> RustFuture {
        let inner = self.inner.clone();

        let future = async move {
            inner.get_mut().flush().await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_bool(true);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }
}

/// AsyncSeeker wraps Shared<Box<dyn AsyncSeek + Unpin>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncSeeker")]
pub struct AsyncSeeker {
    inner: Shared<Box<dyn AsyncSeek + Unpin>>,
}

impl AsyncSeeker {
    /// Create AsyncSeeker from a tokio AsyncSeek type
    pub fn from_seeker<S: AsyncSeek + Unpin + 'static>(seeker: S) -> Self {
        Self {
            inner: Shared::new(Box::new(seeker)),
        }
    }

    /// Get a clone of the inner Shared for direct tokio usage
    pub fn into_tokio(self) -> Shared<Box<dyn AsyncSeek + Unpin>> {
        self.inner
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn as_tokio(&self) -> Shared<Box<dyn AsyncSeek + Unpin>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncSeeker {
    /// Seek to a position
    pub fn seek(&mut self, offset: i64, whence: i64) -> RustFuture {
        let inner = self.inner.clone();

        let future = async move {
            let seek_from = match whence {
                0 => SeekFrom::Start(offset as u64),
                1 => SeekFrom::Current(offset),
                2 => SeekFrom::End(offset),
                _ => SeekFrom::Start(offset as u64),
            };

            let new_pos = inner.get_mut().seek(seek_from).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(new_pos as i64);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }
}

/// AsyncBufReader wraps Shared<Box<dyn AsyncBufRead + Unpin>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncBufReader")]
pub struct AsyncBufReader {
    inner: Shared<Box<dyn AsyncBufRead + Unpin>>,
}

impl AsyncBufReader {
    /// Create AsyncBufReader from a tokio AsyncBufRead type
    pub fn from_buf_reader<B: AsyncBufRead + Unpin + 'static>(reader: B) -> Self {
        Self {
            inner: Shared::new(Box::new(reader)),
        }
    }

    /// Get a clone of the inner Shared for direct tokio usage
    pub fn into_tokio(self) -> Shared<Box<dyn AsyncBufRead + Unpin>> {
        self.inner
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn as_tokio(&self) -> Shared<Box<dyn AsyncBufRead + Unpin>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncBufReader {
    /// Read a line
    pub fn read_line(&mut self) -> RustFuture {
        let inner = self.inner.clone();

        let future = async move {
            let mut line = String::new();
            let n = inner.get_mut().read_line(&mut line).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            let mut z = Zval::new();
            z.set_string(&line, false)
                .map_err(|e| format!("set_string error: {:?}", e))?;
            Ok(z)
        };

        RustFuture::new(future)
    }

    /// Read until delimiter
    pub fn read_until(&mut self, delim: u8) -> RustFuture {
        let inner = self.inner.clone();

        let future = async move {
            let mut buf = Vec::new();
            let n = inner.get_mut().read_until(delim, &mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            // Use set_binary to preserve all bytes (binary-safe)
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok(z)
        };

        RustFuture::new(future)
    }
}
