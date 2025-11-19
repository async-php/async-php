/// Core IO utilities for async-php
/// This module provides Rust types that wrap tokio IO traits

use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::{Shared, tuple2};
use crate::channel::AsyncChannel;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeek, AsyncBufRead, AsyncReadExt, AsyncWriteExt, AsyncSeekExt, AsyncBufReadExt};
use std::io::{SeekFrom, Result as IoResult, Error as IoError, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::future::LocalBoxFuture;
use futures::FutureExt;

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
    /// Create AsyncBufReader from a Shared-wrapped reader
    /// This allows multiple AsyncBufReader instances to share the same underlying reader
    pub fn from_shared(shared: Shared<Box<dyn AsyncBufRead + Unpin>>) -> Self {
        Self { inner: shared }
    }

    /// Create AsyncBufReader from a tokio AsyncBufRead type
    /// This wraps the reader in a new Shared container
    pub fn new<B: AsyncBufRead + Unpin + 'static>(reader: B) -> Self {
        Self {
            inner: Shared::new(Box::new(reader)),
        }
    }

    /// Get a clone of the inner Shared without consuming self
    pub fn get_inner(&self) -> Shared<Box<dyn AsyncBufRead + Unpin>> {
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

/// PhpReader implements AsyncRead for PHP IO objects
///
/// Calls PHP method via channel: ['read', [length]] -> bytes
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpReader")]
pub struct PhpReader {
    bridge: PhpIoBridge,
    pending: Option<PhpIoCallFuture>,
    eof: bool,
}

unsafe impl Send for PhpReader {}
unsafe impl Sync for PhpReader {}

#[php_impl]
impl PhpReader {
    /// Create a PhpReader from dual channels
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            bridge: PhpIoBridge::new(request_channel, response_channel),
            pending: None,
            eof: false,
        }
    }

    /// Convert to AsyncReader for use in async operations
    #[php]
    pub fn as_reader(&self) -> AsyncReader {
        // Create a clone of self wrapped as AsyncRead
        let reader = PhpReader {
            bridge: self.bridge.clone(),
            pending: None,
            eof: false,
        };

        let trait_object: Shared<Box<dyn AsyncRead + Unpin + Send>> =
            Shared::new(Box::new(reader));
        AsyncReader::from_shared(trait_object)
    }
}

impl Drop for PhpReader {
    fn drop(&mut self) {
        self.bridge.close_sync();
    }
}

impl AsyncRead for PhpReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        if self.eof {
            return Poll::Ready(Ok(()));
        }

        if self.pending.is_none() {
            let mut len_zval = Zval::new();
            let _ = len_zval.set_long(buf.remaining() as i64);
            self.pending = Some(php_io_call_future(
                self.bridge.clone(),
                "read".to_string(),
                vec![len_zval],
            ));
        }

        let poll_result = {
            let fut = self
                .pending
                .as_mut()
                .expect("pending future must exist after initialization");
            fut.as_mut().poll(cx)
        };

        match poll_result {
            Poll::Ready(Ok(result)) => {
                self.pending = None;
                if result.is_null() {
                    self.eof = true;
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
                self.pending = None;
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// PhpWriter implements AsyncWrite for PHP IO objects
///
/// Calls PHP methods via channel:
/// - write: ['write', [data]] -> bytes_written
/// - flush: ['flush', []] -> success
/// - close: ['close', []] -> success
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpWriter")]
pub struct PhpWriter {
    bridge: PhpIoBridge,
    pending_write: Option<LocalBoxFuture<'static, IoResult<usize>>>,
    pending_flush: Option<LocalBoxFuture<'static, IoResult<()>>>,
    pending_shutdown: Option<LocalBoxFuture<'static, IoResult<()>>>,
}

unsafe impl Send for PhpWriter {}
unsafe impl Sync for PhpWriter {}

#[php_impl]
impl PhpWriter {
    /// Create a PhpWriter from dual channels
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            bridge: PhpIoBridge::new(request_channel, response_channel),
            pending_write: None,
            pending_flush: None,
            pending_shutdown: None,
        }
    }

    /// Convert to AsyncWriter for use in async operations
    #[php]
    pub fn as_writer(&self) -> AsyncWriter {
        let writer = PhpWriter {
            bridge: self.bridge.clone(),
            pending_write: None,
            pending_flush: None,
            pending_shutdown: None,
        };

        let trait_object: Shared<Box<dyn AsyncWrite + Unpin + Send>> =
            Shared::new(Box::new(writer));
        AsyncWriter::from_shared(trait_object)
    }
}

impl Drop for PhpWriter {
    fn drop(&mut self) {
        self.bridge.close_sync();
    }
}

impl AsyncWrite for PhpWriter {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.get_mut();

        if this.pending_write.is_none() {
            let mut data_zval = Zval::new();
            data_zval.set_binary(buf.to_vec());
            let bridge = this.bridge.clone();
            this.pending_write = Some(
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

        let poll_result = {
            let fut = this
                .pending_write
                .as_mut()
                .expect("pending write future must exist after initialization");
            fut.as_mut().poll(cx)
        };

        match poll_result {
            Poll::Ready(res) => {
                this.pending_write = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.get_mut();

        if this.pending_flush.is_none() {
            let bridge = this.bridge.clone();
            this.pending_flush = Some(
                async move {
                    let _ = bridge.call("flush", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = {
            let fut = this
                .pending_flush
                .as_mut()
                .expect("pending flush future must exist after initialization");
            fut.as_mut().poll(cx)
        };

        match poll_result {
            Poll::Ready(res) => {
                this.pending_flush = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.get_mut();

        if this.pending_shutdown.is_none() {
            let bridge = this.bridge.clone();
            this.pending_shutdown = Some(
                async move {
                    let _ = bridge.call("close", vec![]).await?;
                    Ok(())
                }
                .boxed_local(),
            );
        }

        let poll_result = {
            let fut = this
                .pending_shutdown
                .as_mut()
                .expect("pending shutdown future must exist after initialization");
            fut.as_mut().poll(cx)
        };

        match poll_result {
            Poll::Ready(res) => {
                this.pending_shutdown = None;
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// PhpSeeker implements AsyncSeek for PHP IO objects
///
/// Calls PHP method via channel: ['seek', [offset, whence]] -> new_position
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpSeeker")]
pub struct PhpSeeker {
    bridge: PhpIoBridge,
    pending: Option<SeekFrom>,
    pending_call: Option<PhpIoCallFuture>,
}

unsafe impl Send for PhpSeeker {}
unsafe impl Sync for PhpSeeker {}

#[php_impl]
impl PhpSeeker {
    /// Create a PhpSeeker from dual channels
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            bridge: PhpIoBridge::new(request_channel, response_channel),
            pending: None,
            pending_call: None,
        }
    }

    /// Convert to AsyncSeeker for use in async operations
    #[php]
    pub fn as_seeker(&self) -> AsyncSeeker {
        let seeker = PhpSeeker {
            bridge: self.bridge.clone(),
            pending: None,
            pending_call: None,
        };

        let trait_object: Shared<Box<dyn AsyncSeek + Unpin + Send>> =
            Shared::new(Box::new(seeker));
        AsyncSeeker::from_shared(trait_object)
    }
}

impl Drop for PhpSeeker {
    fn drop(&mut self) {
        self.bridge.close_sync();
    }
}

impl AsyncSeek for PhpSeeker {
    fn start_seek(mut self: Pin<&mut Self>, pos: SeekFrom) -> IoResult<()> {
        self.pending = Some(pos);
        Ok(())
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        if self.pending_call.is_none() {
            let seek = match self.pending.take() {
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

            self.pending_call = Some(php_io_call_future(
                self.bridge.clone(),
                "seek".to_string(),
                vec![offset_zval, whence_zval],
            ));
        }

        let poll_result = {
            let fut = self
                .pending_call
                .as_mut()
                .expect("pending seek future must exist after initialization");
            fut.as_mut().poll(cx)
        };

        match poll_result {
            Poll::Ready(Ok(result)) => {
                self.pending_call = None;
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
                self.pending_call = None;
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// PhpBufReader implements AsyncBufRead for PHP IO objects
///
/// Calls PHP methods via channel:
/// - read_line: ['read_line', []] -> line
/// - read: ['read', [length]] -> bytes
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpBufReader")]
pub struct PhpBufReader {
    bridge: PhpIoBridge,
    buffer: Vec<u8>,
    pending: Option<PhpBufReaderPending>,
    eof: bool,
}

unsafe impl Send for PhpBufReader {}
unsafe impl Sync for PhpBufReader {}

enum PhpBufReaderPending {
    Read(PhpIoCallFuture),
    ReadLine(PhpIoCallFuture),
}

#[php_impl]
impl PhpBufReader {
    /// Create a PhpBufReader from dual channels
    #[php(constructor)]
    pub fn __construct(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            bridge: PhpIoBridge::new(request_channel, response_channel),
            buffer: Vec::new(),
            pending: None,
            eof: false,
        }
    }

    /// Convert to AsyncBufReader for use in async operations
    #[php]
    pub fn as_buf_reader(&self) -> AsyncBufReader {
        let reader = PhpBufReader {
            bridge: self.bridge.clone(),
            buffer: Vec::new(),
            pending: None,
            eof: false,
        };

        let trait_object: Shared<Box<dyn AsyncBufRead + Unpin>> =
            Shared::new(Box::new(reader));
        AsyncBufReader::from_shared(trait_object)
    }
}

impl Drop for PhpBufReader {
    fn drop(&mut self) {
        self.bridge.close_sync();
    }
}

impl AsyncRead for PhpBufReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        // Consume buffer first
        if !self.buffer.is_empty() {
            let n = self.buffer.len().min(buf.remaining());
            buf.put_slice(&self.buffer[..n]);
            self.buffer.drain(..n);
            return Poll::Ready(Ok(()));
        }

        if self.eof {
            return Poll::Ready(Ok(()));
        }

        if self.pending.is_none() {
            let mut len_zval = Zval::new();
            let _ = len_zval.set_long(buf.remaining() as i64);
            self.pending = Some(PhpBufReaderPending::Read(php_io_call_future(
                self.bridge.clone(),
                "read".to_string(),
                vec![len_zval],
            )));
        }

        let poll_result = match self.pending.as_mut().expect("pending must exist") {
            PhpBufReaderPending::Read(fut) | PhpBufReaderPending::ReadLine(fut) => fut.as_mut().poll(cx),
        };

        match poll_result {
            Poll::Ready(Ok(result)) => {
                self.pending = None;
                if result.is_null() {
                    self.eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    self.buffer.extend_from_slice(&bytes);
                } else if let Some(s) = result.str() {
                    self.buffer.extend_from_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }

                let n = self.buffer.len().min(buf.remaining());
                buf.put_slice(&self.buffer[..n]);
                self.buffer.drain(..n);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => {
                self.pending = None;
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncBufRead for PhpBufReader {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<&[u8]>> {
        let this = self.get_mut();

        if !this.buffer.is_empty() {
            return Poll::Ready(Ok(&this.buffer));
        }

        if this.eof {
            return Poll::Ready(Ok(&[]));
        }

        if this.pending.is_none() {
            this.pending = Some(PhpBufReaderPending::ReadLine(php_io_call_future(
                this.bridge.clone(),
                "read_line".to_string(),
                vec![],
            )));
        }

        let poll_result = match this.pending.as_mut().expect("pending must exist") {
            PhpBufReaderPending::Read(fut) | PhpBufReaderPending::ReadLine(fut) => fut.as_mut().poll(cx),
        };

        match poll_result {
            Poll::Ready(Ok(result)) => {
                this.pending = None;
                if result.is_null() {
                    this.eof = true;
                    return Poll::Ready(Ok(&[]));
                }

                if let Some(bytes) = result.binary() {
                    this.buffer.extend_from_slice(&bytes);
                } else if let Some(s) = result.str() {
                    this.buffer.extend_from_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read_line response",
                    )));
                }

                Poll::Ready(Ok(&this.buffer))
            }
            Poll::Ready(Err(e)) => {
                this.pending = None;
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        let n = amt.min(self.buffer.len());
        self.buffer.drain(..n);
    }
}
