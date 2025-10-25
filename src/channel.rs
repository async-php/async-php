use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use tokio::sync::mpsc;
use std::rc::Rc;
use std::cell::RefCell;

// Zval is !Send.
struct SendZval(Zval);
unsafe impl Send for SendZval {}

#[php_class]
#[php(name = "Async\\Driver\\Channel")]
pub struct AsyncChannel {
    sender: mpsc::UnboundedSender<SendZval>,
    receiver: Rc<RefCell<mpsc::UnboundedReceiver<SendZval>>>,
}

#[php_impl]
impl AsyncChannel {
    pub fn __construct() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: tx,
            receiver: Rc::new(RefCell::new(rx)),
        }
    }

    pub fn send(&self, value: &Zval) -> bool {
        let val = value.shallow_clone(); 
        self.sender.send(SendZval(val)).is_ok()
    }

    pub fn recv(&self) -> RustFuture {
        let rx = self.receiver.clone();
        let future = async move {
            if let Ok(mut lock) = rx.try_borrow_mut() {
                match lock.recv().await {
                    Some(wrapper) => wrapper.0,
                    None => Zval::new(), 
                }
            } else {
                Zval::new() // Busy
            }
        };
        RustFuture::new(future)
    }
    
    pub fn close(&self) {
        // No-op, rely on drop
    }
}