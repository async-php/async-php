use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;

#[php_class]
pub struct AsyncTcpListener {
    inner: Arc<TcpListener>,
}

#[php_impl]
impl AsyncTcpListener {
    pub fn bind(addr: String) -> PhpResult<RustFuture> {
        let future = async move {
            match TcpListener::bind(addr).await {
                Ok(listener) => {
                    let obj = AsyncTcpListener { inner: Arc::new(listener) };
                    match ext_php_rs::types::ZendClassObject::new(obj).into_zval(false) {
                        Ok(z) => z,
                        Err(_e) => {
                             // Ideally throw exception inside future, but we return Zval.
                             // We can return a primitive error code or throw next time.
                             // For now, let's return null on internal allocation error (rare).
                             Zval::new()
                        }
                    }
                }
                Err(_e) => {
                    // Bind failed. Throwing exception from async block is tricky.
                    // Best practice: Return False or Result object.
                    // We will return False for simplicity.
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        Ok(RustFuture::new(future))
    }

    pub fn accept(&self) -> RustFuture {
        let listener = self.inner.clone();
        let future = async move {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let obj = AsyncTcpStream { inner: Arc::new(Mutex::new(stream)) };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                }
                Err(_) => {
                     let mut z = Zval::new();
                     z.set_bool(false);
                     z
                }
            }
        };
        RustFuture::new(future)
    }
}

#[php_class]
pub struct AsyncTcpStream {
    // Mutex needed because AsyncRead/Write require mutable access, 
    // and PHP objects are essentially shared references.
    inner: Arc<Mutex<TcpStream>>,
}

#[php_impl]
impl AsyncTcpStream {
    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let mut lock = stream.lock().await;
            match lock.read(&mut buf).await {
                Ok(0) => {
                    // EOF -> Empty String
                    let mut z = Zval::new();
                    z.set_string("", false).unwrap();
                    z
                },
                Ok(n) => {
                    buf.truncate(n);
                    let s = String::from_utf8_lossy(&buf).to_string();
                    let mut z = Zval::new();
                    z.set_string(&s, false).unwrap();
                    z
                }
                Err(_) => {
                     let mut z = Zval::new();
                     z.set_bool(false);
                     z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut lock = stream.lock().await;
            match lock.write_all(data.as_bytes()).await {
                Ok(_) => {
                    let mut z = Zval::new();
                    z.set_long(data.len() as i64);
                    z
                }
                Err(_) => {
                     let mut z = Zval::new();
                     z.set_bool(false);
                     z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
             let mut lock = stream.lock().await;
             let _ = lock.shutdown().await;
             let mut z = Zval::new();
             z.set_bool(true);
             z
        };
        RustFuture::new(future)
    }
}
