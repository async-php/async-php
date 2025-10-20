use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};

// Zval is !Send. But our runtime is single-threaded (LocalSet).
// We wrap Zval in a struct that we assert is Send, ONLY for use within our local channel.
struct SendZval(Zval);
unsafe impl Send for SendZval {}

#[php_class]
pub struct AsyncChannel {
    sender: mpsc::UnboundedSender<SendZval>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<SendZval>>>,
}

#[php_impl]
impl AsyncChannel {
    pub fn __construct() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn send(&self, value: &Zval) -> bool {
        // We must clone the Zval (increment refcount/copy) to send it.
        let val = value.shallow_clone(); 
        self.sender.send(SendZval(val)).is_ok()
    }

    pub fn recv(&self) -> RustFuture {
        let rx = self.receiver.clone();
        let future = async move {
            let mut lock = rx.lock().unwrap();
            match lock.recv().await {
                Some(wrapper) => wrapper.0,
                None => Zval::new(), // Closed
            }
        };
        RustFuture::new(future)
    }
    
    pub fn close(&self) {
        // Dropping the sender closes the channel, but we hold one in `self`.
        // To strictly close, we might need a RefCell or Option.
        // For unbounded channel, we don't have an explicit close on Sender handle 
        // without dropping it.
        // For simplicity in this PoC, we rely on GC dropping the object.
    }
}
