use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use tokio::sync::mpsc;
use crate::util::Shared;

// Zval is !Send.
struct SendZval(Zval);
unsafe impl Send for SendZval {}

#[php_class]
#[php(name = "Async\\Kernel\\Channel")]
pub struct AsyncChannel {
    sender: mpsc::UnboundedSender<SendZval>,
    receiver: Shared<mpsc::UnboundedReceiver<SendZval>>,
}

#[php_impl]
impl AsyncChannel {
    pub fn __construct() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: tx,
            receiver: Shared::new(rx),
        }
    }

    pub fn send(&self, value: &Zval) -> bool {
        let val = value.shallow_clone(); 
        self.sender.send(SendZval(val)).is_ok()
    }

    pub fn recv(&self) -> RustFuture {
        let rx = self.receiver.clone();
        let future = async move {
            match rx.get_mut().recv().await {
                Some(wrapper) => wrapper.0,
                None => Zval::new(),
            }
        };
        RustFuture::new(future)
    }
    
    pub fn close(&self) {
        // No-op, rely on drop
    }
}