use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::IntoZval; 
use futures::FutureExt;

mod future;
mod io;
mod net;
mod fs;
mod http;
mod channel;
mod db;
mod time;
mod util;
mod logger;
mod tls;

use future::RustFuture;
use net::{AsyncTcpListener, AsyncTcpStream, AsyncUdpSocket, AsyncUnixListener, AsyncUnixStream};
use fs::{AsyncFilesystem, AsyncFileHandle};
use channel::AsyncChannel;
use db::{AsyncMySql, AsyncPgSql, AsyncMySqlTransaction, AsyncPgSqlTransaction};
use time::AsyncTime;
use logger::AsyncLogger;
use tls::AsyncTlsStream;
use io::{
    PhpInterfaceReader, PhpInterfaceWriter, PhpInterfaceCloser,
    PhpInterfaceReaderAt, PhpInterfaceWriterAt, PhpInterfaceSeeker,
    PhpInterfaceReaderFrom, PhpInterfaceWriterTo,
    PhpInterfaceByteReader, PhpInterfaceByteScanner, PhpInterfaceStringReader
};

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
    let _ = rustls::crypto::ring::default_provider().install_default();
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
        .class::<AsyncUnixListener>()
        .class::<AsyncUnixStream>()
        .class::<AsyncTlsStream>()
        .class::<AsyncFilesystem>()
        .class::<AsyncFileHandle>()
        .class::<http::HttpRequest>()
        .class::<http::HttpResponse>()
        .class::<http::HttpResponseBody>()
        .class::<http::HttpClient>()
        .class::<AsyncChannel>()
        .class::<AsyncMySql>()
        .class::<AsyncMySqlTransaction>()
        .class::<AsyncPgSql>()
        .class::<AsyncPgSqlTransaction>()
        .class::<AsyncTime>()
        .class::<AsyncLogger>() // Register AsyncLogger
        .interface::<PhpInterfaceReader>()
        .interface::<PhpInterfaceWriter>()
        .interface::<PhpInterfaceCloser>()
        .interface::<PhpInterfaceReaderAt>()
        .interface::<PhpInterfaceWriterAt>()
        .interface::<PhpInterfaceSeeker>()
        .interface::<PhpInterfaceReaderFrom>()
        .interface::<PhpInterfaceWriterTo>()
        .interface::<PhpInterfaceByteReader>()
        .interface::<PhpInterfaceByteScanner>()
        .interface::<PhpInterfaceStringReader>()
        .function(wrap_function!(run))
        .function(wrap_function!(go))
}
