use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::IntoZval;
use futures::FutureExt;

use crate::future::RustFuture;

pub(crate) async fn call_method_async(
    handler: Zval,
    method: &str,
    args: Vec<Zval>,
) -> PhpResult<Zval> {
    let fiber_ce = ClassEntry::try_find("Fiber")
        .ok_or_else(|| PhpException::default("Fiber class not found".into()))?;

    let fiber_obj = fiber_ce.new();
    let fiber_zval = fiber_obj.into_zval(false).map_err(|e| {
        PhpException::default(format!("Failed to create Fiber zval: {:?}", e))
    })?;

    // Construct callable: [$handler, $method]
    let mut callable = Vec::with_capacity(2);
    callable.push(handler);
    callable.push(
        method
            .into_zval(false)
            .map_err(|e| PhpException::default(format!("Failed to convert method: {:?}", e)))?,
    );

    let mut callable_ht = ext_php_rs::types::ZendHashTable::new();
    for item in callable {
        callable_ht.push(item).map_err(|e| {
            PhpException::default(format!("Failed to push to callable array: {:?}", e))
        })?;
    }
    let callable_zval = callable_ht.into_zval(false).map_err(|e| {
        PhpException::default(format!("Failed to convert callable to Zval: {:?}", e))
    })?;

    fiber_zval
        .try_call_method("__construct", vec![&callable_zval])
        .map_err(|e| {
            PhpException::default(format!("Failed to construct Fiber: {:?}", e))
        })?;

    let args_refs: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = args
        .iter()
        .map(|z| z as &dyn ext_php_rs::convert::IntoZvalDyn)
        .collect();

    drive_fiber(fiber_zval, args_refs).await
}

pub(crate) async fn call_closure_async(
    closure: Zval,
    args: Vec<Zval>,
) -> PhpResult<Zval> {
    let fiber_ce = ClassEntry::try_find("Fiber")
        .ok_or_else(|| PhpException::default("Fiber class not found".into()))?;

    let fiber_obj = fiber_ce.new();
    let fiber_zval = fiber_obj.into_zval(false).map_err(|e| {
        PhpException::default(format!("Failed to create Fiber zval: {:?}", e))
    })?;

    fiber_zval
        .try_call_method("__construct", vec![&closure])
        .map_err(|e| {
            PhpException::default(format!("Failed to construct Fiber: {:?}", e))
        })?;

    let args_refs: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = args
        .iter()
        .map(|z| z as &dyn ext_php_rs::convert::IntoZvalDyn)
        .collect();

    drive_fiber(fiber_zval, args_refs).await
}

pub(crate) async fn drive_fiber(fiber: Zval, args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn>) -> PhpResult<Zval> {
    let mut current_val = fiber
        .try_call_method("start", args)
        .map_err(|e| PhpException::default(format!("Fiber start error: {}", e)))?;

    loop {
        let is_terminated = fiber
            .try_call_method("isTerminated", vec![])
            .map_err(|e| PhpException::default(format!("Check terminated error: {}", e)))?
            .bool()
            .unwrap_or(false);

        if is_terminated {
            break;
        }

        if current_val.is_null() {
             tokio::task::yield_now().await;
             current_val = fiber
                .try_call_method("resume", vec![])
                .map_err(|e| PhpException::default(format!("Fiber resume error: {}", e)))?;
             continue;
        }

        if let Some(rust_fut) = <&mut RustFuture as ext_php_rs::convert::FromZvalMut>::from_zval_mut(&mut current_val) {
            if let Some(fut) = rust_fut.take_inner() {
                let catch_res = std::panic::AssertUnwindSafe(fut).catch_unwind().await;

                match catch_res {
                    Ok(exec_res) => {
                        match exec_res {
                            Ok(val) => {
                                let args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = vec![&val];
                                current_val = fiber
                                    .try_call_method("resume", args)
                                    .map_err(|e| PhpException::default(format!("Fiber resume error: {}", e)))?;
                            }
                            Err(err_msg) => {
                                let ex_ce = ClassEntry::try_find("Exception").ok_or_else(|| PhpException::default("Exception class not found".into()))?;
                                let ex_obj = ex_ce.new();
                                let ex_zval = ex_obj.into_zval(false).map_err(|e| PhpException::default(format!("Failed to convert exception to zval: {:?}", e)))?;
                                ex_zval.try_call_method("__construct", vec![&err_msg]).map_err(|e| PhpException::default(format!("Failed to construct exception: {:?}", e)))?;
                                
                                current_val = fiber
                                    .try_call_method("throw", vec![&ex_zval])
                                    .map_err(|e| PhpException::default(format!("Fiber throw error: {}", e)))?;
                            }
                        }
                    }
                    Err(payload) => {
                         let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                             format!("Rust Panic: {}", s)
                         } else if let Some(s) = payload.downcast_ref::<String>() {
                             format!("Rust Panic: {}", s)
                         } else {
                             "Rust Panic: Unknown payload".to_string()
                         };
                         
                         tracing::error!("{}", msg);
                         
                         let ex_ce = ClassEntry::try_find("Exception").ok_or_else(|| PhpException::default("Exception class not found".into()))?;
                         let ex_obj = ex_ce.new();
                         let ex_zval = ex_obj.into_zval(false).map_err(|e| PhpException::default(format!("Failed to convert exception to zval: {:?}", e)))?;
                         ex_zval.try_call_method("__construct", vec![&msg]).map_err(|e| PhpException::default(format!("Failed to construct exception: {:?}", e)))?;

                         current_val = fiber
                             .try_call_method("throw", vec![&ex_zval])
                             .map_err(|e| PhpException::default(format!("Fiber throw (panic) error: {}", e)))?;
                    }
                }
            } else {
                return Err(PhpException::default("RustFuture was already awaited!".into()));
            }
        } else {
            return Err(PhpException::default(format!("Fiber suspended with unknown value. Type: {:?}", current_val.get_type())));
        }
    }
    
    fiber
        .try_call_method("getReturn", vec![])
        .map_err(|e| PhpException::default(format!("Fiber getReturn error: {}", e)))
}

#[php_class]
#[php(name = "Async\\Kernel\\Runtime")]
pub struct AsyncRuntime;

#[php_impl]
impl AsyncRuntime {
    pub fn spawn(fiber: &mut Zval) -> PhpResult<i64> {
        let fiber_clone = fiber.shallow_clone();
        let fiber_id = crate::runtime::context::next_fiber_id() as i64;
        
        crate::runtime::context::spawn_local_with_fiber_id(fiber_id as u64, async move {
            if let Err(e) = drive_fiber(fiber_clone, vec![]).await {
                tracing::error!("Spawned fiber failed: {:?}", e);
            }
        });
        
        Ok(fiber_id)
    }

    pub fn run(fiber: &mut Zval) -> PhpResult<()> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| PhpException::default(format!("Tokio Error: {}", e)))?;

        let local = tokio::task::LocalSet::new();
        let _local_guard = crate::runtime::context::set_current_local_set(&local);

        local.block_on(&rt, crate::runtime::context::scope(async {
            let fiber_clone = fiber.shallow_clone();
            drive_fiber(fiber_clone, vec![]).await.map(|_| ())
        }))
    }
}
