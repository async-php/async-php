/// Basic wrapper types for async IO operations
///
/// These wrappers provide PHP-accessible classes for AsyncRead, AsyncWrite, and AsyncSeek

use ext_php_rs::prelude::*;
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

use crate::future::RustFuture;
use crate::util::Shared;

// ==================== AsyncReader ====================

/// AsyncReader wraps Shared<Box<dyn AsyncRead + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReader")]
pub struct AsyncReader {
    inner: Shared<Box<dyn AsyncRead + Unpin + Send>>,
}

impl AsyncReader {
    /// Create AsyncReader from a Shared-wrapped reader
    /// This allows multiple AsyncReader instances to share the same underlying reader
    pub fn from_shared(shared: Shared<Box<dyn AsyncRead + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    /// Create AsyncReader from a tokio AsyncRead type
    /// This wraps the reader in a new Shared container
    pub fn new<R: AsyncRead + Unpin + Send + 'static>(reader: R) -> Self {
        Self {
            inner: Shared::new(Box::new(reader)),
        }
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn get_inner(&self) -> Shared<Box<dyn AsyncRead + Unpin + Send>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncReader {
    /// Read up to length bytes
    pub fn read(&mut self, length: i64) -> RustFuture {
        self.inner.read_impl(length)
    }
}

// ==================== AsyncWriter ====================

/// AsyncWriter wraps Shared<Box<dyn AsyncWrite + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncWriter")]
pub struct AsyncWriter {
    inner: Shared<Box<dyn AsyncWrite + Unpin + Send>>,
}

impl AsyncWriter {
    /// Create AsyncWriter from a Shared-wrapped writer
    /// This allows multiple AsyncWriter instances to share the same underlying writer
    pub fn from_shared(shared: Shared<Box<dyn AsyncWrite + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    /// Create AsyncWriter from a tokio AsyncWrite type
    /// This wraps the writer in a new Shared container
    pub fn new<W: AsyncWrite + Unpin + Send + 'static>(writer: W) -> Self {
        Self {
            inner: Shared::new(Box::new(writer)),
        }
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn get_inner(&self) -> Shared<Box<dyn AsyncWrite + Unpin + Send>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncWriter {
    /// Write data
    pub fn write(&mut self, data: String) -> RustFuture {
        self.inner.write_impl(data)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> RustFuture {
        self.inner.flush_impl()
    }
}

// ==================== AsyncSeeker ====================

/// AsyncSeeker wraps Shared<Box<dyn AsyncSeek + Unpin + Send>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncSeeker")]
pub struct AsyncSeeker {
    inner: Shared<Box<dyn AsyncSeek + Unpin + Send>>,
}

impl AsyncSeeker {
    /// Create AsyncSeeker from a Shared-wrapped seeker
    /// This allows multiple AsyncSeeker instances to share the same underlying seeker
    pub fn from_shared(shared: Shared<Box<dyn AsyncSeek + Unpin + Send>>) -> Self {
        Self { inner: shared }
    }

    /// Create AsyncSeeker from a tokio AsyncSeek type
    /// This wraps the seeker in a new Shared container
    pub fn new<S: AsyncSeek + Unpin + Send + 'static>(seeker: S) -> Self {
        Self {
            inner: Shared::new(Box::new(seeker)),
        }
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn get_inner(&self) -> Shared<Box<dyn AsyncSeek + Unpin + Send>> {
        self.inner.clone()
    }
}

#[php_impl]
impl AsyncSeeker {
    /// Seek to a position
    pub fn seek(&mut self, offset: i64, whence: i64) -> RustFuture {
        self.inner.seek_impl(offset, whence)
    }
}
