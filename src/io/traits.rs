/// Combined trait definitions for async IO operations
///
/// These traits combine multiple tokio IO traits into single trait objects
/// for easier handling and type safety.

use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};

// ==================== Combined Trait Definitions ====================

/// Trait combining AsyncRead and AsyncWrite
///
/// Note: Unpin + Send bounds are applied when creating trait objects,
/// not in the trait definition itself, following tokio's pattern.
pub trait AsyncReadWrite: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite> AsyncReadWrite for T {}

/// Trait combining AsyncRead and AsyncSeek
pub trait AsyncReadSeek: AsyncRead + AsyncSeek {}
impl<T: AsyncRead + AsyncSeek> AsyncReadSeek for T {}

/// Trait combining AsyncWrite and AsyncSeek
pub trait AsyncWriteSeek: AsyncWrite + AsyncSeek {}
impl<T: AsyncWrite + AsyncSeek> AsyncWriteSeek for T {}

/// Trait combining AsyncRead, AsyncWrite, and AsyncSeek
pub trait AsyncReadWriteSeek: AsyncRead + AsyncWrite + AsyncSeek {}
impl<T: AsyncRead + AsyncWrite + AsyncSeek> AsyncReadWriteSeek for T {}
