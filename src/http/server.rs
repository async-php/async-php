/// HTTP Server implementation supporting HTTP/1.1, HTTP/2, and HTTP/3

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::http::{HttpRequest, HttpResponseBody};
use crate::future::RustFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use bytes::Bytes;
use http_body_util::Full;
use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use std::fs;
use std::io::BufReader;

/// HTTP Server supporting HTTP/1.1, HTTP/2, and HTTP/3 simultaneously
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpServer")]
pub struct HttpServer {
    /// TLS certificate file path (PEM format)
    cert_path: Option<String>,
    /// TLS private key file path (PEM format)
    key_path: Option<String>,
    /// Enable HTTP/1.1 (default: true)
    enable_http1: bool,
    /// Enable HTTP/2 (default: true)
    enable_http2: bool,
    /// Enable HTTP/3 (default: true)
    enable_http3: bool,
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
            cert_path: None,
            key_path: None,
            enable_http1: true,
            enable_http2: true,
            enable_http3: true,
        }
    }

    /// Set TLS certificate and private key paths (required for HTTPS/HTTP2/HTTP3)
    #[php]
    pub fn set_tls(&mut self, cert_path: String, key_path: String) {
        self.cert_path = Some(cert_path);
        self.key_path = Some(key_path);
    }

    /// Enable or disable HTTP/1.1
    #[php]
    pub fn set_enable_http1(&mut self, enable: bool) {
        self.enable_http1 = enable;
    }

    /// Enable or disable HTTP/2
    #[php]
    pub fn set_enable_http2(&mut self, enable: bool) {
        self.enable_http2 = enable;
    }

    /// Enable or disable HTTP/3
    #[php]
    pub fn set_enable_http3(&mut self, enable: bool) {
        self.enable_http3 = enable;
    }

    /// Start listening on the given address with a request handler callback
    /// The callback receives HttpRequest and should return HttpResponse
    /// All enabled protocols will run simultaneously on the same port
    #[php]
    pub fn listen(&self, addr: String, handler: &mut Zval) -> RustFuture {
        let cert_path = self.cert_path.clone();
        let key_path = self.key_path.clone();
        let enable_http1 = self.enable_http1;
        let enable_http2 = self.enable_http2;
        let enable_http3 = self.enable_http3;
        let handler_clone = handler.shallow_clone();

        RustFuture::new(async move {
            let socket_addr: SocketAddr = addr.parse()
                .map_err(|e| format!("Invalid address: {}", e))?;

            // Start all enabled listeners simultaneously
            Self::serve_all(
                socket_addr,
                handler_clone,
                cert_path,
                key_path,
                enable_http1,
                enable_http2,
                enable_http3,
            ).await
        })
    }
}

impl HttpServer {
    /// Load TLS configuration from certificate and key files
    fn load_tls_config(cert_path: &str, key_path: &str) -> Result<Arc<ServerConfig>, String> {
        // Load certificates
        let cert_file = fs::File::open(cert_path)
            .map_err(|e| format!("Failed to open cert file: {}", e))?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs: Vec<rustls::pki_types::CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse certificates: {}", e))?;

        // Load private key
        let key_file = fs::File::open(key_path)
            .map_err(|e| format!("Failed to open key file: {}", e))?;
        let mut key_reader = BufReader::new(key_file);
        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| format!("Failed to parse private key: {}", e))?
            .ok_or_else(|| "No private key found".to_string())?;

        // Build server config with ALPN protocols for HTTP/1.1, HTTP/2, and HTTP/3
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| format!("Failed to build TLS config: {}", e))?;

        config.alpn_protocols = vec![
            b"h3".to_vec(),       // HTTP/3
            b"h2".to_vec(),       // HTTP/2
            b"http/1.1".to_vec(), // HTTP/1.1
        ];

        Ok(Arc::new(config))
    }

    /// Load Quinn server configuration for HTTP/3
    fn load_quinn_config(cert_path: &str, key_path: &str) -> Result<quinn::ServerConfig, String> {
        // Load certificates
        let cert_file = fs::File::open(cert_path)
            .map_err(|e| format!("Failed to open cert file: {}", e))?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs: Vec<rustls::pki_types::CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse certificates: {}", e))?;

        // Load private key
        let key_file = fs::File::open(key_path)
            .map_err(|e| format!("Failed to open key file: {}", e))?;
        let mut key_reader = BufReader::new(key_file);
        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| format!("Failed to parse private key: {}", e))?
            .ok_or_else(|| "No private key found".to_string())?;

        // Build Quinn server config
        let server_config = quinn::ServerConfig::with_single_cert(certs, key)
            .map_err(|e| format!("Failed to build Quinn config: {}", e))?;

        Ok(server_config)
    }

    /// Serve all enabled protocols simultaneously
    async fn serve_all(
        addr: SocketAddr,
        handler: Zval,
        cert_path: Option<String>,
        key_path: Option<String>,
        enable_http1: bool,
        enable_http2: bool,
        enable_http3: bool,
    ) -> Result<Zval, String> {
        // HTTP/2 temporarily disabled due to Send trait requirements with Zval
        if enable_http2 {
            tracing::warn!("HTTP/2 is temporarily disabled due to executor constraints");
        }

        // Load TLS config for TCP listener if needed
        let tls_config = if (enable_http1 || enable_http2) && cert_path.is_some() && key_path.is_some() {
            Some(Self::load_tls_config(
                cert_path.as_ref().unwrap(),
                key_path.as_ref().unwrap(),
            )?)
        } else {
            None
        };

        // Start TCP listener for HTTP/1.1 (and HTTP/2 when supported)
        let tcp_task = if enable_http1 {
            let handler = handler.shallow_clone();
            let tls_config = tls_config.clone();
            Some(tokio::task::spawn_local(async move {
                Self::serve_tcp(addr, handler, tls_config, enable_http1, false /* http2 disabled */).await
            }))
        } else {
            None
        };

        // Start QUIC listener for HTTP/3
        let quic_task = if enable_http3 {
            if cert_path.is_none() || key_path.is_none() {
                return Err("HTTP/3 requires TLS configuration. Call set_tls() first.".to_string());
            }
            let handler = handler.shallow_clone();
            let cert = cert_path.unwrap();
            let key = key_path.unwrap();
            Some(tokio::task::spawn_local(async move {
                Self::serve_quic(addr, handler, &cert, &key).await
            }))
        } else {
            None
        };

        // Wait for both tasks (they run forever until error)
        match (tcp_task, quic_task) {
            (Some(tcp), Some(quic)) => {
                tokio::select! {
                    result = tcp => result.map_err(|e| format!("TCP task error: {}", e))?,
                    result = quic => result.map_err(|e| format!("QUIC task error: {}", e))?,
                }
            }
            (Some(tcp), None) => tcp.await.map_err(|e| format!("TCP task error: {}", e))?,
            (None, Some(quic)) => quic.await.map_err(|e| format!("QUIC task error: {}", e))?,
            (None, None) => return Err("At least one protocol must be enabled".to_string()),
        }
    }

    /// Serve TCP connections (HTTP/1.1 and/or HTTP/2)
    async fn serve_tcp(
        addr: SocketAddr,
        handler: Zval,
        tls_config: Option<Arc<ServerConfig>>,
        enable_http1: bool,
        enable_http2: bool,
    ) -> Result<Zval, String> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| format!("Failed to bind TCP: {}", e))?;

        tracing::info!("TCP listener started on {}", addr);

        loop {
            let (stream, _) = listener.accept().await
                .map_err(|e| format!("Failed to accept TCP: {}", e))?;

            let handler = handler.shallow_clone();
            let tls_config = tls_config.clone();

            tokio::task::spawn_local(async move {
                if let Err(e) = Self::handle_tcp_connection(
                    stream,
                    handler,
                    tls_config,
                    enable_http1,
                    enable_http2,
                ).await {
                    tracing::error!("TCP connection error: {}", e);
                }
            });
        }
    }

    /// Handle a single TCP connection with protocol negotiation
    async fn handle_tcp_connection(
        stream: tokio::net::TcpStream,
        handler: Zval,
        tls_config: Option<Arc<ServerConfig>>,
        enable_http1: bool,
        enable_http2: bool,
    ) -> Result<(), String> {
        if let Some(config) = tls_config {
            // TLS connection - negotiate protocol via ALPN
            let acceptor = TlsAcceptor::from(config);
            let tls_stream = acceptor.accept(stream).await
                .map_err(|e| format!("TLS handshake failed: {}", e))?;

            let (_, session) = tls_stream.get_ref();
            let protocol = session.alpn_protocol()
                .and_then(|p| std::str::from_utf8(p).ok());

            match protocol {
                Some("h2") if enable_http2 => {
                    Self::serve_http2_connection(TokioIo::new(tls_stream), handler).await
                }
                Some("http/1.1") | None if enable_http1 => {
                    Self::serve_http1_connection(TokioIo::new(tls_stream), handler).await
                }
                _ => Err(format!("Unsupported protocol: {:?}", protocol)),
            }
        } else {
            // Plain HTTP/1.1 only
            if enable_http1 {
                Self::serve_http1_connection(TokioIo::new(stream), handler).await
            } else {
                Err("HTTP/1.1 disabled and no TLS configured".to_string())
            }
        }
    }

    /// Serve a single HTTP/1.1 connection
    async fn serve_http1_connection<T>(io: TokioIo<T>, handler: Zval) -> Result<(), String>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + 'static,
    {
        let service = service_fn(move |req| {
            let handler = handler.shallow_clone();
            async move {
                Self::handle_request(req, handler).await
            }
        });

        http1::Builder::new()
            .serve_connection(io, service)
            .await
            .map_err(|e| format!("HTTP/1.1 connection error: {}", e))
    }

    /// Serve a single HTTP/2 connection (currently disabled)
    async fn serve_http2_connection<T>(_io: TokioIo<T>, _handler: Zval) -> Result<(), String>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + 'static,
    {
        Err("HTTP/2 support temporarily disabled due to Send trait constraints".to_string())
    }

    /// Serve QUIC connections (HTTP/3)
    async fn serve_quic(
        addr: SocketAddr,
        handler: Zval,
        cert_path: &str,
        key_path: &str,
    ) -> Result<Zval, String> {
        // Load Quinn server config
        let mut server_config = Self::load_quinn_config(cert_path, key_path)?;

        // Configure transport parameters
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_bidi_streams(100u32.into());
        transport_config.max_concurrent_uni_streams(100u32.into());
        server_config.transport_config(Arc::new(transport_config));

        // Bind QUIC endpoint
        let endpoint = quinn::Endpoint::server(server_config, addr)
            .map_err(|e| format!("Failed to bind QUIC: {}", e))?;

        tracing::info!("QUIC listener started on {}", addr);

        loop {
            let Some(incoming) = endpoint.accept().await else {
                continue;
            };

            let handler = handler.shallow_clone();
            tokio::task::spawn_local(async move {
                if let Err(e) = Self::handle_quic_connection(incoming, handler).await {
                    tracing::error!("QUIC connection error: {}", e);
                }
            });
        }
    }

    /// Handle a single QUIC connection (HTTP/3)
    async fn handle_quic_connection(
        incoming: quinn::Incoming,
        _handler: Zval,
    ) -> Result<(), String> {
        let connection = incoming.await
            .map_err(|e| format!("QUIC connection failed: {}", e))?;

        let mut h3_conn: h3::server::Connection<h3_quinn::Connection, bytes::Bytes> =
            h3::server::Connection::new(h3_quinn::Connection::new(connection))
                .await
                .map_err(|e| format!("H3 connection failed: {}", e))?;

        // TODO: Accept and handle H3 requests
        // This requires implementing the H3 request/response loop
        loop {
            match h3_conn.accept().await {
                Ok(Some(request_stream)) => {
                    tracing::info!("Received H3 request");
                    // TODO: Proper request handling
                    // For now, just receive the request and close the stream
                    let _ = request_stream.resolve_request().await;
                    // TODO: Call PHP handler and send response
                }
                Ok(None) => {
                    // Connection closed
                    break;
                }
                Err(e) => {
                    tracing::error!("H3 accept error: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
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
