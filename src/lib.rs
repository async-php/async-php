use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::IntoZval; // Added import

mod future;
mod net;
mod fs;
mod http;
mod channel;
mod db;
mod time;
mod util;
mod quic; // Add quic module
mod logger; // Add logger mod

use future::RustFuture;
use net::{AsyncTcpListener, AsyncTcpStream, AsyncUdpSocket};
use fs::{AsyncFilesystem, AsyncFileHandle};
use http::{AsyncHttpServer, AsyncHttpRequest, AsyncHttpResponse}; // Updated http imports
use channel::AsyncChannel;
use db::{AsyncMySql, AsyncPgSql, AsyncMySqlTransaction, AsyncPgSqlTransaction};
use time::AsyncTime;
use quic::{AsyncQuicServer, AsyncQuicConnection}; // Import quic structs
use logger::AsyncLogger; // Import AsyncLogger

use std::ffi::CString;
use ext_php_rs::ffi::zend_function;

// Minimal FFI for Zend Function Table manipulation
mod zend_ffi {
    use std::ffi::{c_char, c_void};

    extern "C" {
        pub fn zend_hash_str_find(ht: *mut c_void, key: *const c_char, len: usize) -> *mut c_void;
        pub fn zend_hash_str_update(ht: *mut c_void, key: *const c_char, len: usize, data: *mut c_void) -> *mut c_void;
    }
}

#[php_function]
pub fn override_function(original: String, replacement: &Zval) -> bool {
    // We need to be very careful here. This is modifying the Zend Engine state globally.
    
    unsafe {
        // 1. Get EG(function_table)
        let eg = ext_php_rs::zend::ExecutorGlobals::get();
        
        // function_table is *mut zend_array. 
        let ht_ptr = eg.function_table as *mut std::ffi::c_void;
        
        let original_lower = original.to_lowercase();
        let key_c = CString::new(original_lower.clone()).unwrap();
        
        // Find original
        let original_ptr = zend_ffi::zend_hash_str_find(
            ht_ptr, 
            key_c.as_ptr(), 
            key_c.as_bytes().len()
        ) as *mut zend_function;

        if original_ptr.is_null() {
             return false;
        }
        
        // dbg!(std::mem::size_of::<zend_function>()); 
        // If size is 0, we can't swap!

        // Find replacement
        if !replacement.is_string() {
             return false;
        }
        let repl_name = replacement.string().unwrap();
        let repl_name_lower = repl_name.to_lowercase();
        let repl_c = CString::new(repl_name_lower).unwrap();
        
        let source_ptr = zend_ffi::zend_hash_str_find(
            ht_ptr, 
            repl_c.as_ptr(), 
            repl_c.as_bytes().len()
        ); // as *mut zend_function; // Don't need typed ptr for update
        
        if source_ptr.is_null() {
            return false; 
        }
        
        // OVERWRITE STRATEGY (Leaky/Double-free on shutdown risk, but simple)
        // We copy the data from source bucket to target bucket.
        let target_c = CString::new(original_lower).unwrap();
        zend_ffi::zend_hash_str_update(
            ht_ptr,
            target_c.as_ptr(),
            target_c.as_bytes().len(),
            source_ptr 
        );
        
        // We DO NOT delete the source.
        // This results in two entries pointing to the same function body.
        // Shutdown will likely double-free. 
        // But runtime should be stable.
        
        true
    }
}

pub(crate) async fn drive_fiber(fiber: Zval) -> PhpResult<()> {
    let mut current_val = fiber
        .try_call_method("start", vec![])
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
                let result = fut.await;
                let args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = vec![&result];
                current_val = fiber
                    .try_call_method("resume", args)
                    .map_err(|e| PhpException::default(format!("Fiber resume error: {}", e)))?;
            } else {
                return Err(PhpException::default("RustFuture was already awaited!".into()));
            }
        } else {
            return Err(PhpException::default(format!("Fiber suspended with unknown value. Type: {:?}", current_val.get_type())));
        }
    }
    Ok(())
}

#[php_function]
pub fn go(callable: &Zval) -> PhpResult<Zval> {
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
    
    tokio::task::spawn_local(async move {
            if let Err(e) = crate::drive_fiber(fiber_clone).await {
                tracing::error!("Spawned fiber failed: {:?}", e);
            }
    });
    
    Ok(fiber_zval)
}

#[php_function]
pub fn run(fiber: &mut Zval) -> PhpResult<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| PhpException::default(format!("Tokio Error: {}", e)))?;

    let local = tokio::task::LocalSet::new();

    local.block_on(&rt, async {
        let fiber_clone = fiber.shallow_clone();
        drive_fiber(fiber_clone).await
    })
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<RustFuture>()
        .class::<AsyncTcpListener>()
        .class::<AsyncTcpStream>()
        .class::<AsyncUdpSocket>()
        .class::<AsyncFilesystem>()
        .class::<AsyncFileHandle>()
        .class::<AsyncHttpServer>()
        .class::<AsyncHttpRequest>()
        .class::<AsyncHttpResponse>()
        .class::<AsyncQuicServer>()
        .class::<AsyncQuicConnection>()
        .class::<AsyncChannel>()
        .class::<AsyncMySql>()
        .class::<AsyncMySqlTransaction>()
        .class::<AsyncPgSql>()
        .class::<AsyncPgSqlTransaction>()
        .class::<AsyncTime>()
        .class::<AsyncLogger>() // Register AsyncLogger
        .function(wrap_function!(run))
        .function(wrap_function!(go))
        .function(wrap_function!(override_function))
}
