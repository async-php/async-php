/// Generic PhpIo implementation for async IO operations
/// Provides a universal bridge between PHP objects and Rust tokio traits

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::io::{Error as IoError, ErrorKind, Result as IoResult, SeekFrom};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};
use futures::future::LocalBoxFuture;
use futures::{FutureExt, Future};
use pin_project::{pin_project, pinned_drop};

use super::php_bridge::{PhpIoBridge, PhpIoCallFuture, php_io_call_future};
use super::traits::AsyncReadWriteSeek;
use super::cast::cast_io;
use crate::util::Shared;

// ==================== Helper Functions ====================

/// Helper to process read response from PHP and write to buffer
fn process_read_response(result: Zval, buf: &mut tokio::io::ReadBuf<'_>) -> IoResult<()>
{
    if result.is_null() {
        return Ok(());
    }

    if let Some(bytes) = result.binary() {
        if bytes.len() > buf.remaining() {
            return Err(IoError::new(
                ErrorKind::InvalidData,
                "Read response exceeded requested length",
            ));
        }
        buf.put_slice(&bytes);
    } else if let Some(s) = result.str() {
        let bytes = s.as_bytes();
        if bytes.len() > buf.remaining() {
            return Err(IoError::new(
                ErrorKind::InvalidData,
                "Read response exceeded requested length",
            ));
        }
        buf.put_slice(bytes);
    } else {
        return Err(IoError::new(
            ErrorKind::InvalidData,
            "Invalid read response",
        ));
    }
    Ok(())
}

/// Helper to create write future
fn create_write_future(
    bridge: PhpIoBridge,
    write_buf: &[u8],
) -> LocalBoxFuture<'static, IoResult<usize>> {
    let mut data_zval = Zval::new();
    data_zval.set_binary(write_buf.to_vec());
    async move {
        let result = bridge.call("write", vec![data_zval]).await?;
        if let Some(n) = result.long() {
            Ok(n as usize)
        } else {
            Err(IoError::new(ErrorKind::InvalidData, "Invalid write response"))
        }
    }
    .boxed_local()
}

/// Helper to create flush future
fn create_flush_future(bridge: PhpIoBridge) -> LocalBoxFuture<'static, IoResult<()>> {
    async move {
        bridge.call("flush", vec![]).await?;
        Ok(())
    }
    .boxed_local()
}

/// Helper to create shutdown future
fn create_shutdown_future(bridge: PhpIoBridge) -> LocalBoxFuture<'static, IoResult<()>> {
    async move {
        bridge.call("close", vec![]).await?;
        Ok(())
    }
    .boxed_local()
}

/// Helper to create seek future
fn create_seek_call_future(
    bridge: PhpIoBridge,
    pos: SeekFrom,
) -> PhpIoCallFuture {
    let (offset, whence) = match pos {
        SeekFrom::Start(pos) => (pos as i64, 0),
        SeekFrom::Current(off) => (off, 1),
        SeekFrom::End(off) => (off, 2),
    };

    let mut offset_zval = Zval::new();
    let _ = offset_zval.set_long(offset);
    let mut whence_zval = Zval::new();
    let _ = whence_zval.set_long(whence);

    php_io_call_future(bridge, "seek".to_string(), vec![offset_zval, whence_zval])
}

/// Helper to process seek result
fn process_seek_result(result: Zval) -> IoResult<u64> {
    if let Some(pos) = result.long() {
        Ok(pos as u64)
    } else {
        Err(IoError::new(ErrorKind::InvalidData, "Invalid seek response"))
    }
}

// ==================== Universal PhpIo Implementation ====================

/// State holding all possible pending operations
#[derive(Default)]
struct PhpIoState {
    // Read state
    read_eof: bool,
    
    // BufRead state
    buffer: Vec<u8>,
    
    // Write state
    pending_write: Option<LocalBoxFuture<'static, IoResult<usize>>>,
    pending_flush: Option<LocalBoxFuture<'static, IoResult<()>>>,
    pending_shutdown: Option<LocalBoxFuture<'static, IoResult<()>>>,
    
    // Seek state
    pending_seek: Option<SeekFrom>,
}

/// Universal PHP IO wrapper that implements all IO traits.
/// Delegates to PHP methods and fails if not implemented.
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpIo")]
#[pin_project(PinnedDrop)]
pub struct PhpIo {
    bridge: PhpIoBridge,
    #[pin]
    pending: Option<PhpIoCallFuture>,
    state: PhpIoState,
}

unsafe impl Send for PhpIo {}
unsafe impl Sync for PhpIo {}

impl Clone for PhpIo {
    fn clone(&self) -> Self {
        Self {
            bridge: self.bridge.clone(),
            pending: None,
            state: PhpIoState::default(),
        }
    }
}

#[php_impl]
impl PhpIo {
    #[php(constructor)]
    pub fn __construct(handler: &Zval) -> Self {
        Self {
            bridge: PhpIoBridge::new(handler.shallow_clone()),
            pending: None,
            state: PhpIoState::default(),
        }
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let io = self.clone();
        
        // If target includes BUF, treat as AsyncBufRead to preserve native buffering logic
        // (calling PHP's read_line if available)
        if (ty & crate::IO_BUF) != 0 {
             let trait_object: Box<dyn AsyncBufRead + Unpin + Send> = Box::new(io);
             let shared = Shared::new(trait_object);
             return cast_io(&shared, ty);
        }

        // Otherwise treat as ReadWriteSeek which covers other cases
        let trait_object: Box<dyn AsyncReadWriteSeek + Unpin + Send> = Box::new(io);
        let shared = Shared::new(trait_object);
        cast_io(&shared, ty)
    }
}

#[pinned_drop]
impl PinnedDrop for PhpIo {
    fn drop(self: Pin<&mut Self>) {
        self.bridge.close_sync();
    }
}

// Implement AsyncRead
impl AsyncRead for PhpIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        // Consume internal buffer first (from BufRead operations)
        if !this.state.buffer.is_empty() {
            let n = this.state.buffer.len().min(buf.remaining());
            buf.put_slice(&this.state.buffer[..n]);
            this.state.buffer.drain(..n);
            return Poll::Ready(Ok(()));
        }

        if this.state.read_eof {
            return Poll::Ready(Ok(()));
        }

        if this.pending.is_none() {
            let mut len_zval = Zval::new();
            let _ = len_zval.set_long(buf.remaining() as i64);
            this.pending.set(Some(php_io_call_future(
                this.bridge.clone(),
                "read".to_string(),
                vec![len_zval],
            )));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending future must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if result.is_null() {
                    this.state.read_eof = true;
                    return Poll::Ready(Ok(()));
                }
                process_read_response(result, buf)?;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Implement AsyncWrite
impl AsyncWrite for PhpIo {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, write_buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.project();

        if this.state.pending_write.is_none() {
            this.state.pending_write = Some(create_write_future(this.bridge.clone(), write_buf));
        }

        let poll_result = this.state.pending_write.as_mut() 
            .expect("pending write must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.pending_write = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.pending_flush.is_none() {
            this.state.pending_flush = Some(create_flush_future(this.bridge.clone()));
        }

        let poll_result = this.state.pending_flush.as_mut()
            .expect("pending flush must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.pending_flush = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.pending_shutdown.is_none() {
            this.state.pending_shutdown = Some(create_shutdown_future(this.bridge.clone()));
        }

        let poll_result = this.state.pending_shutdown.as_mut()
            .expect("pending shutdown must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.pending_shutdown = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Implement AsyncSeek
impl AsyncSeek for PhpIo {
    fn start_seek(mut self: Pin<&mut Self>, pos: SeekFrom) -> IoResult<()> {
        self.state.pending_seek = Some(pos);
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        let mut this = self.project();

        if this.pending.is_none() {
            let seek = match this.state.pending_seek.take() {
                Some(s) => s,
                None => return Poll::Ready(Err(IoError::new(ErrorKind::Other, "No pending seek"))),
            };

            this.pending.set(Some(create_seek_call_future(this.bridge.clone(), seek)));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending seek must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                Poll::Ready(process_seek_result(result))
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Implement AsyncBufRead
impl AsyncBufRead for PhpIo {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<&[u8]>> {
        let mut this = self.project();

        if !this.state.buffer.is_empty() {
            return Poll::Ready(Ok(&this.state.buffer));
        }

        if this.state.read_eof {
            return Poll::Ready(Ok(&[]));
        }

        if this.pending.is_none() {
            this.pending.set(Some(php_io_call_future(
                this.bridge.clone(),
                "read_line".to_string(),
                vec![],
            )));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if result.is_null() {
                    this.state.read_eof = true;
                    return Poll::Ready(Ok(&[]));
                }

                if let Some(bytes) = result.binary() {
                    this.state.buffer.extend_from_slice(&bytes);
                } else if let Some(s) = result.str() {
                    this.state.buffer.extend_from_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read_line response",
                    )));
                }

                Poll::Ready(Ok(&this.state.buffer))
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        let n = amt.min(self.state.buffer.len());
        self.state.buffer.drain(..n);
    }
}
