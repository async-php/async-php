use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
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
            let s = String::from_utf8_lossy(&buf).to_string();
            let mut z = Zval::new();
            z.set_string(&s, false).map_err(|e| format!("Zval error: {:?}", e))?;
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

// --- UDP Socket ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\UdpSocket")]
pub struct AsyncUdpSocket {
    inner: Rc<UdpSocket>,
}

#[php_impl]
impl AsyncUdpSocket {
    pub fn bind(addr: String) -> RustFuture {
        let future = async move {
            let socket = UdpSocket::bind(addr).await.map_err(|e| e.to_string())?;
            // UDP Socket in Tokio has recv_from/send_to taking &self (no mut needed)
            let obj = AsyncUdpSocket { inner: Rc::new(socket) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUdpSocket to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn recv_from(&self, length: usize) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            // recv_from only needs &self
            let (n, addr) = socket.recv_from(&mut buf).await.map_err(|e| e.to_string())?;
            
            buf.truncate(n);
            let s = String::from_utf8_lossy(&buf).to_string();
            
            let mut arr = ext_php_rs::types::ZendHashTable::new();
            arr.push(s).map_err(|e| format!("Failed to push data: {:?}", e))?;
            arr.push(addr.to_string()).map_err(|e| format!("Failed to push addr: {:?}", e))?;
            
            arr.into_zval(false).map_err(|e| format!("Failed to convert array to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn send_to(&self, data: String, addr: String) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            // send_to only needs &self
            let n = socket.send_to(data.as_bytes(), &addr).await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_long(n as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }
}

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
            let s = String::from_utf8_lossy(&buf).to_string();
            let mut z = Zval::new();
            z.set_string(&s, false).map_err(|e| format!("Zval error: {:?}", e))?;
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