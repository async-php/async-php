use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::exception::PhpException;
use std::future::Future;
use std::pin::Pin;

pub type DynFuture = Pin<Box<dyn Future<Output = Zval> + 'static>>;

#[php_class]
pub struct RustFuture {
    inner: Option<DynFuture>,
}

impl RustFuture {
    pub fn new<F>(future: F) -> Self 
    where 
        F: Future + 'static,
        F::Output: IntoZval,
    {
        Self {
            inner: Some(Box::pin(async move {
                let res = future.await;
                res.into_zval(false).unwrap_or_else(|_| Zval::new()) 
            })),
        }
    }

    pub fn take_inner(&mut self) -> Option<DynFuture> {
        self.inner.take()
    }
}

#[php_impl]
impl RustFuture {
    /// Simulates fetching data via FFI.
    pub fn ffi_fetch_data(id: i64) -> Self {
        let future = async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            format!("Fetched data for {}", id)
        };
        Self::new(future)
    }

    /// Spawns a new PHP Fiber.
    pub fn spawn(callable: &Zval) -> PhpResult<Zval> {
        let fiber_class = ClassEntry::try_find("Fiber")
            .ok_or_else(|| PhpException::default("Fiber class not found".into()))?;
        
        let fiber_obj = fiber_class.new();
        let fiber_zval = fiber_obj.into_zval(false).map_err(|e| {
            PhpException::default(format!("Failed to convert fiber object: {:?}", e))
        })?;

        fiber_zval.try_call_method("__construct", vec![callable]).map_err(|e| {
            PhpException::default(format!("Failed to construct Fiber: {:?}", e))
        })?;
        
        let fiber_clone = fiber_zval.shallow_clone();
        
        // Import drive_fiber from crate root via callback or public function?
        // To avoid cyclic deps, we assume the runtime handles the task logic locally
        // or we simply move the drive logic here.
        // For simplicity in this split, we keep the spawn logic simple:
        // It needs access to `drive_fiber`. We will export `drive_fiber` in lib.rs and make it public crate-wide.
        
        tokio::task::spawn_local(async move {
             if let Err(e) = crate::drive_fiber(fiber_clone).await {
                 eprintln!("Spawned fiber failed: {:?}", e);
             }
        });
        
        Ok(fiber_zval)
    }
}
