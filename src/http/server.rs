/// HTTP Server implementation

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use crate::http::{HttpRequest, HttpResponse};
use crate::net::AsyncTcpListener;
use crate::future::RustFuture;
use futures::FutureExt;
use std::sync::Arc;
use std::collections::HashMap;

/// HTTP request handler trait
/// PHP implementations should implement this to handle requests
#[php_interface]
#[php(name = "Async\\Kernel\\Network\\Http\\RequestHandler")]
pub trait RequestHandler {
    /// Handle an HTTP request and return a response
    fn handle(&self, request: &HttpRequest) -> PhpResult<HttpResponse>;
}

/// HTTP Server
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpServer")]
pub struct HttpServer {
    /// The TCP listener for incoming connections
    listener: Option<AsyncTcpListener>,
    /// Request handlers by path pattern
    handlers: HashMap<String, Zval>, // Store PHP callbacks/handlers
    /// Default handler for 404 responses
    default_handler: Option<Zval>,
    /// Server configuration
    config: ServerConfig,
}

/// Server configuration
#[derive(Clone)]
struct ServerConfig {
    /// Read timeout in seconds
    read_timeout: Option<Duration>,
    /// Write timeout in seconds
    write_timeout: Option<Duration>,
    /// Keep-alive timeout
    keep_alive_timeout: Option<Duration>,
    /// Maximum request size in bytes
    max_request_size: usize,
    /// Server name for Server header
    server_name: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            read_timeout: Some(Duration::from_secs(60)),
            write_timeout: Some(Duration::from_secs(60)),
            keep_alive_timeout: Some(Duration::from_secs(30)),
            max_request_size: 10 * 1024 * 1024, // 10MB
            server_name: "async-php/1.0".to_string(),
        }
    }
}

use std::time::Duration;

#[php_impl]
impl HttpServer {
    /// Create a new HTTP server
    #[php(constructor)]
    pub fn __construct() -> Self {
        Self {
            listener: None,
            handlers: HashMap::new(),
            default_handler: None,
            config: ServerConfig::default(),
        }
    }

    /// Set the default handler for 404 responses
    #[php]
    pub fn set_default_handler(&mut self, handler: &Zval) -> PhpResult<()> {
        self.default_handler = Some(handler.shallow_clone());
        Ok(())
    }

    /// Set read timeout
    pub fn set_read_timeout(&mut self, seconds: i64) {
        self.config.read_timeout = if seconds > 0 {
            Some(Duration::from_secs(seconds as u64))
        } else {
            None
        };
    }

    /// Set write timeout
    pub fn set_write_timeout(&mut self, seconds: i64) {
        self.config.write_timeout = if seconds > 0 {
            Some(Duration::from_secs(seconds as u64))
        } else {
            None
        };
    }

    /// Set keep-alive timeout
    pub fn set_keep_alive_timeout(&mut self, seconds: i64) {
        self.config.keep_alive_timeout = if seconds > 0 {
            Some(Duration::from_secs(seconds as u64))
        } else {
            None
        };
    }

    /// Set maximum request size
    pub fn set_max_request_size(&mut self, size: i64) {
        self.config.max_request_size = size as usize;
    }
}
