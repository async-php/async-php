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

    /// Bind the server to a specific address
    #[php]
    pub fn bind(&mut self, addr: String) -> PhpResult<()> {
        // Create a TCP listener
        let listener = AsyncTcpListener::__construct(addr)?;
        self.listener = Some(listener);
        Ok(())
    }

    /// Add a request handler for a specific path pattern
    /// The handler can be a callable, an object implementing RequestHandler, or a class name
    #[php]
    pub fn handle(&mut self, pattern: String, handler: &Zval) -> PhpResult<()> {
        self.handlers.insert(pattern, handler.shallow_clone());
        Ok(())
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

    /// Start the server and begin accepting connections
    /// Returns a future that runs the server loop
    #[php]
    pub fn start(&self) -> PhpResult<Zval> {
        let server = self.clone();

        let fut = async move {
            if server.listener.is_none() {
                return Err(PhpException::default("Server not bound to any address. Call bind() first.".into()));
            }

            // In a real implementation, this would:
            // 1. Accept connections in a loop
            // 2. Parse HTTP requests from the stream
            // 3. Create HttpRequest objects with streaming bodies
            // 4. Call the appropriate handler
            // 5. Stream the response back

            // For now, just simulate running
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        };

        let rust_fut = RustFuture::new(fut.boxed());
        rust_fut.into_zval(false)
    }

    /// Process a single request (useful for testing)
    /// This allows PHP code to manually process a request without starting the server
    #[php]
    pub fn process_request(&self, request: &HttpRequest
    ) -> PhpResult<HttpResponse> {
        let path = request.get_uri();

        // Find matching handler
        for (pattern, handler) in &self.handlers {
            if Self::path_matches(pattern, &path) {
                return self.call_handler(request, handler);
            }
        }

        // Use default handler or return 404
        if let Some(ref default) = self.default_handler {
            return self.call_handler(request, default);
        }

        // Default 404 response
        Ok(HttpResponse::__construct(404))
    }

    /// Clone the server
    pub fn clone(&self) -> Self {
        Self {
            listener: self.listener.as_ref().map(|l| l.clone()),
            handlers: self.handlers.iter()
                .map(|(k, v)| (k.clone(), v.shallow_clone()))
                .collect(),
            default_handler: self.default_handler.as_ref().map(|h| h.shallow_clone()),
            config: self.config.clone(),
        }
    }
}

impl HttpServer {
    /// Check if a path matches a pattern (simplified implementation)
    fn path_matches(pattern: &str, path: &str) -> bool {
        // Simple exact match for now
        // In a real implementation, this would support:
        // - Path parameters (e.g., /users/{id})
        // - Wildcards (e.g., /static/*)
        // - Regular expressions
        pattern == path
    }

    /// Call a request handler
    fn call_handler(&self,
        request: &HttpRequest,
        handler: &Zval
    ) -> PhpResult<HttpResponse> {
        // First, check if it's a RequestHandler object
        if let Ok(handler_obj) = <dyn RequestHandler>::from_zval(handler) {
            return handler_obj.handle(request);
        }

        // Then check if it's a callable (function, closure, etc.)
        if let Ok(response_zval) = handler.try_call(vec![request]) {
            // Try to convert the response to HttpResponse
            if let Some(response) = HttpResponse::from_zval(&response_zval) {
                return Ok(response.clone());
            }
        }

        Err(PhpException::default("Handler must be a RequestHandler object or callable returning HttpResponse".into()))
    }
}

/// Router trait for handling complex routing
#[php_interface]
#[php(name = "Async\\Kernel\\Network\\Http\\Router")]
pub trait Router {
    /// Match a request path and return handler and path parameters
    fn match_path(&self,
        path: String,
        method: String
    ) -> PhpResult<RouterMatch>;
}

/// Router match result
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\RouterMatch")]
pub struct RouterMatch {
    /// The matched handler
    handler: Option<Zval>,
    /// Path parameters (e.g., {id: "123"})
    params: HashMap<String, String>,
}

#[php_impl]
impl RouterMatch {
    /// Create a new router match
    #[php(constructor)]
    pub fn __construct(
        handler: Option<Zval>,
        params: HashMap<String, String>
    ) -> Self {
        Self { handler, params }
    }

    /// Get the handler
    pub fn get_handler(&self) -> Option<Zval> {
        self.handler.as_ref().map(|h| h.shallow_clone())
    }

    /// Get path parameters
    pub fn get_params(&self) -> HashMap<String, String> {
        self.params.clone()
    }

    /// Get a specific parameter
    pub fn get_param(&self, name: String) -> Option<String> {
        self.params.get(&name).cloned()
    }
}