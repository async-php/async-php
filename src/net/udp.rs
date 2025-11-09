use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use tokio::net::UdpSocket;
use std::net::{Ipv4Addr, Ipv6Addr};

// --- UDP Socket ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\UdpSocket")]
pub struct AsyncUdpSocket {
    inner: Shared<UdpSocket>,
}

#[php_impl]
impl AsyncUdpSocket {
    /// Bind a UDP socket to the specified address
    pub fn bind(addr: String) -> RustFuture {
        let future = async move {
            let socket = UdpSocket::bind(addr).await.map_err(|e| e.to_string())?;
            let obj = AsyncUdpSocket { inner: Shared::new(socket) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUdpSocket to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    /// Receive a datagram from the socket, returns [data, sender_address]
    pub fn recv_from(&self, length: usize) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let (n, addr) = socket.get_ref().recv_from(&mut buf).await.map_err(|e| e.to_string())?;

            buf.truncate(n);

            let mut arr = ext_php_rs::types::ZendHashTable::new();
            arr.push(buf).map_err(|e| format!("Failed to push data: {:?}", e))?;
            arr.push(addr.to_string()).map_err(|e| format!("Failed to push addr: {:?}", e))?;

            arr.into_zval(false).map_err(|e| format!("Failed to convert array to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    /// Peek at incoming data without removing it, returns [data, sender_address]
    pub fn peek_from(&self, length: usize) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let (n, addr) = socket.get_ref().peek_from(&mut buf).await.map_err(|e| e.to_string())?;

            buf.truncate(n);

            let mut arr = ext_php_rs::types::ZendHashTable::new();
            arr.push(buf).map_err(|e| format!("Failed to push data: {:?}", e))?;
            arr.push(addr.to_string()).map_err(|e| format!("Failed to push addr: {:?}", e))?;

            arr.into_zval(false).map_err(|e| format!("Failed to convert array to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    /// Send a datagram to the specified address, returns number of bytes sent
    pub fn send_to(&self, data: String, addr: String) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let n = socket.get_ref().send_to(data.as_bytes(), &addr).await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_long(n as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Connect this UDP socket to a remote address (for send/recv without specifying address)
    pub fn connect(&self, addr: String) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            socket.get_ref().connect(&addr).await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_bool(true);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Receive a datagram from the connected remote address
    pub fn recv(&self, length: usize) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let n = socket.get_ref().recv(&mut buf).await.map_err(|e| e.to_string())?;

            buf.truncate(n);
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Send a datagram to the connected remote address
    pub fn send(&self, data: String) -> RustFuture {
        let socket = self.inner.clone();
        let future = async move {
            let n = socket.get_ref().send(data.as_bytes()).await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_long(n as i64);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Get the local address this socket is bound to
    pub fn local_addr(&self) -> String {
        self.inner.get_ref().local_addr().map(|a| a.to_string()).unwrap_or_default()
    }

    /// Get the remote address this socket is connected to (if connected)
    pub fn peer_addr(&self) -> String {
        self.inner.get_ref().peer_addr().map(|a| a.to_string()).unwrap_or_default()
    }

    /// Get the value of the SO_BROADCAST option
    pub fn broadcast(&self) -> bool {
        self.inner.get_ref().broadcast().unwrap_or(false)
    }

    /// Set the value of the SO_BROADCAST option
    pub fn set_broadcast(&self, broadcast: bool) -> bool {
        self.inner.get_ref().set_broadcast(broadcast).is_ok()
    }

    /// Get the value of the IP_TTL option
    pub fn ttl(&self) -> i64 {
        self.inner.get_ref().ttl().unwrap_or(0) as i64
    }

    /// Set the value of the IP_TTL option
    pub fn set_ttl(&self, ttl: i64) -> bool {
        self.inner.get_ref().set_ttl(ttl as u32).is_ok()
    }

    /// Join a multicast group (IPv4 address, interface address)
    pub fn join_multicast_v4(&self, multicast_addr: String, interface_addr: String) -> bool {
        let multicast: Result<Ipv4Addr, _> = multicast_addr.parse();
        let interface: Result<Ipv4Addr, _> = interface_addr.parse();

        match (multicast, interface) {
            (Ok(m), Ok(i)) => self.inner.get_ref().join_multicast_v4(m, i).is_ok(),
            _ => false,
        }
    }

    /// Leave a multicast group (IPv4 address, interface address)
    pub fn leave_multicast_v4(&self, multicast_addr: String, interface_addr: String) -> bool {
        let multicast: Result<Ipv4Addr, _> = multicast_addr.parse();
        let interface: Result<Ipv4Addr, _> = interface_addr.parse();

        match (multicast, interface) {
            (Ok(m), Ok(i)) => self.inner.get_ref().leave_multicast_v4(m, i).is_ok(),
            _ => false,
        }
    }

    /// Join a multicast group (IPv6 address, interface index)
    pub fn join_multicast_v6(&self, multicast_addr: String, interface_index: i64) -> bool {
        let multicast: Result<Ipv6Addr, _> = multicast_addr.parse();

        match multicast {
            Ok(m) => self.inner.get_ref().join_multicast_v6(&m, interface_index as u32).is_ok(),
            _ => false,
        }
    }

    /// Leave a multicast group (IPv6 address, interface index)
    pub fn leave_multicast_v6(&self, multicast_addr: String, interface_index: i64) -> bool {
        let multicast: Result<Ipv6Addr, _> = multicast_addr.parse();

        match multicast {
            Ok(m) => self.inner.get_ref().leave_multicast_v6(&m, interface_index as u32).is_ok(),
            _ => false,
        }
    }

    /// Get the value of the IP_MULTICAST_LOOP option
    pub fn multicast_loop_v4(&self) -> bool {
        self.inner.get_ref().multicast_loop_v4().unwrap_or(false)
    }

    /// Set the value of the IP_MULTICAST_LOOP option
    pub fn set_multicast_loop_v4(&self, enabled: bool) -> bool {
        self.inner.get_ref().set_multicast_loop_v4(enabled).is_ok()
    }

    /// Get the value of the IP_MULTICAST_TTL option
    pub fn multicast_ttl_v4(&self) -> i64 {
        self.inner.get_ref().multicast_ttl_v4().unwrap_or(0) as i64
    }

    /// Set the value of the IP_MULTICAST_TTL option
    pub fn set_multicast_ttl_v4(&self, ttl: i64) -> bool {
        self.inner.get_ref().set_multicast_ttl_v4(ttl as u32).is_ok()
    }

    /// Get the value of the IPV6_MULTICAST_LOOP option
    pub fn multicast_loop_v6(&self) -> bool {
        self.inner.get_ref().multicast_loop_v6().unwrap_or(false)
    }

    /// Set the value of the IPV6_MULTICAST_LOOP option
    pub fn set_multicast_loop_v6(&self, enabled: bool) -> bool {
        self.inner.get_ref().set_multicast_loop_v6(enabled).is_ok()
    }
}
