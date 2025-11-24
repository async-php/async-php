use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::IntoZval; 
use futures::FutureExt;

mod future;
mod io;
mod bytes;
mod net;
mod fs;
mod http;
mod channel;
mod pdo;
mod curl;
mod time;
mod util;
mod logger;
mod context;
mod sql_parser;

use future::RustFuture;
use io::{
    AsyncReader, AsyncWriter, AsyncSeeker, AsyncBufReader,
    AsyncReadWriter, AsyncReadSeeker, AsyncWriteSeeker, AsyncReadWriteSeeker,
};
use bytes::{BytesReader, BytesWriter};
use net::{AsyncTcpListener, AsyncTcpStream, AsyncTlsConfig, AsyncTlsStream, AsyncUdpSocket, AsyncUnixListener, AsyncUnixStream};
use fs::{AsyncFilesystem, AsyncFileHandle};
use http::HttpServer;
use channel::AsyncChannel;
use pdo::{AsyncPdoMySql, AsyncPdoPgSql, AsyncPdoMySqlTransaction, AsyncPdoPgSqlTransaction};
use curl::{CurlHandle, CurlMulti};
use time::AsyncTime;
use logger::AsyncLogger;
use context::AsyncContext;

// Export PHP IO bridge types for external use
pub use io::{
    PhpReader, PhpWriter, PhpSeeker, PhpBufReader,
    PhpReadWriter, PhpReadSeeker, PhpWriteSeeker, PhpReadWriteSeeker,
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
pub fn go(callable: &Zval) -> PhpResult<i64> {
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

    let fiber_id = crate::context::next_fiber_id() as i64;
    
    crate::context::spawn_local_with_fiber_id(fiber_id as u64, async move {
        if let Err(e) = crate::drive_fiber(fiber_clone).await {
            tracing::error!("Spawned fiber failed: {:?}", e);
        }
    });
    
    Ok(fiber_id)
}

#[php_function]
pub fn run(fiber: &mut Zval) -> PhpResult<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| PhpException::default(format!("Tokio Error: {}", e)))?;

    let local = tokio::task::LocalSet::new();

    local.block_on(&rt, crate::context::scope(async {
        let fiber_clone = fiber.shallow_clone();
        drive_fiber(fiber_clone).await
    }))
}

/// PHP-exposed function to compile SQL placeholders
/// Returns: ['sql' => rewritten_sql, 'placeholders' => [...]]
#[php_function]
pub fn sql_compile_placeholders(driver: String, sql: String) -> PhpResult<Zval> {
    let parsed = crate::sql_parser::parse_and_rewrite_sql(&sql, &driver)
        .map_err(|e| PhpException::default(e))?;

    // Build result array
    let mut result = ext_php_rs::types::ZendHashTable::new();

    // Add rewritten SQL
    result.insert("sql", parsed.rewritten).ok();

    // Build placeholders array
    let mut placeholders_array = ext_php_rs::types::ZendHashTable::new();
    for placeholder in parsed.placeholders {
        let mut ph_entry = ext_php_rs::types::ZendHashTable::new();
        match placeholder.kind {
            crate::sql_parser::PlaceholderKind::Positional(num) => {
                ph_entry.insert("kind", "pos").ok();
                ph_entry.insert("key", num as i64).ok();
            }
            crate::sql_parser::PlaceholderKind::Named(name) => {
                ph_entry.insert("kind", "named").ok();
                ph_entry.insert("key", name).ok();
            }
        }
        placeholders_array.push(ph_entry).ok();
    }

    result.insert("placeholders", placeholders_array).ok();

    result
        .into_zval(false)
        .map_err(|e| PhpException::default(format!("Zval conversion error: {:?}", e)))
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<AsyncContext>()
        .class::<RustFuture>()
        .class::<AsyncTcpListener>()
        .class::<AsyncTcpStream>()
        .class::<AsyncUdpSocket>()
        .class::<AsyncUnixListener>()
        .class::<AsyncUnixStream>()
        .class::<AsyncTlsConfig>()
        .class::<AsyncTlsStream>()
        .class::<AsyncFilesystem>()
        .class::<AsyncFileHandle>()
        .class::<http::HttpClient>()
        .class::<http::HttpRequest>()
        .class::<http::HttpResponse>()
        .class::<HttpServer>()
        .class::<AsyncChannel>()
        .class::<AsyncPdoMySql>()
        .class::<AsyncPdoMySqlTransaction>()
        .class::<AsyncPdoPgSql>()
        .class::<AsyncPdoPgSqlTransaction>()
        .class::<CurlHandle>() // cURL support
        .class::<CurlMulti>()
        .class::<AsyncTime>()
        .class::<AsyncLogger>() // Register AsyncLogger
        .class::<AsyncReader>() // IO types
        .class::<AsyncWriter>()
        .class::<AsyncSeeker>()
        .class::<AsyncBufReader>()
        .class::<AsyncReadWriter>() // Combined IO types
        .class::<AsyncReadSeeker>()
        .class::<AsyncWriteSeeker>()
        .class::<AsyncReadWriteSeeker>()
        .class::<BytesReader>() // In-memory IO
        .class::<BytesWriter>()
        .class::<PhpReader>() // PHP IO bridges
        .class::<PhpWriter>()
        .class::<PhpSeeker>()
        .class::<PhpBufReader>()
        .class::<PhpReadWriter>() // Combined PHP IO bridges
        .class::<PhpReadSeeker>()
        .class::<PhpWriteSeeker>()
        .class::<PhpReadWriteSeeker>()
        .function(wrap_function!(run))
        .function(wrap_function!(go))
        .function(wrap_function!(sql_compile_placeholders))
}
