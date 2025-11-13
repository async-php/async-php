use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use flume;
use std::time::Duration;

#[php_class]
#[php(name = "Async\\Kernel\\Channel")]
pub struct AsyncChannel {
    sender: Option<flume::Sender<Zval>>,
    receiver: flume::Receiver<Zval>,
    capacity: i64,
}

#[php_impl]
impl AsyncChannel {
    /// Create a new channel with the specified capacity (Go-style)
    ///
    /// # Arguments
    /// * `capacity` - Buffer capacity (default: 0)
    ///   - 0: unbuffered channel (fully blocking, like Go's `make(chan T)`)
    ///   - 1+: buffered channel (like Go's `make(chan T, N)`)
    ///
    /// # Examples
    /// - `new Channel()` - Creates unbuffered channel (default, like Go's `make(chan T)`)
    /// - `new Channel(1)` - Creates buffered channel with capacity 1
    /// - `new Channel(10)` - Creates buffered channel with capacity 10 (like Go's `make(chan T, 10)`)
    #[php(optional = "capacity")]
    pub fn __construct(capacity: Option<i64>) -> Self {
        let cap = capacity.unwrap_or(0);
        let (tx, rx) = if cap < 0 {
            flume::unbounded()
        } else {
            flume::bounded(cap as usize)
        };

        Self {
            sender: Some(tx),
            receiver: rx,
            capacity: cap,
        }
    }

    /// Send a value to the channel with optional timeout
    ///
    /// # Arguments
    /// * `value` - The value to send
    /// * `timeout` - Timeout in seconds (null for blocking wait)
    ///
    /// # Returns
    /// true on success, false on failure (closed or timeout)
    #[php(optional = "timeout")]
    pub fn send(&self, value: &Zval, timeout: Option<f64>) -> PhpResult<RustFuture> {
        let val = value.shallow_clone();
        let tx = match self.sender.as_ref() {
            Some(sender) => sender.clone(),
            None => {
                return Err(PhpException::default("Cannot send to closed channel".to_string()));
            }
        };

        let future = async move {
            let result = match timeout {
                Some(secs) => {
                    let duration = Duration::from_secs_f64(secs);
                    tokio::time::timeout(duration, tx.send_async(val))
                        .await
                        .map_or(false, |r| r.is_ok())
                }
                None => tx.send_async(val).await.is_ok(),
            };

            Zval::from(result)
        };

        Ok(RustFuture::new(future))
    }

    /// Push a value to the channel with optional timeout (alias for send)
    ///
    /// # Arguments
    /// * `value` - The value to push
    /// * `timeout` - Timeout in seconds (null for blocking wait)
    ///
    /// # Returns
    /// true on success, false on failure (closed or timeout)
    #[php(optional = "timeout")]
    pub fn push(&self, value: &Zval, timeout: Option<f64>) -> PhpResult<RustFuture> {
        self.send(value, timeout)
    }

    /// Receive a value from the channel with optional timeout
    ///
    /// # Arguments
    /// * `timeout` - Timeout in seconds (null for blocking wait)
    ///
    /// # Returns
    /// The received value, or null on failure (closed or timeout)
    #[php(optional = "timeout")]
    pub fn recv(&self, timeout: Option<f64>) -> RustFuture {
        let rx = self.receiver.clone();

        let future = async move {
            match timeout {
                Some(secs) => {
                    let duration = Duration::from_secs_f64(secs);
                    tokio::time::timeout(duration, rx.recv_async())
                        .await
                        .map_or(Zval::new(), |r| r.unwrap_or_else(|_| Zval::new()))
                }
                None => rx.recv_async().await.unwrap_or_else(|_| Zval::new()),
            }
        };

        RustFuture::new(future)
    }

    /// Pop a value from the channel with optional timeout (alias for recv)
    ///
    /// # Arguments
    /// * `timeout` - Timeout in seconds (null for blocking wait)
    ///
    /// # Returns
    /// The received value, or null on failure (closed or timeout)
    #[php(optional = "timeout")]
    pub fn pop(&self, timeout: Option<f64>) -> RustFuture {
        self.recv(timeout)
    }

    /// Check if the channel is empty
    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }

    /// Check if the channel is full
    pub fn is_full(&self) -> bool {
        self.receiver.is_full()
    }

    /// Get the current number of items in the channel
    pub fn length(&self) -> i64 {
        self.receiver.len() as i64
    }

    /// Close the channel
    /// After closing, no more values can be sent
    pub fn close(&mut self) -> bool {
        let _ = self.sender.take();
        true
    }

    /// Check if the channel is closed
    pub fn is_closed(&self) -> bool {
        self.sender.is_none()
    }

    /// Get channel statistics
    /// Returns an array with: capacity, length, is_empty, is_full, is_closed
    pub fn stat(&self) -> Zval {
        let mut ht = ZendHashTable::new();
        ht.insert("capacity", self.capacity as i64).ok();
        ht.insert("length", self.length()).ok();
        ht.insert("is_empty", self.is_empty()).ok();
        ht.insert("is_full", self.is_full()).ok();
        ht.insert("is_closed", self.is_closed()).ok();
        ht.into_zval(false).unwrap_or_else(|_| Zval::new())
    }
}
// Internal Rust API (not exposed to PHP)
impl AsyncChannel {
    /// Get a clone of the sender for Rust-side use
    /// Returns the sender if the channel is not closed
    pub fn get_sender(&self) -> flume::Sender<Zval> {
        self.sender.as_ref()
            .expect("Cannot get sender: channel is closed")
            .clone()
    }

    /// Get a clone of the receiver for Rust-side use
    pub fn get_receiver(&self) -> flume::Receiver<Zval> {
        self.receiver.clone()
    }
}
