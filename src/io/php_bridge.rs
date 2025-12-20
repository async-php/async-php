/// PHP IO Bridge infrastructure
///
/// This module provides the bridge between PHP IO objects and Rust tokio traits
/// using channels for async communication

use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use std::io::{Error as IoError, ErrorKind, Result as IoResult};

use crate::channel::AsyncChannel;
use crate::util::tuple2;

// ==================== PHP IO Bridge ====================

/// Helper for bridging PHP IO method calls through dual channels
#[derive(Clone)]
pub(super) struct PhpIoBridge {
    request_tx: flume::Sender<Zval>,
    response_rx: flume::Receiver<Zval>,
}

pub(super) type PhpIoCallFuture = LocalBoxFuture<'static, IoResult<Zval>>;

pub(super) fn php_io_call_future(bridge: PhpIoBridge, method: String, args: Vec<Zval>) -> PhpIoCallFuture {
    async move { bridge.call(&method, args).await }.boxed_local()
}

impl PhpIoBridge {
    pub(super) fn new(request_channel: &AsyncChannel, response_channel: &AsyncChannel) -> Self {
        Self {
            request_tx: request_channel.get_sender(),
            response_rx: response_channel.get_receiver(),
        }
    }

    /// Build args array from Vec<Zval>
    fn build_args_array(args: Vec<Zval>) -> IoResult<Zval> {
        let mut args_ht = ZendHashTable::new();
        for (i, arg) in args.into_iter().enumerate() {
            args_ht.insert(i as i64, arg)
                .map_err(|_| IoError::new(ErrorKind::Other, "Failed to build args"))?;
        }
        args_ht.into_zval(false)
            .map_err(|_| IoError::new(ErrorKind::Other, "Failed to convert args"))
    }

    pub(super) async fn call(&self, method: &str, args: Vec<Zval>) -> IoResult<Zval> {
        // Build request: [method, args] using tuple2
        let args_zval = Self::build_args_array(args)?;
        let request = tuple2(method, args_zval);

        // Send request through request channel
        self.request_tx.send_async(request).await
            .map_err(|_| IoError::new(ErrorKind::BrokenPipe, "Send failed"))?;

        // Receive response from response channel
        self.response_rx.recv_async().await
            .map_err(|_| IoError::new(ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Send close command to terminate the spawned fiber
    pub(super) fn close_sync(&self) {
        // Build request: ['__close__', []] using tuple2
        let empty_args = ZendHashTable::new().into_zval(false).unwrap_or_else(|_| Zval::new());
        let request = tuple2("__close__", empty_args);

        // Try to send close command (best effort, ignore errors)
        let _ = self.request_tx.try_send(request);
    }
}
