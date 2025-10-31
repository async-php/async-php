/// HTTP Server implementation supporting HTTP/1.1, HTTP/2, and HTTP/3

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::http::{HttpRequest, HttpResponseBody};
use crate::future::RustFuture;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use bytes::Bytes;
use http_body_util::Full;

/// HTTP Server supporting HTTP/1.1, HTTP/2, and HTTP/3
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpServer")]
pub struct HttpServer {
    protocol: String,
}

// SAFETY: Safe because runtime is single-threaded
unsafe impl Send for HttpServer {}
unsafe impl Sync for HttpServer {}

#[php_impl]
impl HttpServer {
    /// Create a new HTTP server
    #[php(constructor)]
    pub fn __construct() -> Self {
        Self {
            protocol: "http1".to_string(),
        }
    }

    /// Set protocol (http1, http2, http3)
    #[php]
    pub fn set_protocol(&mut self, protocol: String) {
        self.protocol = protocol;
    }

    /// Start listening on the given address with a request handler callback
    /// The callback receives HttpRequest and should return HttpResponse
    #[php]
    pub fn listen(&self, addr: String, handler: &mut Zval) -> RustFuture {
        let protocol = self.protocol.clone();
        let handler_clone = handler.shallow_clone();

        RustFuture::new(async move {
            let socket_addr: SocketAddr = addr.parse()
                .map_err(|e| format!("Invalid address: {}", e))?;

            match protocol.as_str() {
                "http1" => Self::serve_http1(socket_addr, handler_clone).await,
                "http2" => Self::serve_http2(socket_addr, handler_clone).await,
                "http3" => Self::serve_http3(socket_addr, handler_clone).await,
                _ => Err(format!("Unsupported protocol: {}", protocol)),
            }
        })
    }
}

impl HttpServer {
    /// Serve HTTP/1.1
    async fn serve_http1(addr: SocketAddr, handler: Zval) -> Result<Zval, String> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| format!("Failed to bind: {}", e))?;

        loop {
            let (stream, _) = listener.accept().await
                .map_err(|e| format!("Failed to accept: {}", e))?;

            let io = TokioIo::new(stream);
            let handler_clone = handler.shallow_clone();

            tokio::task::spawn_local(async move {
                let service = service_fn(move |req| {
                    let handler = handler_clone.shallow_clone();
                    async move {
                        Self::handle_request(req, handler).await
                    }
                });

                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                {
                    eprintln!("Error serving connection: {:?}", e);
                }
            });
        }
    }

    /// Serve HTTP/2
    /// TODO: HTTP/2 support requires Send-safe executor which conflicts with single-threaded PHP runtime
    async fn serve_http2(_addr: SocketAddr, _handler: Zval) -> Result<Zval, String> {
        Err("HTTP/2 support is not yet implemented due to executor constraints. Use http1 instead.".to_string())
    }

    /// Serve HTTP/3
    /// TODO: HTTP/3 support requires compatible versions of quinn and h3
    async fn serve_http3(_addr: SocketAddr, _handler: Zval) -> Result<Zval, String> {
        Err("HTTP/3 support is not yet implemented. Use http1 or http2 instead.".to_string())
    }

    /// Handle incoming HTTP request and call PHP handler
    async fn handle_request(
        req: hyper::Request<hyper::body::Incoming>,
        handler: Zval,
    ) -> Result<hyper::Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        // Extract request parts
        let (parts, body) = req.into_parts();

        // Create HttpRequest
        let mut http_request = HttpRequest::__construct(
            parts.method.to_string(),
            parts.uri.to_string(),
        );

        // Set version
        let version = match parts.version {
            hyper::Version::HTTP_09 => "0.9",
            hyper::Version::HTTP_10 => "1.0",
            hyper::Version::HTTP_11 => "1.1",
            hyper::Version::HTTP_2 => "2.0",
            hyper::Version::HTTP_3 => "3.0",
            _ => "1.1",
        };
        http_request.set_version(version.to_string());

        // Set headers
        for (key, value) in parts.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                http_request.set_header(key.to_string(), value_str.to_string());
            }
        }

        // Wrap request body
        let request_body = HttpResponseBody::new_internal(body);
        let body_zval = ext_php_rs::types::ZendClassObject::new(request_body)
            .into_zval(false)
            .map_err(|e| format!("Failed to create request body: {:?}", e))?;

        http_request.set_body(&body_zval)
            .map_err(|e| format!("Failed to set request body: {:?}", e))?;

        // Convert HttpRequest to Zval
        let request_zval = ext_php_rs::types::ZendClassObject::new(http_request)
            .into_zval(false)
            .map_err(|e| format!("Failed to convert request: {:?}", e))?;

        // Call PHP handler
        let response_zval = handler
            .try_call_method("handle", vec![&request_zval])
            .map_err(|e| format!("Handler error: {:?}", e))?;

        // Extract HttpResponse
        let response_obj = response_zval.object()
            .ok_or("Handler did not return an object")?;

        // Get response data using methods
        let status_code = response_obj
            .try_call_method("get_status_code", vec![])
            .ok()
            .and_then(|v| v.long())
            .unwrap_or(500) as u16;

        // Build hyper response
        let mut response_builder = hyper::Response::builder()
            .status(status_code);

        // Get headers
        if let Ok(headers_zval) = response_obj.try_call_method("get_headers", vec![]) {
            if let Some(headers_array) = headers_zval.array() {
                for (key, value) in headers_array.iter() {
                    let key_str = match key {
                        ext_php_rs::types::ArrayKey::Long(i) => i.to_string(),
                        ext_php_rs::types::ArrayKey::String(s) => s.to_string(),
                        ext_php_rs::types::ArrayKey::Str(s) => s.to_string(),
                    };

                    if let Some(v) = value.string() {
                        response_builder = response_builder.header(&key_str, v.as_str());
                    }
                }
            }
        }

        // Get response body
        let body_bytes = if let Ok(body_zval) = response_obj.try_call_method("get_body", vec![]) {
            if body_zval.is_null() {
                Bytes::new()
            } else if let Some(body_str) = body_zval.string() {
                // Body is already a string
                Bytes::from(body_str.to_string())
            } else {
                // Body is an object - try to read all content
                match body_zval.try_call_method("read_all", vec![]) {
                    Ok(read_future) => {
                        // Check if it's a RustFuture that we need to extract
                        if let Some(rust_future) = <&mut RustFuture as ext_php_rs::convert::FromZvalMut>::from_zval_mut(&mut read_future.shallow_clone()) {
                            if let Some(fut) = rust_future.take_inner() {
                                match fut.await {
                                    Ok(content_zval) => {
                                        if let Some(content_str) = content_zval.string() {
                                            Bytes::from(content_str.to_string())
                                        } else {
                                            Bytes::new()
                                        }
                                    }
                                    Err(_) => Bytes::new(),
                                }
                            } else {
                                Bytes::new()
                            }
                        } else if let Some(content_str) = read_future.string() {
                            // Direct string result
                            Bytes::from(content_str.to_string())
                        } else {
                            Bytes::new()
                        }
                    }
                    Err(_) => Bytes::new(),
                }
            }
        } else {
            Bytes::new()
        };

        let response = response_builder
            .body(Full::new(body_bytes))?;

        Ok(response)
    }
}
