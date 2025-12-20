use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::async_io::cast_io;
use crate::util::Shared;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

// --- TCP Listener ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TcpListener")]
pub struct AsyncTcpListener {
    inner: Shared<TcpListener>,
}

#[php_impl]
impl AsyncTcpListener {
    /// Bind to an address
    pub fn bind(addr: String) -> PhpResult<RustFuture> {
        let future = async move {
            let listener = TcpListener::bind(addr).await.map_err(|e| e.to_string())?;
            let obj = AsyncTcpListener { inner: Shared::new(listener) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncTcpListener to Zval: {:?}", e))
        };
        Ok(RustFuture::new(future))
    }

    /// Accept a new incoming connection
    pub fn accept(&self) -> RustFuture {
        let listener = self.inner.clone();
        let future = async move {
            let (stream, _addr) = listener.get_ref().accept().await.map_err(|e| e.to_string())?;
            let obj = AsyncTcpStream { inner: Shared::new(stream) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncTcpStream to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    /// Get the local address this listener is bound to
    pub fn local_addr(&self) -> String {
        self.inner.get_ref().local_addr().map(|a| a.to_string()).unwrap_or_default()
    }

    /// Get the value of the IP_TTL option for this socket
    pub fn ttl(&self) -> i64 {
        self.inner.get_ref().ttl().unwrap_or(0) as i64
    }

    /// Set the value of the IP_TTL option for this socket
    pub fn set_ttl(&self, ttl: i64) -> bool {
        self.inner.get_ref().set_ttl(ttl as u32).is_ok()
    }
}

// --- TCP Stream ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TcpStream")]
pub struct AsyncTcpStream {
    inner: Shared<TcpStream>,
}

#[php_impl]
impl AsyncTcpStream {
    /// Connect to a remote address
    pub fn connect(addr: String) -> RustFuture {
        let future = async move {
            let stream = TcpStream::connect(addr).await.map_err(|e| e.to_string())?;
            let obj = AsyncTcpStream { inner: Shared::new(stream) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncTcpStream to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    /// Read up to length bytes from the stream
    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];

            let n = stream.get_mut().read(&mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::new());
            }

            buf.truncate(n);
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Peek at incoming data without removing it from the buffer
    pub fn peek(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];

            let n = stream.get_ref().peek(&mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::new());
            }

            buf.truncate(n);
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Write all bytes from data to the stream
    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(data.len() as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Shutdown the connection
    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
             stream.get_mut().shutdown().await.map_err(|e| e.to_string())?;

             let mut z = Zval::new();
             z.set_bool(true);
             Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Get the remote peer address
    pub fn peer_addr(&self) -> String {
        self.inner.get_ref().peer_addr().map(|a| a.to_string()).unwrap_or_default()
    }

    /// Get the local address
    pub fn local_addr(&self) -> String {
        self.inner.get_ref().local_addr().map(|a| a.to_string()).unwrap_or_default()
    }

    /// Get the value of the TCP_NODELAY option
    pub fn nodelay(&self) -> bool {
        self.inner.get_ref().nodelay().unwrap_or(false)
    }

    /// Set the value of the TCP_NODELAY option
    pub fn set_nodelay(&self, nodelay: bool) -> bool {
        self.inner.get_ref().set_nodelay(nodelay).is_ok()
    }

    /// Get the value of the IP_TTL option
    pub fn ttl(&self) -> i64 {
        self.inner.get_ref().ttl().unwrap_or(0) as i64
    }

    /// Set the value of the IP_TTL option
    pub fn set_ttl(&self, ttl: i64) -> bool {
        self.inner.get_ref().set_ttl(ttl as u32).is_ok()
    }


    /// Get the value of the SO_LINGER option (returns linger timeout in seconds, or -1 if disabled)
    pub fn linger(&self) -> i64 {
        match self.inner.get_ref().linger() {
            Ok(Some(duration)) => duration.as_secs() as i64,
            Ok(None) => -1,
            Err(_) => -1,
        }
    }

    /// Set the value of the SO_LINGER option (use None or -1 to disable)
    pub fn set_linger(&self, secs: i64) -> bool {
        let duration = if secs >= 0 {
            Some(Duration::from_secs(secs as u64))
        } else {
            None
        };
        self.inner.get_ref().set_linger(duration).is_ok()
    }

    /// Cast this stream into a specific Kernel IO wrapper by bitflags.
    ///
    /// Example:
    /// - `castTo(IO_READ | IO_WRITE)` => `AsyncReadWriter`
    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let io: Shared<Box<dyn crate::io::AsyncReadWrite>> =
            Shared::new(Box::new(self.inner.clone()));
        cast_io(&io, ty)
    }
}

impl AsyncTcpStream {
    /// Internal: Get inner Shared<TcpStream> for zero-copy operations
    #[allow(dead_code)]
    pub(crate) fn get_inner(&self) -> Shared<TcpStream> {
        self.inner.clone()
    }
}
