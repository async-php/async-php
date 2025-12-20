/// Combined wrapper types for async IO operations
///
/// These wrappers combine multiple IO traits (Read+Write, Read+Seek, etc.)

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use tokio::io::AsyncBufRead;

use crate::future::RustFuture;
use crate::util::Shared;

use super::traits::{AsyncReadSeek, AsyncReadWrite, AsyncReadWriteSeek, AsyncWriteSeek};
use super::cast::{cast_io, AsyncIO};
use super::wrappers::{AsyncReader, AsyncSeeker, AsyncWriter};

// ==================== AsyncReadWriter ====================

/// AsyncReadWriter wraps Shared<Box<dyn AsyncReadWrite + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReadWriter")]
pub struct AsyncReadWriter {
    inner: Shared<Box<dyn AsyncReadWrite + Unpin + Send>>,
}

impl AsyncReadWriter {
    pub fn from_shared(shared: Shared<Box<dyn AsyncReadWrite + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncReadWrite + Unpin + Send + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncReadWrite + Unpin + Send>> {
        self.inner.clone()
    }
}

impl AsyncIO for AsyncReadWriter {
    fn as_read_writer(&self) -> PhpResult<AsyncReadWriter> {
        Ok(AsyncReadWriter::from_shared(self.inner.clone()))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        Ok(AsyncReader::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        Ok(AsyncWriter::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }
}

#[php_impl]
impl AsyncReadWriter {
    /// Read up to length bytes
    pub fn read(&mut self, length: i64) -> RustFuture {
        self.inner.read_impl(length)
    }

    /// Write data
    pub fn write(&mut self, data: String) -> RustFuture {
        self.inner.write_impl(data)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> RustFuture {
        self.inner.flush_impl()
    }

    /// Cast this IO into a specific Kernel IO wrapper by bitflags
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        cast_io(self, ty)
    }
}

// ==================== AsyncReadSeeker ====================

/// AsyncReadSeeker wraps Shared<Box<dyn AsyncReadSeek + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReadSeeker")]
pub struct AsyncReadSeeker {
    inner: Shared<Box<dyn AsyncReadSeek + Unpin + Send>>,
}

impl AsyncReadSeeker {
    pub fn from_shared(shared: Shared<Box<dyn AsyncReadSeek + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncReadSeek + Unpin + Send + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncReadSeek + Unpin + Send>> {
        self.inner.clone()
    }
}

impl AsyncIO for AsyncReadSeeker {
    fn as_read_seeker(&self) -> PhpResult<AsyncReadSeeker> {
        Ok(AsyncReadSeeker::from_shared(self.inner.clone()))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        Ok(AsyncReader::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        Ok(AsyncSeeker::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }
}

#[php_impl]
impl AsyncReadSeeker {
    /// Read up to length bytes
    pub fn read(&mut self, length: i64) -> RustFuture {
        self.inner.read_impl(length)
    }

    /// Seek to a position
    pub fn seek(&mut self, offset: i64, whence: i64) -> RustFuture {
        self.inner.seek_impl(offset, whence)
    }

    /// Cast this IO into a specific Kernel IO wrapper by bitflags
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        cast_io(self, ty)
    }
}

// ==================== AsyncWriteSeeker ====================

/// AsyncWriteSeeker wraps Shared<Box<dyn AsyncWriteSeek + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncWriteSeeker")]
pub struct AsyncWriteSeeker {
    inner: Shared<Box<dyn AsyncWriteSeek + Unpin + Send>>,
}

impl AsyncWriteSeeker {
    pub fn from_shared(shared: Shared<Box<dyn AsyncWriteSeek + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncWriteSeek + Unpin + Send + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncWriteSeek + Unpin + Send>> {
        self.inner.clone()
    }
}

impl AsyncIO for AsyncWriteSeeker {
    fn as_write_seeker(&self) -> PhpResult<AsyncWriteSeeker> {
        Ok(AsyncWriteSeeker::from_shared(self.inner.clone()))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        Ok(AsyncWriter::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        Ok(AsyncSeeker::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }
}

#[php_impl]
impl AsyncWriteSeeker {
    /// Write data
    pub fn write(&mut self, data: String) -> RustFuture {
        self.inner.write_impl(data)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> RustFuture {
        self.inner.flush_impl()
    }

    /// Seek to a position
    pub fn seek(&mut self, offset: i64, whence: i64) -> RustFuture {
        self.inner.seek_impl(offset, whence)
    }

    /// Cast this IO into a specific Kernel IO wrapper by bitflags
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        cast_io(self, ty)
    }
}

// ==================== AsyncReadWriteSeeker ====================

/// AsyncReadWriteSeeker wraps Shared<Box<dyn AsyncReadWriteSeek + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReadWriteSeeker")]
pub struct AsyncReadWriteSeeker {
    inner: Shared<Box<dyn AsyncReadWriteSeek + Unpin + Send>>,
}

impl AsyncReadWriteSeeker {
    pub fn from_shared(shared: Shared<Box<dyn AsyncReadWriteSeek + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncReadWriteSeek + Unpin + Send + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncReadWriteSeek + Unpin + Send>> {
        self.inner.clone()
    }
}

impl AsyncIO for AsyncReadWriteSeeker {
    fn as_read_write_seeker(&self) -> PhpResult<AsyncReadWriteSeeker> {
        Ok(AsyncReadWriteSeeker::from_shared(self.inner.clone()))
    }

    fn as_read_writer(&self) -> PhpResult<AsyncReadWriter> {
        Ok(AsyncReadWriter::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_read_seeker(&self) -> PhpResult<AsyncReadSeeker> {
        Ok(AsyncReadSeeker::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_write_seeker(&self) -> PhpResult<AsyncWriteSeeker> {
        Ok(AsyncWriteSeeker::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        Ok(AsyncReader::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        Ok(AsyncWriter::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        Ok(AsyncSeeker::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }
}

#[php_impl]
impl AsyncReadWriteSeeker {
    /// Read up to length bytes
    pub fn read(&mut self, length: i64) -> RustFuture {
        self.inner.read_impl(length)
    }

    /// Write data
    pub fn write(&mut self, data: String) -> RustFuture {
        self.inner.write_impl(data)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> RustFuture {
        self.inner.flush_impl()
    }

    /// Seek to a position
    pub fn seek(&mut self, offset: i64, whence: i64) -> RustFuture {
        self.inner.seek_impl(offset, whence)
    }

    /// Cast this IO into a specific Kernel IO wrapper by bitflags
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        cast_io(self, ty)
    }
}

// ==================== AsyncBufReader ====================

/// AsyncBufReader wraps Shared<Box<dyn AsyncBufRead + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncBufReader")]
pub struct AsyncBufReader {
    inner: Shared<Box<dyn AsyncBufRead + Unpin + Send>>,
}

impl AsyncBufReader {
    /// Create AsyncBufReader from a Shared-wrapped reader
    /// This allows multiple AsyncBufReader instances to share the same underlying reader
    pub fn from_shared(shared: Shared<Box<dyn AsyncBufRead + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    /// Create AsyncBufReader from a tokio AsyncBufRead type
    /// This wraps the reader in a new Shared container
    pub fn new<B: AsyncBufRead + Unpin + Send + 'static>(reader: B) -> Self {
        Self {
            inner: Shared::new(Box::new(reader)),
        }
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn get_inner(&self) -> Shared<Box<dyn AsyncBufRead + Unpin + Send>> {
        self.inner.clone()
    }
}

impl AsyncIO for AsyncBufReader {
    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Ok(AsyncBufReader::from_shared(self.inner.clone()))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        Ok(AsyncReader::from_shared(Shared::new(Box::new(self.inner.clone()))))
    }
}

#[php_impl]
impl AsyncBufReader {
    /// Read a line
    pub fn read_line(&mut self) -> RustFuture {
        self.inner.read_line_impl()
    }

    /// Read until delimiter
    pub fn read_until(&mut self, delim: u8) -> RustFuture {
        self.inner.read_until_impl(delim)
    }

    /// Cast this IO into a specific Kernel IO wrapper by bitflags
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        cast_io(self, ty)
    }
}
