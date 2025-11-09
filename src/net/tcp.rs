use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// --- TCP Listener ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TcpListener")]
pub struct AsyncTcpListener {
    inner: Shared<TcpListener>,
}

#[php_impl]
impl AsyncTcpListener {
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

    pub fn local_addr(&self) -> String {
        self.inner.get_ref().local_addr().map(|a| a.to_string()).unwrap_or_default()
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

    pub fn peer_addr(&self) -> String {
        self.inner.get_ref().peer_addr().map(|a| a.to_string()).unwrap_or_default()
    }

    pub fn set_nodelay(&self, nodelay: bool) -> bool {
        self.inner.get_ref().set_nodelay(nodelay).is_ok()
    }
}
