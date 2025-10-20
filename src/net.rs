use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;

// --- TCP Listener ---

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
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                }
                Err(_e) => {
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
    
    pub fn local_addr(&self) -> String {
        self.inner.local_addr().map(|a| a.to_string()).unwrap_or_default()
    }
}

// --- TCP Stream ---

#[php_class]
pub struct AsyncTcpStream {
    inner: Arc<Mutex<TcpStream>>,
}

#[php_impl]
impl AsyncTcpStream {
    pub fn connect(addr: String) -> RustFuture {
        let future = async move {
            match TcpStream::connect(addr).await {
                Ok(stream) => {
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

    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let mut lock = stream.lock().await;
            match lock.read(&mut buf).await {
                Ok(0) => {
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
    
    pub fn peer_addr(&self) -> String {
        // We need to lock to get access, although peer_addr() on TcpStream technically doesn't need mutable access,
        // our Mutex wrapper enforces it unless we use async lock.
        // But peer_addr is synchronous. We can't block_on here.
        // Limitation: To get peer_addr cleanly without async, we'd need try_lock or similar, 
        // or store the addr on creation.
        // For simplicity, we will attempt to block_on via tokio handle or just return empty string 
        // if we can't get it easily. 
        // Actually, let's make this an async method returning Future<String> to be safe.
        // BUT, standard PHP API is sync. 
        // Let's try try_lock().
        if let Ok(lock) = self.inner.try_lock() {
            lock.peer_addr().map(|a| a.to_string()).unwrap_or_default()
        } else {
            // If locked, we can't get it synchronously easily.
            "".to_string()
        }
    }

    pub fn set_nodelay(&self, nodelay: bool) -> bool {
        if let Ok(lock) = self.inner.try_lock() {
            lock.set_nodelay(nodelay).is_ok()
        } else {
            false
        }
    }
}

// --- UDP Socket ---

#[php_class]
pub struct AsyncUdpSocket {
    inner: Arc<UdpSocket>,
}

#[php_impl]
impl AsyncUdpSocket {
    pub fn bind(addr: String) -> RustFuture {
        let future = async move {
            match UdpSocket::bind(addr).await {
                Ok(socket) => {
                    let obj = AsyncUdpSocket { inner: Arc::new(socket) };
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

    pub fn recv_from(&self, length: usize) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            match socket.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    buf.truncate(n);
                    let s = String::from_utf8_lossy(&buf).to_string();
                    
                    // Return array [data, address]
                    let mut arr = ext_php_rs::types::ZendHashTable::new();
                    arr.push(s).unwrap();
                    arr.push(addr.to_string()).unwrap();
                    
                    let z = arr.into_zval(false).unwrap_or_else(|_| Zval::new());
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

    pub fn send_to(&self, data: String, addr: String) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            match socket.send_to(data.as_bytes(), &addr).await {
                Ok(n) => {
                    let mut z = Zval::new();
                    z.set_long(n as i64);
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
}