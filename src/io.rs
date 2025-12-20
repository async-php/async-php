/// Core IO utilities for async-php
/// This module provides Rust types that wrap tokio IO traits

use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::async_io::cast_io;
use crate::util::{Shared, tuple2};
use crate::channel::AsyncChannel;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeek, AsyncBufRead, AsyncReadExt, AsyncWriteExt, AsyncSeekExt, AsyncBufReadExt};
use std::io::{SeekFrom, Result as IoResult, Error as IoError, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::future::LocalBoxFuture;
use futures::{FutureExt, Future};
use pin_project::{pin_project, pinned_drop};

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

// ==================== Combined Trait Types ====================

/// AsyncReadWriter wraps Shared<Box<dyn AsyncReadWrite>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReadWriter")]
pub struct AsyncReadWriter {
    inner: Shared<Box<dyn AsyncReadWrite>>,
}

impl AsyncReadWriter {
    pub fn from_shared(shared: Shared<Box<dyn AsyncReadWrite>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncReadWrite + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncReadWrite>> {
        self.inner.clone()
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

}

/// AsyncReadSeeker wraps Shared<Box<dyn AsyncReadSeek>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReadSeeker")]
pub struct AsyncReadSeeker {
    inner: Shared<Box<dyn AsyncReadSeek>>,
}

impl AsyncReadSeeker {
    pub fn from_shared(shared: Shared<Box<dyn AsyncReadSeek>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncReadSeek + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncReadSeek>> {
        self.inner.clone()
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
}

/// AsyncWriteSeeker wraps Shared<Box<dyn AsyncWriteSeek>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncWriteSeeker")]
pub struct AsyncWriteSeeker {
    inner: Shared<Box<dyn AsyncWriteSeek>>,
}

impl AsyncWriteSeeker {
    pub fn from_shared(shared: Shared<Box<dyn AsyncWriteSeek>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncWriteSeek + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncWriteSeek>> {
        self.inner.clone()
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
}

/// AsyncReadWriteSeeker wraps Shared<Box<dyn AsyncReadWriteSeek>>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReadWriteSeeker")]
pub struct AsyncReadWriteSeeker {
    inner: Shared<Box<dyn AsyncReadWriteSeek>>,
}

impl AsyncReadWriteSeeker {
    pub fn from_shared(shared: Shared<Box<dyn AsyncReadWriteSeek>>) -> Self {
        Self { inner: shared }
    }

    pub fn new<T: AsyncReadWriteSeek + 'static>(io: T) -> Self {
        Self {
            inner: Shared::new(Box::new(io)),
        }
    }

    pub fn get_inner(&self) -> Shared<Box<dyn AsyncReadWriteSeek>> {
        self.inner.clone()
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

}

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
}

// ==================== Generic Methods for Shared<Box<T>> to Reduce Duplication ====================

/// Implement read method for all Shared<Box<T>> where T implements AsyncRead
impl<T: ?Sized + 'static> Shared<Box<T>>
where
    T: AsyncRead + Unpin + Send,
{
    pub fn read_impl(&self, length: i64) -> RustFuture {
        let inner = self.clone();
        let future = async move {
            let mut buf = vec![0u8; length as usize];
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

// ==================== PHP IO Bridge ====================
// Bridge PHP async IO objects to Rust tokio traits via channel

/// Helper for bridging PHP IO method calls through dual channels
#[derive(Clone)]
struct PhpIoBridge {
    request_tx: flume::Sender<Zval>,
    response_rx: flume::Receiver<Zval>,
}

type PhpIoCallFuture = LocalBoxFuture<'static, IoResult<Zval>>;

fn php_io_call_future(bridge: PhpIoBridge, method: String, args: Vec<Zval>) -> PhpIoCallFuture {
    async move { bridge.call(&method, args).await }.boxed_local()
}

impl PhpIoBridge {
    fn new(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            request_tx: request_channel.get_sender(),
            response_rx: response_channel.get_receiver(),
        }
    }

    /// Build args array from Vec<Zval>
    fn build_args_array(args: Vec<Zval>) -> IoResult<Zval> {
        let mut args_ht = ZendHashTable::new();
        for (i, arg) in args.into_iter().enumerate() {
            args_ht.insert(i as i64, arg)
                .map_err(|_| IoError::new(ErrorKind::Other, "Failed to build args"))?;
        }
        args_ht.into_zval(false)
            .map_err(|_| IoError::new(ErrorKind::Other, "Failed to convert args"))
    }

    async fn call(&self, method: &str, args: Vec<Zval>) -> IoResult<Zval> {
        // Build request: [method, args] using tuple2
        let args_zval = Self::build_args_array(args)?;
        let request = tuple2(method, args_zval);

        // Send request through request channel
        self.request_tx.send_async(request).await
            .map_err(|_| IoError::new(ErrorKind::BrokenPipe, "Send failed"))?;

        // Receive response from response channel
        self.response_rx.recv_async().await
            .map_err(|_| IoError::new(ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Send close command to terminate the spawned fiber
    fn close_sync(&self) {
        // Build request: ['__close__', []] using tuple2
        let empty_args = ZendHashTable::new().into_zval(false).unwrap_or_else(|_| Zval::new());
        let request = tuple2("__close__", empty_args);

        // Try to send close command (best effort, ignore errors)
        let _ = self.request_tx.try_send(request);
    }
}

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
        let io: Shared<Box<dyn AsyncReadWrite>> = Shared::new(Box::new(rw));
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
        let io: Shared<Box<dyn AsyncReadSeek>> = Shared::new(Box::new(rs));
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
        let io: Shared<Box<dyn AsyncWriteSeek>> = Shared::new(Box::new(ws));
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
        let io: Shared<Box<dyn AsyncReadWriteSeek>> = Shared::new(Box::new(rws));
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
