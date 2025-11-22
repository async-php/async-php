use crate::util::Shared;
use crate::http::{HttpRequest, HttpResponse};
use crate::io::{AsyncReadWriter, AsyncReadWrite};
use crate::future::RustFuture;
use hyper::{Request, Response, body::Incoming};
use http_body_util::BodyExt;
use bytes::Bytes;
use hyper_util::server::conn::auto;
use std::future::Future;
use std::time::Duration;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};
use ext_php_rs::convert::IntoZval;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct LocalExecutor;

// 实现 Hyper 的 Executor trait
impl<Fut> hyper::rt::Executor<Fut> for LocalExecutor
where
    Fut: Future + 'static, // 注意：这里没有 Send 约束！
{
    fn execute(&self, fut: Fut) {
        // 使用 spawn_local 而不是 spawn
        tokio::task::spawn_local(fut);
    }
}

/// Adapter to convert Shared<Box<dyn AsyncReadWrite>> to tokio::io traits
struct SharedIoAdapter {
    inner: Shared<Box<dyn AsyncReadWrite>>,
}

impl tokio::io::AsyncRead for SharedIoAdapter {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.get_mut().inner.get_mut()).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for SharedIoAdapter {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut *self.get_mut().inner.get_mut()).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.get_mut().inner.get_mut()).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut *self.get_mut().inner.get_mut()).poll_shutdown(cx)
    }
}

/// PHP Handler Service - wraps a PHP callable for HTTP request handling
#[derive(Clone)]
struct PhpHandlerService {
    /// PHP callable stored as Zval
    handler: std::sync::Arc<std::sync::Mutex<Zval>>,
}

impl hyper::service::Service<Request<Incoming>> for PhpHandlerService {
    type Response = Response<http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let handler = self.handler.clone();

        Box::pin(async move {
            // Helper to create errors
            let make_error = |msg: String| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg))
            };

            // Convert hyper Request to HttpRequest
            let http_request = http_request_from_hyper(req).await
                .map_err(|e| {
                    eprintln!("Failed to convert request: {}", e);
                    make_error(e)
                })?;

            // Call PHP handler with HttpRequest
            let response = {
                let handler_guard = handler.lock().unwrap();

                // Convert HttpRequest to Zval
                let req_zval = ext_php_rs::types::ZendClassObject::new(http_request)
                    .into_zval(false)
                    .map_err(|e| {
                        eprintln!("Failed to convert HttpRequest to Zval: {:?}", e);
                        make_error(format!("{:?}", e))
                    })?;

                // Call the PHP handler
                handler_guard.try_call(vec![&req_zval])
                    .map_err(|e| {
                        eprintln!("Failed to call PHP handler: {:?}", e);
                        make_error(format!("{:?}", e))
                    })?
            };

            // Extract HttpResponse reference from Zval
            let http_response_ref: &HttpResponse = response
                .extract()
                .ok_or_else(|| {
                    eprintln!("Handler did not return HttpResponse");
                    make_error("Handler did not return HttpResponse".to_string())
                })?;

            // Clone HttpResponse for conversion (we need ownership)
            // Note: This clone is needed because http_response_to_hyper takes ownership
            // In future, we could optimize this by modifying http_response_to_hyper
            let http_response = HttpResponse {
                inner: http_response_ref.inner.clone(),
            };

            // Convert HttpResponse to hyper Response
            let hyper_response = http_response_to_hyper(http_response).await
                .map_err(|e| {
                    eprintln!("Failed to convert response: {}", e);
                    make_error(e)
                })?;

            Ok(hyper_response)
        })
    }
}



#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\ConnectionBuilder")]
pub struct HttpServer {
    pub(super) conn_builder: Shared<auto::Builder<LocalExecutor>>
}

unsafe impl Send for HttpServer {}
unsafe impl Sync for HttpServer {}

#[php_impl]
impl HttpServer {
    /// Create a new connection builder
    #[php]
    pub fn __construct() -> Self {
        let builder = auto::Builder::new(LocalExecutor);
        Self {
            conn_builder: Shared::new(builder),
        }
    }

    /// Only accepts HTTP/1
    #[php]
    pub fn http1_only(&mut self) -> PhpResult<()> {
        let builder = std::mem::replace(
            self.conn_builder.get_mut(),
            auto::Builder::new(LocalExecutor)
        );
        *self.conn_builder.get_mut() = builder.http1_only();
        Ok(())
    }

    /// Only accepts HTTP/2
    #[php]
    pub fn http2_only(&mut self) -> PhpResult<()> {
        let builder = std::mem::replace(
            self.conn_builder.get_mut(),
            auto::Builder::new(LocalExecutor)
        );
        *self.conn_builder.get_mut() = builder.http2_only();
        Ok(())
    }

    /// Returns true if this builder can serve HTTP/1.1 connections
    #[php]
    pub fn is_http1_available(&self) -> bool {
        self.conn_builder.get_ref().is_http1_available()
    }

    /// Returns true if this builder can serve HTTP/2 connections
    #[php]
    pub fn is_http2_available(&self) -> bool {
        self.conn_builder.get_ref().is_http2_available()
    }

    // ==================== HTTP/1 Configuration ====================

    /// Configure HTTP/1 settings
    ///
    /// Available options:
    /// - 'auto_date_header' => bool - Automatically add Date header (default: true)
    /// - 'half_close' => bool - Support half-closures (default: false)
    /// - 'keep_alive' => bool - Enable keep-alive (default: true)
    /// - 'title_case_headers' => bool - Use title case for headers (default: false)
    /// - 'ignore_invalid_headers' => bool - Silently ignore malformed headers (default: false)
    /// - 'preserve_header_case' => bool - Preserve original header case (default: false)
    /// - 'max_headers' => int - Maximum number of headers (default: 100)
    /// - 'header_read_timeout' => float|null - Timeout for reading headers in seconds
    /// - 'writev' => bool - Use vectored writes (default: auto)
    /// - 'max_buf_size' => int - Maximum buffer size in bytes (default: ~400kb)
    /// - 'pipeline_flush' => bool - Aggregate flushes for pipelining (default: false, experimental)
    #[php]
    pub fn http1(&mut self, options: &ZendHashTable) -> PhpResult<()> {
        let mut h1_builder = self.conn_builder.get_mut().http1();

        // auto_date_header
        if let Some(val) = options.get("auto_date_header") {
            if let Some(enabled) = val.bool() {
                h1_builder.auto_date_header(enabled);
            }
        }

        // half_close
        if let Some(val) = options.get("half_close") {
            if let Some(enabled) = val.bool() {
                h1_builder.half_close(enabled);
            }
        }

        // keep_alive
        if let Some(val) = options.get("keep_alive") {
            if let Some(enabled) = val.bool() {
                h1_builder.keep_alive(enabled);
            }
        }

        // title_case_headers
        if let Some(val) = options.get("title_case_headers") {
            if let Some(enabled) = val.bool() {
                h1_builder.title_case_headers(enabled);
            }
        }

        // ignore_invalid_headers
        if let Some(val) = options.get("ignore_invalid_headers") {
            if let Some(enabled) = val.bool() {
                h1_builder.ignore_invalid_headers(enabled);
            }
        }

        // preserve_header_case
        if let Some(val) = options.get("preserve_header_case") {
            if let Some(enabled) = val.bool() {
                h1_builder.preserve_header_case(enabled);
            }
        }

        // max_headers
        if let Some(val) = options.get("max_headers") {
            if let Some(max) = val.long() {
                h1_builder.max_headers(max as usize);
            }
        }

        // header_read_timeout
        if let Some(val) = options.get("header_read_timeout") {
            if val.is_null() {
                h1_builder.header_read_timeout(None);
            } else if let Some(seconds) = val.double() {
                h1_builder.header_read_timeout(Some(Duration::from_secs_f64(seconds)));
            }
        }

        // writev
        if let Some(val) = options.get("writev") {
            if let Some(enabled) = val.bool() {
                h1_builder.writev(enabled);
            }
        }

        // max_buf_size
        if let Some(val) = options.get("max_buf_size") {
            if let Some(size) = val.long() {
                h1_builder.max_buf_size(size as usize);
            }
        }

        // pipeline_flush
        if let Some(val) = options.get("pipeline_flush") {
            if let Some(enabled) = val.bool() {
                h1_builder.pipeline_flush(enabled);
            }
        }

        Ok(())
    }

    // ==================== HTTP/2 Configuration ====================

    /// Configure HTTP/2 settings
    ///
    /// Available options:
    /// - 'max_pending_accept_reset_streams' => int|null - Max pending reset streams (default: 20)
    /// - 'max_local_error_reset_streams' => int|null - Max local error reset streams (default: 1024)
    /// - 'initial_stream_window_size' => int|null - Initial stream window size (default: 65535)
    /// - 'initial_connection_window_size' => int|null - Initial connection window size (default: 65535)
    /// - 'adaptive_window' => bool - Use adaptive flow control (default: false)
    /// - 'max_frame_size' => int|null - Maximum frame size (default: 16384, max: 16777215)
    /// - 'max_concurrent_streams' => int|null - Max concurrent streams (default: 200, null = unlimited)
    /// - 'keep_alive_interval' => float|null - Keep-alive ping interval in seconds
    /// - 'keep_alive_timeout' => float - Keep-alive ping timeout in seconds (default: 20)
    /// - 'max_send_buf_size' => int - Max write buffer size per stream (default: ~400KB)
    /// - 'enable_connect_protocol' => bool - Enable extended CONNECT protocol (RFC 8441)
    /// - 'max_header_list_size' => int - Max header list size (default: ~16MB)
    /// - 'auto_date_header' => bool - Automatically add Date header (default: true)
    #[php]
    pub fn http2(&mut self, options: &ZendHashTable) -> PhpResult<()> {
        let mut h2_builder = self.conn_builder.get_mut().http2();

        // max_pending_accept_reset_streams
        if let Some(val) = options.get("max_pending_accept_reset_streams") {
            if val.is_null() {
                h2_builder.max_pending_accept_reset_streams(None);
            } else if let Some(max) = val.long() {
                h2_builder.max_pending_accept_reset_streams(Some(max as usize));
            }
        }

        // max_local_error_reset_streams
        if let Some(val) = options.get("max_local_error_reset_streams") {
            if val.is_null() {
                h2_builder.max_local_error_reset_streams(None);
            } else if let Some(max) = val.long() {
                h2_builder.max_local_error_reset_streams(Some(max as usize));
            }
        }

        // initial_stream_window_size
        if let Some(val) = options.get("initial_stream_window_size") {
            if val.is_null() {
                h2_builder.initial_stream_window_size(None);
            } else if let Some(size) = val.long() {
                h2_builder.initial_stream_window_size(Some(size as u32));
            }
        }

        // initial_connection_window_size
        if let Some(val) = options.get("initial_connection_window_size") {
            if val.is_null() {
                h2_builder.initial_connection_window_size(None);
            } else if let Some(size) = val.long() {
                h2_builder.initial_connection_window_size(Some(size as u32));
            }
        }

        // adaptive_window
        if let Some(val) = options.get("adaptive_window") {
            if let Some(enabled) = val.bool() {
                h2_builder.adaptive_window(enabled);
            }
        }

        // max_frame_size
        if let Some(val) = options.get("max_frame_size") {
            if val.is_null() {
                h2_builder.max_frame_size(None);
            } else if let Some(size) = val.long() {
                h2_builder.max_frame_size(Some(size as u32));
            }
        }

        // max_concurrent_streams
        if let Some(val) = options.get("max_concurrent_streams") {
            if val.is_null() {
                h2_builder.max_concurrent_streams(None);
            } else if let Some(max) = val.long() {
                h2_builder.max_concurrent_streams(Some(max as u32));
            }
        }

        // keep_alive_interval
        if let Some(val) = options.get("keep_alive_interval") {
            if val.is_null() {
                h2_builder.keep_alive_interval(None);
            } else if let Some(seconds) = val.double() {
                h2_builder.keep_alive_interval(Some(Duration::from_secs_f64(seconds)));
            }
        }

        // keep_alive_timeout
        if let Some(val) = options.get("keep_alive_timeout") {
            if let Some(seconds) = val.double() {
                h2_builder.keep_alive_timeout(Duration::from_secs_f64(seconds));
            }
        }

        // max_send_buf_size
        if let Some(val) = options.get("max_send_buf_size") {
            if let Some(size) = val.long() {
                h2_builder.max_send_buf_size(size as usize);
            }
        }

        // enable_connect_protocol
        if let Some(val) = options.get("enable_connect_protocol") {
            if let Some(true) = val.bool() {
                h2_builder.enable_connect_protocol();
            }
        }

        // max_header_list_size
        if let Some(val) = options.get("max_header_list_size") {
            if let Some(size) = val.long() {
                h2_builder.max_header_list_size(size as u32);
            }
        }

        // auto_date_header
        if let Some(val) = options.get("auto_date_header") {
            if let Some(enabled) = val.bool() {
                h2_builder.auto_date_header(enabled);
            }
        }

        Ok(())
    }

    /// Serve HTTP requests on a connection with zero-copy IO optimization
    ///
    /// # Parameters
    /// - `io`: AsyncReadWriter - The connection IO (from TcpStream, UnixStream, TlsStream, etc.)
    /// - `handler`: PHP callable - Function to handle requests, signature: fn(HttpRequest): HttpResponse
    ///
    /// # Example (Go-style)
    /// ```php
    /// $server = new ConnectionBuilder();
    /// $listener = TcpListener::bind('127.0.0.1:8080');
    /// while (true) {
    ///     $conn = $listener->accept();
    ///     $server->serve($conn->asReadWriter(), function($req) {
    ///         $resp = new HttpResponse();
    ///         $resp->setStatus(200);
    ///         $resp->setBody("Hello World");
    ///         return $resp;
    ///     });
    /// }
    /// ```
    #[php]
    pub fn serve(&self, io: &AsyncReadWriter, handler: &mut Zval) -> RustFuture {
        // Clone the connection builder for this connection
        let builder = self.conn_builder.clone();

        // Extract the inner tokio IO (zero-copy!)
        let io_inner = io.get_inner();

        // Clone the handler Zval for the service
        let handler_clone = handler.shallow_clone();

        // Wrap the PHP handler in a service
        let service = PhpHandlerService {
            handler: std::sync::Arc::new(std::sync::Mutex::new(handler_clone)),
        };

        let future = async move {
            // Get the builder and serve the connection
            let result = builder.get_ref()
                .serve_connection(
                    hyper_util::rt::TokioIo::new(SharedIoAdapter { inner: io_inner }),
                    service
                )
                .await;

            match result {
                Ok(_) => {
                    let mut z = Zval::new();
                    z.set_bool(true);
                    Ok::<Zval, String>(z)
                }
                Err(e) => Err(format!("Connection error: {}", e))
            }
        };

        RustFuture::new(future)
    }
}

/// Convert hyper Request to HttpRequest
pub async fn http_request_from_hyper(req: Request<Incoming>) -> Result<HttpRequest, String> {
    use futures::TryStreamExt;
    use http_body::Frame;
    use http_body_util::StreamBody;
    use sync_wrapper::SyncStream;
    use std::io;

    // Extract request parts
    let (parts, body) = req.into_parts();

    // Convert Incoming body to BoxBody
    let stream = body
        .into_data_stream()
        .map_ok(Frame::data)
        .map_err(|e| Box::new(io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send>);

    let boxed_body = StreamBody::new(SyncStream::new(stream)).boxed();

    // Reconstruct request with converted body
    let http_request = http::Request::from_parts(parts, boxed_body);

    Ok(HttpRequest {
        inner: Shared::new(http_request),
    })
}

/// Convert HttpResponse to hyper Response with streaming body
pub async fn http_response_to_hyper(
    mut response: HttpResponse,
) -> Result<Response<http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>>, String> {
    // Take the body from response (keeps it streaming)
    let body = response.take_body()?;

    // Get response parts (status, headers, etc.) before consuming
    let resp = response.inner.get_ref();
    let status = resp.status();
    let headers = resp.headers().clone();
    let version = resp.version();

    // Build hyper response with streaming BoxBody
    let mut hyper_response = Response::new(body);
    *hyper_response.status_mut() = status;
    *hyper_response.headers_mut() = headers;
    *hyper_response.version_mut() = version;

    Ok(hyper_response)
}
