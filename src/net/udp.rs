use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::net::UdpSocket;
use std::rc::Rc;

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
