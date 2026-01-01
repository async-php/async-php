use std::future::Future;
use std::io::{Error as IoError, ErrorKind, Result as IoResult, SeekFrom};
use std::pin::Pin;
use std::task::{Context, Poll};

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::{IntoZval, IntoZvalDyn};
use pin_project::pin_project;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite, ReadBuf};

use super::cast::cast_io;
use super::traits::AsyncReadWriteSeek;
use crate::runtime::runtime::drive_fiber;
use crate::util::Shared;

// ==================== Fiber Call Helper ====================

async fn invoke_method(
    handler: Zval,
    method: &str,
    args: Vec<Zval>,
) -> IoResult<Zval> {
    let fiber_ce = ClassEntry::try_find("Fiber")
        .ok_or_else(|| IoError::new(ErrorKind::Other, "Fiber class not found"))?;

    let fiber_obj = fiber_ce.new();
    let fiber_zval = fiber_obj.into_zval(false).map_err(|e| {
        IoError::new(
            ErrorKind::Other,
            format!("Failed to create Fiber zval: {:?}", e),
        )
    })?;

    // Construct callable: [$handler, $method]
    let mut callable = Vec::with_capacity(2);
    callable.push(handler);
    callable.push(
        method
            .into_zval(false)
            .map_err(|e| IoError::new(ErrorKind::Other, format!("Failed to convert method: {:?}", e)))?,
    );

    let mut callable_ht = ext_php_rs::types::ZendHashTable::new();
    for item in callable {
        callable_ht.push(item).map_err(|e| {
            IoError::new(
                ErrorKind::Other,
                format!("Failed to push to callable array: {:?}", e),
            )
        })?;
    }
    let callable_zval = callable_ht.into_zval(false).map_err(|e| {
        IoError::new(
            ErrorKind::Other,
            format!("Failed to convert callable to Zval: {:?}", e),
        )
    })?;

    fiber_zval
        .try_call_method("__construct", vec![&callable_zval])
        .map_err(|e| {
            IoError::new(
                ErrorKind::Other,
                format!("Failed to construct Fiber: {:?}", e),
            )
        })?;

    let args_refs: Vec<&dyn IntoZvalDyn> = args.iter().map(|z| z as &dyn IntoZvalDyn).collect();

    drive_fiber(fiber_zval, args_refs)
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Fiber execution failed: {:?}", e)))
}

// ==================== Universal PhpIo Implementation ====================

/// Universal PHP IO wrapper that implements all IO traits.
/// Delegates to PHP methods and fails if not implemented.
#[php_class]
#[php(name = "Async\\Kernel\\IO\\PhpIo")]
#[pin_project]
pub struct PhpIo {
    handler: Zval,

    // Unified future for all IO operations
    #[pin]
    future: Option<Pin<Box<dyn Future<Output = IoResult<Zval>>>>>,

    // Buffering for AsyncBufRead
    read_buf: Vec<u8>,
    read_eof: bool,

    // Pending seek position
    pending_seek: Option<SeekFrom>,
}

unsafe impl Send for PhpIo {}
unsafe impl Sync for PhpIo {}

impl Clone for PhpIo {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.shallow_clone(),
            future: None,
            read_buf: Vec::new(),
            read_eof: self.read_eof,
            pending_seek: None,
        }
    }
}

#[php_impl]
impl PhpIo {
    #[php(constructor)]
    pub fn __construct(handler: &Zval) -> Self {
        Self {
            handler: handler.shallow_clone(),
            future: None,
            read_buf: Vec::new(),
            read_eof: false,
            pending_seek: None,
        }
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let io = self.clone();

        if (ty & crate::IO_BUF) != 0 {
            let trait_object: Box<dyn AsyncBufRead + Unpin + Send> = Box::new(io);
            let shared = Shared::new(trait_object);
            return cast_io(&shared, ty);
        }

        let trait_object: Box<dyn AsyncReadWriteSeek + Unpin + Send> = Box::new(io);
        let shared = Shared::new(trait_object);
        cast_io(&shared, ty)
    }
}

impl AsyncRead for PhpIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if !this.read_buf.is_empty() {
            let n = this.read_buf.len().min(buf.remaining());
            buf.put_slice(&this.read_buf[..n]);
            this.read_buf.drain(..n);
            return Poll::Ready(Ok(()));
        }

        if *this.read_eof {
            return Poll::Ready(Ok(()));
        }

        if this.future.is_none() {
            let mut len_zval = Zval::new();
            let _ = len_zval.set_long(buf.remaining() as i64);
            let handler = this.handler.shallow_clone();

            this.future.set(Some(Box::pin(async move {
                invoke_method(handler, "read", vec![len_zval]).await
            })));
        }

        match this.future.as_mut().as_pin_mut().expect("future").poll(cx) {
            Poll::Ready(Ok(result)) => {
                this.future.set(None);

                if result.is_null() {
                    *this.read_eof = true;
                    return Poll::Ready(Ok(()));
                }

                if let Some(bytes) = result.binary() {
                    let len = bytes.len();
                    if len > buf.remaining() {
                        return Poll::Ready(Err(IoError::new(
                            ErrorKind::InvalidData,
                            "Read response exceeded requested length",
                        )));
                    }
                    buf.put_slice(&bytes);
                } else if let Some(s) = result.str() {
                    let bytes = s.as_bytes();
                    if bytes.len() > buf.remaining() {
                        return Poll::Ready(Err(IoError::new(
                            ErrorKind::InvalidData,
                            "Read response exceeded requested length",
                        )));
                    }
                    buf.put_slice(bytes);
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read response",
                    )));
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => {
                this.future.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for PhpIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        write_buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        let mut this = self.project();

        if this.future.is_none() {
            let mut data_zval = Zval::new();
            data_zval.set_binary(write_buf.to_vec());
            let handler = this.handler.shallow_clone();

            this.future.set(Some(Box::pin(async move {
                invoke_method(handler, "write", vec![data_zval]).await
            })));
        }

        match this.future.as_mut().as_pin_mut().expect("future").poll(cx) {
            Poll::Ready(Ok(res)) => {
                this.future.set(None);
                if let Some(n) = res.long() {
                    Ok(n as usize).into()
                } else {
                    Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid write response",
                    )))
                }
            }
            Poll::Ready(Err(e)) => {
                this.future.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if this.future.is_none() {
            let handler = this.handler.shallow_clone();
            this.future.set(Some(Box::pin(async move {
                invoke_method(handler, "flush", vec![]).await
            })));
        }

        match this.future.as_mut().as_pin_mut().expect("future").poll(cx) {
            Poll::Ready(Ok(_)) => {
                this.future.set(None);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => {
                this.future.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let mut this = self.project();

        if this.future.is_none() {
            let handler = this.handler.shallow_clone();
            this.future.set(Some(Box::pin(async move {
                invoke_method(handler, "close", vec![]).await
            })));
        }

        match this.future.as_mut().as_pin_mut().expect("future").poll(cx) {
            Poll::Ready(Ok(_)) => {
                this.future.set(None);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => {
                this.future.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncSeek for PhpIo {
    fn start_seek(self: Pin<&mut Self>, pos: SeekFrom) -> IoResult<()> {
        *self.project().pending_seek = Some(pos);
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<u64>> {
        let mut this = self.project();

        if this.future.is_none() {
            let seek = match this.pending_seek.take() {
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

            let handler = this.handler.shallow_clone();

            this.future.set(Some(Box::pin(async move {
                invoke_method(handler, "seek", vec![offset_zval, whence_zval]).await
            })));
        }

        match this.future.as_mut().as_pin_mut().expect("future").poll(cx) {
            Poll::Ready(Ok(result)) => {
                this.future.set(None);
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
                this.future.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncBufRead for PhpIo {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<&[u8]>> {
        let mut this = self.project();

        if !this.read_buf.is_empty() {
            return Poll::Ready(Ok(this.read_buf.as_slice()));
        }

        if *this.read_eof {
            return Poll::Ready(Ok(&[]));
        }

        if this.future.is_none() {
            let handler = this.handler.shallow_clone();
            this.future.set(Some(Box::pin(async move {
                invoke_method(handler, "read_line", vec![]).await
            })));
        }

        match this.future.as_mut().as_pin_mut().expect("future").poll(cx) {
            Poll::Ready(Ok(result)) => {
                this.future.set(None);
                if result.is_null() {
                    *this.read_eof = true;
                    return Poll::Ready(Ok(&[]));
                }

                if let Some(bytes) = result.binary() {
                    this.read_buf.extend_from_slice(&bytes);
                } else if let Some(s) = result.str() {
                    this.read_buf.extend_from_slice(s.as_bytes());
                } else {
                    return Poll::Ready(Err(IoError::new(
                        ErrorKind::InvalidData,
                        "Invalid read_line response",
                    )));
                }

                Poll::Ready(Ok(this.read_buf.as_slice()))
            }
            Poll::Ready(Err(e)) => {
                this.future.set(None);
                Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        let n = amt.min(self.read_buf.len());
        self.read_buf.drain(..n);
    }
}
