/// Generic implementations for Shared<Box<T>> IO operations
///
/// These implementations provide read/write/seek/bufread methods for Shared wrappers,
/// and also implement tokio IO traits directly on Shared<T>

use ext_php_rs::types::Zval;
use std::io::{Result as IoResult, SeekFrom};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};

use crate::future::RustFuture;
use crate::util::Shared;

// ==================== Generic Methods for Shared<Box<T>> to Reduce Duplication ====================

/// Implement read method for all Shared<Box<T>> where T implements AsyncRead
impl<T: ?Sized + 'static> Shared<Box<T>>
where
    T: AsyncRead + Unpin + Send,
{
    pub fn read_impl(&self, length: i64) -> RustFuture {
        let inner = self.clone();
        let future = async move {
            let length = usize::try_from(length)
                .map_err(|_| "read length must be non-negative".to_string())?;
            if length == 0 {
                let mut z = Zval::new();
                z.set_binary(Vec::<u8>::new());
                return Ok::<Zval, String>(z);
            }
            let mut buf = vec![0u8; length];
            let n = inner.get_mut().read(&mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            buf.truncate(n);
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok(z)
        };
        RustFuture::new(future)
    }
}

/// Implement write and flush methods for all Shared<Box<T>> where T implements AsyncWrite
impl<T: ?Sized + 'static> Shared<Box<T>>
where
    T: AsyncWrite + Unpin + Send,
{
    pub fn write_impl(&self, data: String) -> RustFuture {
        let inner = self.clone();
        let future = async move {
            let bytes = data.as_bytes();
            inner.get_mut().write_all(bytes).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(bytes.len() as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    pub fn flush_impl(&self) -> RustFuture {
        let inner = self.clone();
        let future = async move {
            inner.get_mut().flush().await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_bool(true);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }
}

/// Implement seek method for all Shared<Box<T>> where T implements AsyncSeek
impl<T: ?Sized + 'static> Shared<Box<T>>
where
    T: AsyncSeek + Unpin + Send,
{
    pub fn seek_impl(&self, offset: i64, whence: i64) -> RustFuture {
        let inner = self.clone();
        let future = async move {
            let seek_from = match whence {
                0 => {
                    if offset < 0 {
                        return Err("seek offset must be non-negative for SEEK_SET".to_string());
                    }
                    SeekFrom::Start(offset as u64)
                }
                1 => SeekFrom::Current(offset),
                2 => SeekFrom::End(offset),
                _ => return Err(format!("invalid whence value: {}", whence)),
            };

            let new_pos = inner.get_mut().seek(seek_from).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(new_pos as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }
}

/// Implement buffered read methods for all Shared<Box<T>> where T implements AsyncBufRead
impl<T: ?Sized + 'static> Shared<Box<T>>
where
    T: AsyncBufRead + Unpin + Send,
{
    pub fn read_line_impl(&self) -> RustFuture {
        let inner = self.clone();
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

    pub fn read_until_impl(&self, delim: u8) -> RustFuture {
        let inner = self.clone();
        let future = async move {
            let mut buf = Vec::new();
            let n = inner.get_mut().read_until(delim, &mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            let mut z = Zval::new();
            z.set_binary(buf);
            Ok(z)
        };
        RustFuture::new(future)
    }
}

// ==================== AsyncRead/AsyncWrite/AsyncSeek Implementations for Shared<T> ====================
// Directly implement tokio IO traits for Shared<T> to avoid unnecessary wrapper types

/// Implement AsyncRead for Shared<T> where T: AsyncRead + Unpin
impl<T: AsyncRead + Unpin> AsyncRead for Shared<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(self.get_mut().get_mut()).poll_read(cx, buf)
    }
}

/// Implement AsyncWrite for Shared<T> where T: AsyncWrite + Unpin
impl<T: AsyncWrite + Unpin> AsyncWrite for Shared<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        Pin::new(self.get_mut().get_mut()).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(self.get_mut().get_mut()).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(self.get_mut().get_mut()).poll_shutdown(cx)
    }
}

/// Implement AsyncSeek for Shared<T> where T: AsyncSeek + Unpin
impl<T: AsyncSeek + Unpin> AsyncSeek for Shared<T> {
    fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> IoResult<()> {
        Pin::new(self.get_mut().get_mut()).start_seek(position)
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        Pin::new(self.get_mut().get_mut()).poll_complete(cx)
    }
}
