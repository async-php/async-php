use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::rc::Rc;
use std::cell::RefCell;

// --- TCP Listener ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TcpListener")]
pub struct AsyncTcpListener {
    inner: Rc<TcpListener>,
}

#[php_impl]
impl AsyncTcpListener {
    pub fn bind(addr: String) -> PhpResult<RustFuture> {
        let future = async move {
            let listener = TcpListener::bind(addr).await.map_err(|e| e.to_string())?;
            let obj = AsyncTcpListener { inner: Rc::new(listener) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncTcpListener to Zval: {:?}", e))
        };
        Ok(RustFuture::new(future))
    }

    pub fn accept(&self) -> RustFuture {
        let listener = self.inner.clone();
        let future = async move {
            let (stream, _addr) = listener.accept().await.map_err(|e| e.to_string())?;
            let obj = AsyncTcpStream { inner: Rc::new(RefCell::new(stream)) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncTcpStream to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn local_addr(&self) -> String {
        self.inner.local_addr().map(|a| a.to_string()).unwrap_or_default()
    }
}

// --- TCP Stream ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TcpStream")]
pub struct AsyncTcpStream {
    inner: Rc<RefCell<TcpStream>>,
}

#[php_impl]
impl AsyncTcpStream {
    pub fn connect(addr: String) -> RustFuture {
        let future = async move {
            let stream = TcpStream::connect(addr).await.map_err(|e| e.to_string())?;
            let obj = AsyncTcpStream { inner: Rc::new(RefCell::new(stream)) };
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
             lock.shutdown().await.map_err(|e| e.to_string())?;

             let mut z = Zval::new();
             z.set_bool(true);
             Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    pub fn peer_addr(&self) -> String {
        if let Ok(lock) = self.inner.try_borrow() {
            lock.peer_addr().map(|a| a.to_string()).unwrap_or_default()
        } else {
            "".to_string()
        }
    }

    pub fn set_nodelay(&self, nodelay: bool) -> bool {
        if let Ok(lock) = self.inner.try_borrow() {
            lock.set_nodelay(nodelay).is_ok()
        } else {
            false
        }
    }
}
