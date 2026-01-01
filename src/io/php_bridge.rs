/// PHP IO Bridge infrastructure
///
/// This module provides the bridge between PHP IO objects and Rust tokio traits
/// by spawning nested Fibers.

use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::zend::ClassEntry;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use std::io::{Error as IoError, ErrorKind, Result as IoResult};

use crate::runtime::runtime::drive_fiber;

// ==================== PHP IO Bridge ====================

/// Helper for bridging PHP IO method calls
pub(super) struct PhpIoBridge {
    handler: Zval,
}

impl Clone for PhpIoBridge {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.shallow_clone(),
        }
    }
}

pub(super) type PhpIoCallFuture = LocalBoxFuture<'static, IoResult<Zval>>;

pub(super) fn php_io_call_future(bridge: PhpIoBridge, method: String, args: Vec<Zval>) -> PhpIoCallFuture {
    async move { bridge.call(&method, args).await }.boxed_local()
}

impl PhpIoBridge {
    pub(super) fn new(handler: Zval) -> Self {
        Self {
            handler,
        }
    }

    pub(super) async fn call(&self, method: &str, args: Vec<Zval>) -> IoResult<Zval> {
        let fiber_ce = ClassEntry::try_find("Fiber")
            .ok_or_else(|| IoError::new(ErrorKind::Other, "Fiber class not found"))?;

        let fiber_obj = fiber_ce.new();
        let fiber_zval = fiber_obj.into_zval(false)
             .map_err(|e| IoError::new(ErrorKind::Other, format!("Failed to create Fiber zval: {:?}", e)))?;

        // Construct callable: [$handler, $method]
        let mut callable_ht = ext_php_rs::types::ZendHashTable::new();
        callable_ht.push(self.handler.shallow_clone())
            .map_err(|e| IoError::new(ErrorKind::Other, format!("Failed to push handler to callable: {:?}", e)))?;
        callable_ht.push(method)
            .map_err(|e| IoError::new(ErrorKind::Other, format!("Failed to push method to callable: {:?}", e)))?;
            
        let callable = callable_ht.into_zval(false)
            .map_err(|e| IoError::new(ErrorKind::Other, format!("Failed to convert callable: {:?}", e)))?;

        fiber_zval.try_call_method("__construct", vec![&callable])
             .map_err(|e| IoError::new(ErrorKind::Other, format!("Failed to construct Fiber: {:?}", e)))?;

        // Convert args to trait objects
        let args_refs: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = args.iter()
            .map(|z| z as &dyn ext_php_rs::convert::IntoZvalDyn)
            .collect();

        drive_fiber(fiber_zval, args_refs).await
             .map_err(|e| IoError::new(ErrorKind::Other, format!("Fiber execution failed: {:?}", e)))
    }
    
    /// CloseSync is no longer needed/functional in this model as we don't hold a persistent background fiber.
    /// We keep the method signature for compatibility if needed, or we can remove it.
    /// The caller (PhpIo::drop) calls this. We'll leave it empty.
    pub(super) fn close_sync(&self) {
        // No-op
    }
}