//! In-memory byte buffers for async I/O
//!
//! This module provides Rust equivalents of Go's bytes/strings package,
//! allowing efficient in-memory I/O operations without disk or network overhead.
//!
//! # Examples
//!
//! ```php
//! // Create a reader from string
//! $reader = BytesReader::from_string("Hello, World!");
//! $request->body_stream($reader->as_reader());
//!
//! // Create a reader from binary data
//! $data = file_get_contents('data.bin');
//! $reader = BytesReader::from_bytes($data);
//! $request->body_stream($reader->as_reader());
//!
//! // Create a writer to collect data
//! $writer = BytesWriter::new();
//! // ... write data ...
//! $contents = $writer->to_bytes();
//! ```

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;

use std::io::{Cursor, Read, Write, Seek, SeekFrom};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

use crate::io::cast_io;
use crate::util::Shared;

// ==================== BytesCursor (newtype wrapper) ====================

/// Newtype wrapper around Cursor<Vec<u8>> to implement async traits
///
/// This wrapper is needed due to Rust's orphan rules - we can't implement
/// external traits on external types.
struct BytesCursor(Cursor<Vec<u8>>);

impl BytesCursor {
    fn new(data: Vec<u8>) -> Self {
        Self(Cursor::new(data))
    }

    fn position(&self) -> u64 {
        self.0.position()
    }

    fn get_ref(&self) -> &Vec<u8> {
        self.0.get_ref()
    }

    fn get_mut(&mut self) -> &mut Vec<u8> {
        self.0.get_mut()
    }

    fn set_position(&mut self, pos: u64) {
        self.0.set_position(pos)
    }
}

/// Implement AsyncRead for BytesCursor
///
/// Since Cursor operations are in-memory and non-blocking, we can safely
/// wrap synchronous Read operations in Poll::Ready.
impl AsyncRead for BytesCursor {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let amt = self.0.read(buf.initialize_unfilled())?;
        buf.advance(amt);
        Poll::Ready(Ok(()))
    }
}

/// Implement AsyncWrite for BytesCursor
impl AsyncWrite for BytesCursor {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Poll::Ready(self.0.write(buf))
    }

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(self.0.flush())
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.poll_flush(cx)
    }
}

/// Implement AsyncSeek for BytesCursor
impl AsyncSeek for BytesCursor {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> std::io::Result<()> {
        self.0.seek(position)?;
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<u64>> {
        Poll::Ready(Ok(self.position()))
    }
}

// ==================== BytesReader ====================

/// In-memory bytes reader (equivalent to Go's bytes.Reader)
///
/// This allows creating an AsyncReader from a byte slice or string,
/// enabling efficient in-memory I/O without disk/network overhead.
///
/// # PHP Example
/// ```php
/// $reader = BytesReader::from_string("Hello, World!");
/// $data = $reader->as_reader()->read(5); // "Hello"
/// ```
#[php_class]
#[php(name = "Async\\Kernel\\IO\\BytesReader")]
pub struct BytesReader {
    inner: Shared<BytesCursor>,
}

unsafe impl Send for BytesReader {}
unsafe impl Sync for BytesReader {}

#[php_impl]
impl BytesReader {
    /// Create a new BytesReader from a byte string (binary-safe)
    #[php]
    pub fn from_bytes(data: &Zval) -> PhpResult<Self> {
        let bytes = data.binary().ok_or("Expected binary string")?;
        Ok(Self {
            inner: Shared::new(BytesCursor::new(bytes.to_vec())),
        })
    }

    /// Create a new BytesReader from a UTF-8 string
    #[php]
    pub fn from_string(s: String) -> Self {
        Self {
            inner: Shared::new(BytesCursor::new(s.into_bytes())),
        }
    }

    /// Get the total length of the underlying data
    #[php]
    pub fn len(&self) -> i64 {
        self.inner.get_ref().get_ref().len() as i64
    }

    /// Get current position in the buffer
    #[php]
    pub fn position(&self) -> i64 {
        self.inner.get_ref().position() as i64
    }

    /// Get remaining bytes from current position
    #[php]
    pub fn remaining(&self) -> i64 {
        let cursor = self.inner.get_ref();
        let len = cursor.get_ref().len() as u64;
        let pos = cursor.position();
        (len.saturating_sub(pos)) as i64
    }

    /// Reset position to the beginning
    #[php]
    pub fn reset(&mut self) {
        self.inner.get_mut().set_position(0);
    }

    /// Cast this buffer into a specific Kernel IO wrapper by bitflags.
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let io: Shared<Box<dyn crate::io::AsyncReadSeek>> =
            Shared::new(Box::new(self.inner.clone()));
        cast_io(&io, ty)
    }
}

// ==================== BytesWriter ====================

/// In-memory bytes writer (equivalent to Go's bytes.Buffer for writing)
///
/// This allows collecting written data in memory and retrieving it as bytes.
///
/// # PHP Example
/// ```php
/// $writer = BytesWriter::new();
/// $writer->as_writer()->write("Hello");
/// $writer->as_writer()->write(" World");
/// $contents = $writer->to_bytes(); // "Hello World"
/// ```
#[php_class]
#[php(name = "Async\\Kernel\\IO\\BytesWriter")]
pub struct BytesWriter {
    inner: Shared<BytesCursor>,
}

unsafe impl Send for BytesWriter {}
unsafe impl Sync for BytesWriter {}

#[php_impl]
impl BytesWriter {
    /// Create a new empty BytesWriter
    #[php]
    pub fn new() -> Self {
        Self {
            inner: Shared::new(BytesCursor::new(Vec::new())),
        }
    }

    /// Create a BytesWriter with pre-allocated capacity
    #[php]
    pub fn with_capacity(capacity: i64) -> Self {
        let cap = capacity.max(0) as usize;
        Self {
            inner: Shared::new(BytesCursor::new(Vec::with_capacity(cap))),
        }
    }

    /// Get the current length of written data
    #[php]
    pub fn len(&self) -> i64 {
        self.inner.get_ref().get_ref().len() as i64
    }

    /// Get current write position
    #[php]
    pub fn position(&self) -> i64 {
        self.inner.get_ref().position() as i64
    }

    /// Clear all data (reset to empty)
    #[php]
    pub fn clear(&mut self) {
        self.inner.get_mut().get_mut().clear();
        self.inner.get_mut().set_position(0);
    }

    /// Get the written data as a byte string
    #[php]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.get_ref().get_ref().clone()
    }

    /// Get the written data as a UTF-8 string
    ///
    /// Returns None if the data is not valid UTF-8
    #[php]
    pub fn to_string(&self) -> Option<String> {
        String::from_utf8(self.to_bytes()).ok()
    }

    /// Cast this buffer into a specific Kernel IO wrapper by bitflags.
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let io: Shared<Box<dyn crate::io::AsyncReadWriteSeek>> =
            Shared::new(Box::new(self.inner.clone()));
        cast_io(&io, ty)
    }
}
