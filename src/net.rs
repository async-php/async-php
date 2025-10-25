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
            match TcpListener::bind(addr).await {
                Ok(listener) => {
                    let obj = AsyncTcpListener { inner: Rc::new(listener) };
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
                    // TcpStream needs RefCell because Read/Write require &mut self
                    let obj = AsyncTcpStream { inner: Rc::new(RefCell::new(stream)) };
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
#[php(name = "Async\\Kernel\\Network\\TcpStream")]
pub struct AsyncTcpStream {
    inner: Rc<RefCell<TcpStream>>,
}

#[php_impl]
impl AsyncTcpStream {
    pub fn connect(addr: String) -> RustFuture {
        let future = async move {
            match TcpStream::connect(addr).await {
                Ok(stream) => {
                    let obj = AsyncTcpStream { inner: Rc::new(RefCell::new(stream)) };
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
            if let Ok(mut lock) = stream.try_borrow_mut() {
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
            } else {
                let mut z = Zval::new();
                z.set_bool(false); 
                z
            }
        };
        RustFuture::new(future)
    }

    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            if let Ok(mut lock) = stream.try_borrow_mut() {
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
            } else {
                let mut z = Zval::new();
                z.set_bool(false); 
                z
            }
        };
        RustFuture::new(future)
    }

    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
             let mut z = Zval::new();
             if let Ok(mut lock) = stream.try_borrow_mut() {
                 let _ = lock.shutdown().await;
                 z.set_bool(true);
             } else {
                 z.set_bool(false);
             }
             z
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
            match UdpSocket::bind(addr).await {
                Ok(socket) => {
                    // UDP Socket in Tokio has recv_from/send_to taking &self (no mut needed)
                    let obj = AsyncUdpSocket { inner: Rc::new(socket) };
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
            // recv_from only needs &self
            match socket.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    buf.truncate(n);
                    let s = String::from_utf8_lossy(&buf).to_string();
                    
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
            // send_to only needs &self
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
