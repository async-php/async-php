/// HTTP Server implementation

use std::collections::HashMap;
use std::sync::Arc;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::{IntoZval, FromZval, IntoZvalDyn};
use crate::future::RustFuture;
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{info, error};
use http_body_util::Full;
use bytes::Bytes;

#[php_class]
#[derive(Clone)]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpServer")]
pub struct HttpServer {
    config: ServerConfig,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub max_connections: usize,
    pub keep_alive_timeout: u64,
    pub enable_http2: bool,
    pub enable_http3: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_connections: 1000,
            keep_alive_timeout: 90,
            enable_http2: false,
            enable_http3: false,
        }
    }
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    pub fn with_config(config: ServerConfig) -> Self {
        Self { config }
    }
}

#[php_impl]
impl HttpServer {
    pub fn __construct() -> Self {
        Self::new()
    }

    pub fn with_host(&mut self, host: String
    ) -> Self {
        self.config.host = host;
        self.clone()
    }

    pub fn with_port(&mut self, port: u16
    ) -> Self {
        self.config.port = port;
        self.clone()
    }

    pub fn with_max_connections(&mut self, max: usize
    ) -> Self {
        self.config.max_connections = max;
        self.clone()
    }

    pub fn with_keep_alive_timeout(&mut self, timeout: u64
    ) -> Self {
        self.config.keep_alive_timeout = timeout;
        self.clone()
    }

    pub fn enable_http2(&mut self
    ) -> Self {
        self.config.enable_http2 = true;
        self.clone()
    }

    pub fn enable_http3(&mut self
    ) -> Self {
        self.config.enable_http3 = true;
        self.clone()
    }

    pub fn listen(&self, _handler: &Zval
    ) {
        // This is a synchronous method wrapper - actual async implementation
        // will be handled by the future-based system
        unimplemented!("Use async server implementation"
        );
    }

    /// Start server with handler
    pub fn start(&self, handler: &Zval
    ) -> RustFuture {
        let config = self.config.clone();
        // handler is now owned Zval via unsafe copy
        let handler: Zval = unsafe { std::ptr::read(handler) };
        let handler = Arc::new(handler);

        let future = async move {
            let addr = format!("{}:{}", config.host, config.port);

            let listener = match TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind to {}: {}", addr, e);
                    return Zval::new();
                }
            };

            info!("HTTP Server listening on http://{}", addr);

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let io = TokioIo::new(stream);
                let handler_clone = handler.clone();

                tokio::task::spawn_local(async move {
                    let service = service_fn(move |req| {
                        let handler_inner = handler_clone.clone();
                        async move {
                            // Convert hyper request to HttpRequest
                            let php_request = convert_hyper_request(req).await;

                            // Call PHP handler
                            let arg = php_request.into_zval(false).unwrap();
                            let args = vec![&arg as &dyn IntoZvalDyn];
                            
                            // handler_inner is Arc<Zval>. Deref gives Zval.
                            // We need to call methods on Zval.
                            
                            let callable = handler_inner.callable().ok_or("Handler is not callable").unwrap();
                            
                            let handler_result = callable.try_call(args).unwrap();

                            // Convert PHP response to hyper response
                            let hyper_response = convert_php_response(&handler_result);
                            hyper_response
                        }
                    });

                    if let Err(_) = http1::Builder::new().serve_connection(io, service).await {
                        error!("Error serving HTTP connection");
                    }
                });
            }

            #[allow(unreachable_code)]
            Zval::new()
        };

        RustFuture::new(future)
    }

    /// Get server configuration
    pub fn get_config(&self
    ) -> HashMap<String, Zval> {
        let mut config = HashMap::new();

        config.insert("host".to_string(), self.config.host.clone().into_zval(false).unwrap());
        config.insert("port".to_string(), (self.config.port as i64).into_zval(false).unwrap());
        config.insert("max_connections".to_string(), (self.config.max_connections as i64).into_zval(false).unwrap());
        config.insert("keep_alive_timeout".to_string(), (self.config.keep_alive_timeout).into_zval(false).unwrap());
        config.insert("enable_http2".to_string(), self.config.enable_http2.into_zval(false).unwrap());
        config.insert("enable_http3".to_string(), self.config.enable_http3.into_zval(false).unwrap());

        config
    }

    /// Check if HTTP/2 is enabled
    pub fn is_http2_enabled(&self
    ) -> bool {
        self.config.enable_http2
    }

    /// Check if HTTP/3 is supported
    pub fn is_http3_enabled(&self
    ) -> bool {
        self.config.enable_http3
    }
}

/// Convert hyper request to HttpRequest
async fn convert_hyper_request(
    req: Request<Incoming>
) -> HttpRequest {
    let (parts, _body) = req.into_parts();

    // Create method string
    let method = parts.method.to_string();
    let uri_str = parts.uri.to_string();

    // Create request
    let mut php_request = HttpRequest::new(method, uri_str);
    php_request.set_version("1.1".to_string()); // Default to 1.1

    // Copy headers
    for (name_opt, value) in parts.headers {
        if let Some(name) = name_opt {
             if let Ok(value_str) = value.to_str() {
                 php_request.with_header(name.to_string(), value_str.to_string());
             }
        }
    }

    // TODO: Handle body - skipping for now as it requires async reading and converting to HttpsBody
    // In a real implementation we would read the body stream.

    php_request
}

fn convert_php_response(zval: &Zval) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response_builder = Response::builder();
    let mut body_bytes = Bytes::new();

    // Use <&HttpResponse as FromZval>::from_zval(zval) to get a reference if possible, 
    // or if FromZval is implemented for &HttpResponse (returning Option<&HttpResponse>), this works.
    
    if let Some(php_resp) = <&HttpResponse as FromZval>::from_zval(zval) {
        // It's an HttpResponse object
        let arr = php_resp.to_array();
        
        // Status
        if let Some(s) = arr.get("status").and_then(|z| z.long()) {
            response_builder = response_builder.status(s as u16);
        }
        
        // Headers
        if let Some(headers) = arr.get("headers").and_then(|z| z.array()) {
            for (k, v) in headers {
                // Fixed key extraction with ArrayKey
                use ext_php_rs::types::ArrayKey;
                let key_str = match k {
                    ArrayKey::Long(i) => Some(i.to_string()),
                    ArrayKey::Str(s) => Some(s.to_string()),
                    _ => None,
                };

                if let (Some(key), Some(val)) = (key_str, v.string()) {
                    response_builder = response_builder.header(key, val);
                }
            }
        }
        
        if let Some(_body) = php_resp.get_body() {
             let s = php_resp.body_string();
             body_bytes = Bytes::from(s);
        }
        
    } else if let Some(s) = zval.string() {
        // It's a string
        response_builder = response_builder.status(200);
        body_bytes = Bytes::from(s.to_string());
    } else {
        // Unknown response
        response_builder = response_builder.status(500);
        body_bytes = Bytes::from("Internal Server Error: Invalid response from handler");
    }

    Ok(response_builder.body(Full::new(body_bytes)).unwrap())
}