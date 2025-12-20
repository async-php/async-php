/// Generic PhpIo implementation for async IO operations
/// Provides bridge between PHP objects and Rust tokio traits

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::io::{Error as IoError, ErrorKind, Result as IoResult, SeekFrom};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};
use futures::future::LocalBoxFuture;
use futures::{FutureExt, Future};
use pin_project::{pin_project, pinned_drop};
use crate::channel::AsyncChannel;
use crate::util::Shared;

// Re-export types from parent module
use super::php_bridge::{PhpIoBridge, PhpIoCallFuture, php_io_call_future};
use super::traits::{AsyncReadSeek, AsyncReadWrite, AsyncReadWriteSeek, AsyncWriteSeek};
use super::cast::cast_io;

// ==================== Generic PhpIo Implementation ====================

/// Trait defining an IO operation that can be called via PHP bridge
trait IoOperation: Send + 'static {
    /// State needed for this operation (e.g., eof flag, buffer)
    type State: Default;

    /// Create initial state
    fn init_state() -> Self::State {
        Self::State::default()
    }
}

/// Read operation state
#[derive(Default)]
struct ReadState {
    eof: bool,
}

/// Write operation state
#[derive(Default)]
struct WriteState {
    pending_write: Option<LocalBoxFuture<'static, IoResult<usize>>>,
    pending_flush: Option<LocalBoxFuture<'static, IoResult<()>>>,
    pending_shutdown: Option<LocalBoxFuture<'static, IoResult<()>>>,
}

/// Seek operation state
#[derive(Default)]
struct SeekState {
    pending_seek: Option<SeekFrom>,
}

/// BufRead operation state
#[derive(Default)]
struct BufReadState {
    buffer: Vec<u8>,
    eof: bool,
}

/// Read operation marker
struct ReadOp;
impl IoOperation for ReadOp {
    type State = ReadState;
}

/// Write operation marker
struct WriteOp;
impl IoOperation for WriteOp {
    type State = WriteState;
}

/// Seek operation marker
struct SeekOp;
impl IoOperation for SeekOp {
    type State = SeekState;
}

/// BufRead operation marker
struct BufReadOp;
impl IoOperation for BufReadOp {
    type State = BufReadState;
}

/// Combined Read+Write operation
struct ReadWriteOp;
impl IoOperation for ReadWriteOp {
    type State = (ReadState, WriteState);
}

/// Combined Read+Seek operation
struct ReadSeekOp;
impl IoOperation for ReadSeekOp {
    type State = (ReadState, SeekState);
}

/// Combined Write+Seek operation
struct WriteSeekOp;
impl IoOperation for WriteSeekOp {
    type State = (WriteState, SeekState);
}

/// Combined Read+Write+Seek operation
struct ReadWriteSeekOp;
impl IoOperation for ReadWriteSeekOp {
    type State = (ReadState, WriteState, SeekState);
}

/// Generic PhpIo structure using pin-project
#[pin_project(PinnedDrop)]
struct PhpIo<Op: IoOperation> {
    bridge: PhpIoBridge,
    #[pin]
    pending: Option<PhpIoCallFuture>,
    state: Op::State,
}

impl<Op: IoOperation> PhpIo<Op> {
    fn new(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            bridge: PhpIoBridge::new(request_channel, response_channel),
            pending: None,
            state: Op::init_state(),
        }
    }
}

impl<Op: IoOperation> Clone for PhpIo<Op> {
    fn clone(&self) -> Self {
        Self {
            bridge: self.bridge.clone(),
            pending: None,
            state: Op::init_state(),
        }
    }
}

#[pinned_drop]
impl<Op: IoOperation> PinnedDrop for PhpIo<Op> {
    fn drop(self: Pin<&mut Self>) {
        self.bridge.close_sync();
    }
}

// Implement AsyncRead for PhpIo<ReadOp>
impl AsyncRead for PhpIo<ReadOp> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if this.state.eof {
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
                    this.state.eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    buf.put_slice(&bytes);
                } else if let Some(s) = result.str() {
                    buf.put_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }
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

// Implement AsyncWrite for PhpIo<WriteOp>
impl AsyncWrite for PhpIo<WriteOp> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, write_buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.project();

        if this.state.pending_write.is_none() {
            let mut data_zval = Zval::new();
            data_zval.set_binary(write_buf.to_vec());
            let bridge = this.bridge.clone();
            this.state.pending_write = Some(
                async move {
                    let result = bridge.call("write", vec![data_zval]).await?;
                    if let Some(n) = result.long() {
                        Ok(n as usize)
                    } else {
                        Err(IoError::new(ErrorKind::InvalidData, "Invalid write response"))
                    }
                }
                .boxed_local(),
            );
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
            let bridge = this.bridge.clone();
            this.state.pending_flush = Some(
                async move {
                    let _ = bridge.call("flush", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
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
            let bridge = this.bridge.clone();
            this.state.pending_shutdown = Some(
                async move {
                    let _ = bridge.call("close", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
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

// Implement AsyncSeek for PhpIo<SeekOp>
impl AsyncSeek for PhpIo<SeekOp> {
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

            let (offset, whence) = match seek {
                SeekFrom::Start(pos) => (pos as i64, 0),
                SeekFrom::Current(off) => (off, 1),
                SeekFrom::End(off) => (off, 2),
            };

            let mut offset_zval = Zval::new();
            let _ = offset_zval.set_long(offset);
            let mut whence_zval = Zval::new();
            let _ = whence_zval.set_long(whence);

            this.pending.set(Some(php_io_call_future(
                this.bridge.clone(),
                "seek".to_string(),
                vec![offset_zval, whence_zval],
            )));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending seek must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if let Some(pos) = result.long() {
                    Poll::Ready(Ok(pos as u64))
                } else {
                    Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid seek response",
                    )))
                }
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Implement AsyncBufRead for PhpIo<BufReadOp>
impl AsyncRead for PhpIo<BufReadOp> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        // Consume buffer first
        if !this.state.buffer.is_empty() {
            let n = this.state.buffer.len().min(buf.remaining());
            buf.put_slice(&this.state.buffer[..n]);
            this.state.buffer.drain(..n);
            return Poll::Ready(Ok(()));
        }

        if this.state.eof {
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
            .expect("pending must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if result.is_null() {
                    this.state.eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    this.state.buffer.extend_from_slice(&bytes);
                } else if let Some(s) = result.str() {
                    this.state.buffer.extend_from_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }

                let n = this.state.buffer.len().min(buf.remaining());
                buf.put_slice(&this.state.buffer[..n]);
                this.state.buffer.drain(..n);
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

impl AsyncBufRead for PhpIo<BufReadOp> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<&[u8]>> {
        let mut this = self.project();

        if !this.state.buffer.is_empty() {
            return Poll::Ready(Ok(&this.state.buffer));
        }

        if this.state.eof {
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
                    this.state.eof = true;
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

// Implement combined operations for PhpIo<ReadWriteOp>
impl AsyncRead for PhpIo<ReadWriteOp> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if this.state.0.eof {
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
                    this.state.0.eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    buf.put_slice(&bytes);
                } else if let Some(s) = result.str() {
                    buf.put_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }
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

impl AsyncWrite for PhpIo<ReadWriteOp> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, write_buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.project();

        if this.state.1.pending_write.is_none() {
            let mut data_zval = Zval::new();
            data_zval.set_binary(write_buf.to_vec());
            let bridge = this.bridge.clone();
            this.state.1.pending_write = Some(
                async move {
                    let result = bridge.call("write", vec![data_zval]).await?;
                    if let Some(n) = result.long() {
                        Ok(n as usize)
                    } else {
                        Err(IoError::new(ErrorKind::InvalidData, "Invalid write response"))
                    }
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.1.pending_write.as_mut()
            .expect("pending write must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.1.pending_write = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.1.pending_flush.is_none() {
            let bridge = this.bridge.clone();
            this.state.1.pending_flush = Some(
                async move {
                    let _ = bridge.call("flush", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.1.pending_flush.as_mut()
            .expect("pending flush must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.1.pending_flush = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.1.pending_shutdown.is_none() {
            let bridge = this.bridge.clone();
            this.state.1.pending_shutdown = Some(
                async move {
                    let _ = bridge.call("close", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.1.pending_shutdown.as_mut()
            .expect("pending shutdown must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.1.pending_shutdown = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Similar implementations for ReadSeekOp, WriteSeekOp, ReadWriteSeekOp...
// (I'll implement these to keep the pattern consistent)

impl AsyncRead for PhpIo<ReadSeekOp> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if this.state.0.eof {
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
                    this.state.0.eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    buf.put_slice(&bytes);
                } else if let Some(s) = result.str() {
                    buf.put_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }
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

impl AsyncSeek for PhpIo<ReadSeekOp> {
    fn start_seek(mut self: Pin<&mut Self>, pos: SeekFrom) -> IoResult<()> {
        self.state.1.pending_seek = Some(pos);
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        let mut this = self.project();

        if this.pending.is_none() {
            let seek = match this.state.1.pending_seek.take() {
                Some(s) => s,
                None => return Poll::Ready(Err(IoError::new(ErrorKind::Other, "No pending seek"))),
            };

            let (offset, whence) = match seek {
                SeekFrom::Start(pos) => (pos as i64, 0),
                SeekFrom::Current(off) => (off, 1),
                SeekFrom::End(off) => (off, 2),
            };

            let mut offset_zval = Zval::new();
            let _ = offset_zval.set_long(offset);
            let mut whence_zval = Zval::new();
            let _ = whence_zval.set_long(whence);

            this.pending.set(Some(php_io_call_future(
                this.bridge.clone(),
                "seek".to_string(),
                vec![offset_zval, whence_zval],
            )));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending seek must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if let Some(pos) = result.long() {
                    Poll::Ready(Ok(pos as u64))
                } else {
                    Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid seek response",
                    )))
                }
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for PhpIo<WriteSeekOp> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, write_buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.project();

        if this.state.0.pending_write.is_none() {
            let mut data_zval = Zval::new();
            data_zval.set_binary(write_buf.to_vec());
            let bridge = this.bridge.clone();
            this.state.0.pending_write = Some(
                async move {
                    let result = bridge.call("write", vec![data_zval]).await?;
                    if let Some(n) = result.long() {
                        Ok(n as usize)
                    } else {
                        Err(IoError::new(ErrorKind::InvalidData, "Invalid write response"))
                    }
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.0.pending_write.as_mut()
            .expect("pending write must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.0.pending_write = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.0.pending_flush.is_none() {
            let bridge = this.bridge.clone();
            this.state.0.pending_flush = Some(
                async move {
                    let _ = bridge.call("flush", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.0.pending_flush.as_mut()
            .expect("pending flush must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.0.pending_flush = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.0.pending_shutdown.is_none() {
            let bridge = this.bridge.clone();
            this.state.0.pending_shutdown = Some(
                async move {
                    let _ = bridge.call("close", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.0.pending_shutdown.as_mut()
            .expect("pending shutdown must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.0.pending_shutdown = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncSeek for PhpIo<WriteSeekOp> {
    fn start_seek(mut self: Pin<&mut Self>, pos: SeekFrom) -> IoResult<()> {
        self.state.1.pending_seek = Some(pos);
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        let mut this = self.project();

        if this.pending.is_none() {
            let seek = match this.state.1.pending_seek.take() {
                Some(s) => s,
                None => return Poll::Ready(Err(IoError::new(ErrorKind::Other, "No pending seek"))),
            };

            let (offset, whence) = match seek {
                SeekFrom::Start(pos) => (pos as i64, 0),
                SeekFrom::Current(off) => (off, 1),
                SeekFrom::End(off) => (off, 2),
            };

            let mut offset_zval = Zval::new();
            let _ = offset_zval.set_long(offset);
            let mut whence_zval = Zval::new();
            let _ = whence_zval.set_long(whence);

            this.pending.set(Some(php_io_call_future(
                this.bridge.clone(),
                "seek".to_string(),
                vec![offset_zval, whence_zval],
            )));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending seek must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if let Some(pos) = result.long() {
                    Poll::Ready(Ok(pos as u64))
                } else {
                    Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid seek response",
                    )))
                }
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncRead for PhpIo<ReadWriteSeekOp> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if this.state.0.eof {
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
                    this.state.0.eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    buf.put_slice(&bytes);
                } else if let Some(s) = result.str() {
                    buf.put_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }
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

impl AsyncWrite for PhpIo<ReadWriteSeekOp> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, write_buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.project();

        if this.state.1.pending_write.is_none() {
            let mut data_zval = Zval::new();
            data_zval.set_binary(write_buf.to_vec());
            let bridge = this.bridge.clone();
            this.state.1.pending_write = Some(
                async move {
                    let result = bridge.call("write", vec![data_zval]).await?;
                    if let Some(n) = result.long() {
                        Ok(n as usize)
                    } else {
                        Err(IoError::new(ErrorKind::InvalidData, "Invalid write response"))
                    }
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.1.pending_write.as_mut()
            .expect("pending write must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.1.pending_write = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.1.pending_flush.is_none() {
            let bridge = this.bridge.clone();
            this.state.1.pending_flush = Some(
                async move {
                    let _ = bridge.call("flush", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.1.pending_flush.as_mut()
            .expect("pending flush must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.1.pending_flush = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        if this.state.1.pending_shutdown.is_none() {
            let bridge = this.bridge.clone();
            this.state.1.pending_shutdown = Some(
                async move {
                    let _ = bridge.call("close", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = this.state.1.pending_shutdown.as_mut()
            .expect("pending shutdown must exist")
            .as_mut()
            .poll(cx);

        match poll_result {
            Poll::Ready(res) => {
                this.state.1.pending_shutdown = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncSeek for PhpIo<ReadWriteSeekOp> {
    fn start_seek(mut self: Pin<&mut Self>, pos: SeekFrom) -> IoResult<()> {
        self.state.2.pending_seek = Some(pos);
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        let mut this = self.project();

        if this.pending.is_none() {
            let seek = match this.state.2.pending_seek.take() {
                Some(s) => s,
                None => return Poll::Ready(Err(IoError::new(ErrorKind::Other, "No pending seek"))),
            };

            let (offset, whence) = match seek {
                SeekFrom::Start(pos) => (pos as i64, 0),
                SeekFrom::Current(off) => (off, 1),
                SeekFrom::End(off) => (off, 2),
            };

            let mut offset_zval = Zval::new();
            let _ = offset_zval.set_long(offset);
            let mut whence_zval = Zval::new();
            let _ = whence_zval.set_long(whence);

            this.pending.set(Some(php_io_call_future(
                this.bridge.clone(),
                "seek".to_string(),
                vec![offset_zval, whence_zval],
            )));
        }

        let poll_result = this.pending.as_mut().as_pin_mut()
            .expect("pending seek must exist")
            .poll(cx);

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending.set(None);
                if let Some(pos) = result.long() {
                    Poll::Ready(Ok(pos as u64))
                } else {
                    Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid seek response",
                    )))
                }
            }
            Poll::Ready(Err(e)) => {
                this.pending.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// ==================== Type Aliases with PHP Bindings ====================

// ==================== Type Aliases with PHP Bindings ====================

/// PhpReader - AsyncRead implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpReader")]
pub struct PhpReader(PhpIo<ReadOp>);

unsafe impl Send for PhpReader {}
unsafe impl Sync for PhpReader {}

#[php_impl]
impl PhpReader {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let reader = PhpReader(self.0.clone());
        let io: Shared<Box<dyn AsyncRead + Unpin + Send>> = Shared::new(Box::new(reader));
        cast_io(&io, ty)
    }
}

impl AsyncRead for PhpReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

/// PhpWriter - AsyncWrite implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpWriter")]
pub struct PhpWriter(PhpIo<WriteOp>);

unsafe impl Send for PhpWriter {}
unsafe impl Sync for PhpWriter {}

#[php_impl]
impl PhpWriter {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let writer = PhpWriter(self.0.clone());
        let io: Shared<Box<dyn AsyncWrite + Unpin + Send>> = Shared::new(Box::new(writer));
        cast_io(&io, ty)
    }
}

impl AsyncWrite for PhpWriter {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

/// PhpSeeker - AsyncSeek implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpSeeker")]
pub struct PhpSeeker(PhpIo<SeekOp>);

unsafe impl Send for PhpSeeker {}
unsafe impl Sync for PhpSeeker {}

#[php_impl]
impl PhpSeeker {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let seeker = PhpSeeker(self.0.clone());
        let io: Shared<Box<dyn AsyncSeek + Unpin + Send>> = Shared::new(Box::new(seeker));
        cast_io(&io, ty)
    }
}

impl AsyncSeek for PhpSeeker {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> IoResult<()> {
        Pin::new(&mut self.0).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        Pin::new(&mut self.0).poll_complete(cx)
    }
}

/// PhpBufReader - AsyncBufRead implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpBufReader")]
pub struct PhpBufReader(PhpIo<BufReadOp>);

unsafe impl Send for PhpBufReader {}
unsafe impl Sync for PhpBufReader {}

#[php_impl]
impl PhpBufReader {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let reader = PhpBufReader(self.0.clone());
        let io: Shared<Box<dyn AsyncBufRead + Unpin + Send>> = Shared::new(Box::new(reader));
        cast_io(&io, ty)
    }
}

impl AsyncRead for PhpBufReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncBufRead for PhpBufReader {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<&[u8]>> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll_fill_buf(cx) }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        unsafe { self.map_unchecked_mut(|s| &mut s.0).consume(amt) }
    }
}

/// PhpReadWriter - AsyncRead + AsyncWrite implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpReadWriter")]
pub struct PhpReadWriter(PhpIo<ReadWriteOp>);

unsafe impl Send for PhpReadWriter {}
unsafe impl Sync for PhpReadWriter {}

#[php_impl]
impl PhpReadWriter {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let rw = PhpReadWriter(self.0.clone());
        let io: Shared<Box<dyn AsyncReadWrite + Unpin + Send>> = Shared::new(Box::new(rw));
        cast_io(&io, ty)
    }
}

impl AsyncRead for PhpReadWriter {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for PhpReadWriter {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

/// PhpReadSeeker - AsyncRead + AsyncSeek implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpReadSeeker")]
pub struct PhpReadSeeker(PhpIo<ReadSeekOp>);

unsafe impl Send for PhpReadSeeker {}
unsafe impl Sync for PhpReadSeeker {}

#[php_impl]
impl PhpReadSeeker {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let rs = PhpReadSeeker(self.0.clone());
        let io: Shared<Box<dyn AsyncReadSeek + Unpin + Send>> = Shared::new(Box::new(rs));
        cast_io(&io, ty)
    }
}

impl AsyncRead for PhpReadSeeker {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncSeek for PhpReadSeeker {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> IoResult<()> {
        Pin::new(&mut self.0).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        Pin::new(&mut self.0).poll_complete(cx)
    }
}

/// PhpWriteSeeker - AsyncWrite + AsyncSeek implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpWriteSeeker")]
pub struct PhpWriteSeeker(PhpIo<WriteSeekOp>);

unsafe impl Send for PhpWriteSeeker {}
unsafe impl Sync for PhpWriteSeeker {}

#[php_impl]
impl PhpWriteSeeker {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let ws = PhpWriteSeeker(self.0.clone());
        let io: Shared<Box<dyn AsyncWriteSeek + Unpin + Send>> = Shared::new(Box::new(ws));
        cast_io(&io, ty)
    }
}

impl AsyncWrite for PhpWriteSeeker {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl AsyncSeek for PhpWriteSeeker {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> IoResult<()> {
        Pin::new(&mut self.0).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        Pin::new(&mut self.0).poll_complete(cx)
    }
}

/// PhpReadWriteSeeker - AsyncRead + AsyncWrite + AsyncSeek implementation for PHP objects
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpReadWriteSeeker")]
pub struct PhpReadWriteSeeker(PhpIo<ReadWriteSeekOp>);

unsafe impl Send for PhpReadWriteSeeker {}
unsafe impl Sync for PhpReadWriteSeeker {}

#[php_impl]
impl PhpReadWriteSeeker {
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self(PhpIo::new(request_channel, response_channel))
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let rws = PhpReadWriteSeeker(self.0.clone());
        let io: Shared<Box<dyn AsyncReadWriteSeek + Unpin + Send>> = Shared::new(Box::new(rws));
        cast_io(&io, ty)
    }
}

impl AsyncRead for PhpReadWriteSeeker {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for PhpReadWriteSeeker {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl AsyncSeek for PhpReadWriteSeeker {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> IoResult<()> {
        Pin::new(&mut self.0).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        Pin::new(&mut self.0).poll_complete(cx)
    }
}
