use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::rc::Rc;
use std::cell::RefCell;

// --- Unix Listener ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\UnixListener")]
pub struct AsyncUnixListener {
    inner: Rc<tokio::net::UnixListener>,
}

#[php_impl]
impl AsyncUnixListener {
    pub fn bind(path: String) -> PhpResult<RustFuture> {
        let future = async move {
            let path_clone = path.clone(); // Clone path for use within the async block
            // Remove file if it exists to avoid EADDRINUSE
            let _ = tokio::fs::remove_file(&path_clone).await;

            let listener = tokio::net::UnixListener::bind(&path_clone).map_err(|e| e.to_string())?;
            let obj = AsyncUnixListener { inner: Rc::new(listener) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUnixListener to Zval: {:?}", e))
        };
        Ok(RustFuture::new(future))
    }

    pub fn accept(&self) -> RustFuture {
        let listener = self.inner.clone();
        let future = async move {
            let (stream, _addr) = listener.accept().await.map_err(|e| e.to_string())?;
            let obj = AsyncUnixStream { inner: Rc::new(RefCell::new(stream)) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUnixStream to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }
}

// --- Unix Stream ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\UnixStream")]
pub struct AsyncUnixStream {
    inner: Rc<RefCell<tokio::net::UnixStream>>,
}

#[php_impl]
impl AsyncUnixStream {
    pub fn connect(path: String) -> PhpResult<RustFuture> {
        let future = async move {
            let stream = tokio::net::UnixStream::connect(&path).await.map_err(|e| e.to_string())?;
            let obj = AsyncUnixStream { inner: Rc::new(RefCell::new(stream)) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUnixStream to Zval: {:?}", e))
        };
        Ok(RustFuture::new(future))
    }

    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let mut lock = stream.try_borrow_mut().map_err(|_| "Resource busy".to_string())?;

            let n = lock.read(&mut buf).await.map_err(|e| e.to_string())?;

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
            let mut lock = stream.try_borrow_mut().map_err(|_| "Resource busy".to_string())?;
            lock.write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(data.len() as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
             let mut lock = stream.try_borrow_mut().map_err(|_| "Resource busy".to_string())?;
             let _ = lock.shutdown().await;
             let mut z = Zval::new();
             z.set_bool(true);
             Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }
}
