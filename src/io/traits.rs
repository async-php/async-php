/// Combined trait definitions for async IO operations
///
/// These traits combine multiple tokio IO traits into single trait objects
/// for easier handling and type safety.

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

// ==================== Combined Trait Definitions ====================

/// Trait combining AsyncRead and AsyncWrite
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncReadWrite for T {}

/// Trait combining AsyncRead and AsyncSeek
pub trait AsyncReadSeek: AsyncRead + AsyncSeek + Unpin + Send {}
impl<T: AsyncRead + AsyncSeek + Unpin + Send> AsyncReadSeek for T {}

/// Trait combining AsyncWrite and AsyncSeek
pub trait AsyncWriteSeek: AsyncWrite + AsyncSeek + Unpin + Send {}
impl<T: AsyncWrite + AsyncSeek + Unpin + Send> AsyncWriteSeek for T {}

/// Trait combining AsyncRead, AsyncWrite, and AsyncSeek
pub trait AsyncReadWriteSeek: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send> AsyncReadWriteSeek for T {}
