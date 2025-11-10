use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::sync::mpsc;
use crate::util::Shared;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[php_class]
#[php(name = "Async\\Kernel\\Channel")]
pub struct AsyncChannel {
    sender: Shared<mpsc::Sender<Zval>>,
    receiver: Shared<mpsc::Receiver<Zval>>,
    capacity: usize,
    current_len: Arc<AtomicUsize>,
    is_closed: Arc<AtomicBool>,
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
        let cap = capacity.unwrap_or(0).max(0) as usize;
        let (tx, rx) = mpsc::channel(cap);

        Self {
            sender: Shared::new(tx),
            receiver: Shared::new(rx),
            capacity: cap,
            current_len: Arc::new(AtomicUsize::new(0)),
            is_closed: Arc::new(AtomicBool::new(false)),
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
    pub fn send(&self, value: &Zval, timeout: Option<f64>) -> RustFuture {
        if self.is_closed.load(Ordering::Relaxed) {
            let future = async { Zval::from(false) };
            return RustFuture::new(future);
        }

        let val = value.shallow_clone();
        let tx = self.sender.clone();
        let current_len = self.current_len.clone();
        let is_closed = self.is_closed.clone();

        let future = async move {
            if is_closed.load(Ordering::Relaxed) {
                return Zval::from(false);
            }

            let sender = tx.get_mut();
            let result = if let Some(timeout_secs) = timeout {
                let duration = Duration::from_secs_f64(timeout_secs);
                match tokio::time::timeout(duration, sender.send(val)).await {
                    Ok(Ok(_)) => {
                        current_len.fetch_add(1, Ordering::Relaxed);
                        true
                    }
                    _ => false,
                }
            } else {
                match sender.send(val).await {
                    Ok(_) => {
                        current_len.fetch_add(1, Ordering::Relaxed);
                        true
                    }
                    Err(_) => false,
                }
            };

            Zval::from(result)
        };

        RustFuture::new(future)
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
    pub fn push(&self, value: &Zval, timeout: Option<f64>) -> RustFuture {
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
        let current_len = self.current_len.clone();

        let future = async move {
            let receiver = rx.get_mut();
            let result = if let Some(timeout_secs) = timeout {
                let duration = Duration::from_secs_f64(timeout_secs);
                match tokio::time::timeout(duration, receiver.recv()).await {
                    Ok(Some(val)) => {
                        current_len.fetch_sub(1, Ordering::Relaxed);
                        val
                    }
                    _ => Zval::new(),
                }
            } else {
                match receiver.recv().await {
                    Some(val) => {
                        current_len.fetch_sub(1, Ordering::Relaxed);
                        val
                    }
                    None => Zval::new(),
                }
            };

            result
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
        self.current_len.load(Ordering::Relaxed) == 0
    }

    /// Check if the channel is full
    pub fn is_full(&self) -> bool {
        if self.capacity == 0 {
            // Unbuffered channel is always "full" if no receiver is waiting
            false
        } else {
            self.current_len.load(Ordering::Relaxed) >= self.capacity
        }
    }

    /// Get the current number of items in the channel
    pub fn length(&self) -> i64 {
        self.current_len.load(Ordering::Relaxed) as i64
    }

    /// Close the channel
    /// After closing, no more values can be sent
    pub fn close(&self) -> bool {
        self.is_closed.store(true, Ordering::Relaxed);
        true
    }

    /// Check if the channel is closed
    pub fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Relaxed)
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
    pub fn get_sender(&self) -> Shared<mpsc::Sender<Zval>> {
        self.sender.clone()
    }

    /// Get a clone of the receiver for Rust-side use
    pub fn get_receiver(&self) -> Shared<mpsc::Receiver<Zval>> {
        self.receiver.clone()
    }
}
