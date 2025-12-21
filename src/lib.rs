use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::IntoZval; 
use futures::FutureExt;

#[php_const]
#[php(name = "ASYNC_READ")]
pub const IO_READ: i64 = 1;

#[php_const]
#[php(name = "ASYNC_WRITE")]
pub const IO_WRITE: i64 = 2;

#[php_const]
#[php(name = "ASYNC_SEEK")]
pub const IO_SEEK: i64 = 4;

#[php_const]
#[php(name = "ASYNC_BUF")]
pub const IO_BUF: i64 = 8;

mod future;
mod io;
mod bytes;
mod net;
mod fs;
mod http;
mod channel;
mod pdo;
mod curl;
mod redis;
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
use net::{AsyncTcpListener, AsyncTcpStream, AsyncTlsConfig, AsyncTlsStream, AsyncUdpSocket, AsyncUnixListener, AsyncUnixStream, AsyncQuicListener, AsyncQuicConnection, AsyncQuicStream, AsyncQuicRecvStream, AsyncQuicSendStream};
use fs::{AsyncFilesystem, AsyncFileHandle};
use http::HttpServer;
use channel::AsyncChannel;
use pdo::{AsyncPdoMySql, AsyncPdoPgSql, AsyncPdoMySqlTransaction, AsyncPdoPgSqlTransaction};
use curl::{CurlHandle, CurlMulti};
use redis::AsyncRedisClient;
use time::{AsyncTime, AsyncTicker};
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

/// PHP-exposed function to convert PDO DSN to SQLx URI
/// Returns: ['driver' => 'mysql'|'pgsql', 'uri' => 'mysql://...']
#[php_function]
pub fn pdo_dsn_to_sqlx(dsn: String, username: Option<String>, password: Option<String>) -> PhpResult<Zval> {
    // Parse scheme
    let colon_pos = dsn.find(':').ok_or_else(|| {
        PhpException::default(format!("Invalid DSN (missing scheme): {}", dsn))
    })?;

    let scheme = dsn[..colon_pos].to_lowercase();
    let rest = &dsn[colon_pos + 1..];

    if scheme != "mysql" && scheme != "pgsql" {
        return Err(PhpException::default(format!("Unsupported PDO driver: {}", scheme)));
    }

    // Parse DSN pairs
    let pairs = parse_dsn_pairs(rest);

    // Check for URI passthrough
    if let Some(uri) = pairs.get("__uri") {
        let mut final_uri = uri.clone();
        if scheme == "pgsql" {
            final_uri = final_uri.replace("pgsql://", "postgres://");
        }
        let mut result = ext_php_rs::types::ZendHashTable::new();
        result.insert("driver", scheme.as_str()).ok();
        result.insert("uri", final_uri).ok();
        return result.into_zval(false).map_err(|e| PhpException::default(format!("Zval conversion error: {:?}", e)));
    }

    // Extract connection parameters
    let host = pairs.get("host").cloned().unwrap_or_else(|| "localhost".to_string());
    let port = pairs.get("port").cloned();
    let dbname = pairs.get("dbname").cloned().unwrap_or_default();
    let user = username.or_else(|| pairs.get("user").cloned()).unwrap_or_default();
    let pass = password.or_else(|| pairs.get("password").or_else(|| pairs.get("pass")).cloned()).unwrap_or_default();

    // Build query parameters
    let mut query_params = std::collections::HashMap::new();
    for (k, v) in pairs.iter() {
        let lk = k.to_lowercase();
        if ["host", "port", "dbname", "user", "username", "password", "pass"].contains(&lk.as_str()) {
            continue;
        }
        let key = if scheme == "mysql" && lk == "unix_socket" {
            "socket".to_string()
        } else {
            lk
        };
        query_params.insert(key, v.clone());
    }

    // Build auth part
    let auth = if !user.is_empty() {
        let encoded_user = urlencoding::encode(&user);
        if !pass.is_empty() {
            format!("{}:{}@", encoded_user, urlencoding::encode(&pass))
        } else {
            format!("{}@", encoded_user)
        }
    } else {
        String::new()
    };

    // Build host part
    let mut host_part = host.clone();
    if scheme == "pgsql" && host.starts_with('/') {
        // Unix socket mode for PostgreSQL
        query_params.insert("host".to_string(), host);
        host_part = "localhost".to_string();
    }
    if let Some(p) = port {
        if !p.is_empty() {
            host_part = format!("{}:{}", host_part, p);
        }
    }

    // Build path and query string
    let path = if !dbname.is_empty() {
        format!("/{}", urlencoding::encode(&dbname))
    } else {
        String::new()
    };

    let qs = if !query_params.is_empty() {
        let pairs: Vec<String> = query_params.iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();
        format!("?{}", pairs.join("&"))
    } else {
        String::new()
    };

    // Build final URI
    let uri_scheme = if scheme == "pgsql" { "postgres" } else { "mysql" };
    let uri = format!("{}://{}{}{}{}", uri_scheme, auth, host_part, path, qs);

    let mut result = ext_php_rs::types::ZendHashTable::new();
    result.insert("driver", scheme.as_str()).ok();
    result.insert("uri", uri).ok();
    result.into_zval(false).map_err(|e| PhpException::default(format!("Zval conversion error: {:?}", e)))
}

fn parse_dsn_pairs(rest: &str) -> std::collections::HashMap<String, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return std::collections::HashMap::new();
    }

    // Check for URI passthrough
    if rest.contains("://") {
        let mut map = std::collections::HashMap::new();
        map.insert("__uri".to_string(), rest.to_string());
        return map;
    }

    let mut result = std::collections::HashMap::new();
    for chunk in rest.split(';') {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        if let Some(eq_pos) = chunk.find('=') {
            let k = chunk[..eq_pos].trim();
            let v = chunk[eq_pos + 1..].trim();
            result.insert(k.to_string(), v.to_string());
        } else {
            result.insert(chunk.to_string(), String::new());
        }
    }
    result
}

/// PHP-exposed function to normalize parameter keys
/// Removes leading ':' from string parameters
#[php_function]
pub fn pdo_normalize_param_key(param: &Zval) -> Zval {
    if let Some(s) = param.string() {
        let normalized = s.trim_start_matches(':');
        let mut result = Zval::new();
        result.set_string(normalized, false).ok();
        result
    } else if let Some(i) = param.long() {
        let mut result = Zval::new();
        result.set_long(i);
        result
    } else {
        param.shallow_clone()
    }
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
        .class::<AsyncQuicListener>()
        .class::<AsyncQuicConnection>()
        .class::<AsyncQuicStream>()
        .class::<AsyncQuicRecvStream>()
        .class::<AsyncQuicSendStream>()
        .class::<AsyncFilesystem>()
        .class::<AsyncFileHandle>()
        .class::<http::HttpClient>()
        .class::<http::HttpRequest>()
        .class::<http::HttpResponse>()
        .class::<HttpServer>()
        .class::<http::Http3Server>()
        .class::<AsyncChannel>()
        .class::<AsyncPdoMySql>()
        .class::<AsyncPdoMySqlTransaction>()
        .class::<AsyncPdoPgSql>()
        .class::<AsyncPdoPgSqlTransaction>()
        .class::<CurlHandle>() // cURL support
        .class::<CurlMulti>()
        .class::<AsyncRedisClient>() // Redis support
        .class::<AsyncTime>()
        .class::<AsyncTicker>() // Ticker for periodic operations
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
        .function(wrap_function!(pdo_dsn_to_sqlx))
        .function(wrap_function!(pdo_normalize_param_key))
        .constant(wrap_constant!(IO_READ))
        .constant(wrap_constant!(IO_WRITE))
        .constant(wrap_constant!(IO_SEEK))
        .constant(wrap_constant!(IO_BUF))
}
