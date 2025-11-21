use crate::util::Shared;
use crate::http::{HttpRequest, HttpResponse};
use hyper::{Request, Response, body::Incoming};
use http_body_util::{Full, BodyExt};
use bytes::Bytes;

#[cfg(feature = "hyper-server")]
mod server_types {
    use super::*;
    use hyper_util::server::conn::auto;
    use std::future::Future;
    use std::time::Duration;
    use ext_php_rs::prelude::*;
    use ext_php_rs::types::ZendHashTable;

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

    #[php_class]
    #[php(name = "Async\\Kernel\\Network\\Http\\ConnectionBuilder")]
    pub struct ConnectionBuilder {
        pub(super) inner: Shared<auto::Builder<LocalExecutor>>
    }

    unsafe impl Send for ConnectionBuilder {}
    unsafe impl Sync for ConnectionBuilder {}
}

#[cfg(feature = "hyper-server")]
pub use server_types::*;

#[cfg(feature = "hyper-server")]
#[php_impl]
impl ConnectionBuilder {
    /// Create a new connection builder
    #[php]
    pub fn __construct() -> Self {
        let builder = auto::Builder::new(LocalExecutor);
        Self {
            inner: Shared::new(builder),
        }
    }

    /// Only accepts HTTP/1
    #[php]
    pub fn http1_only(&mut self) -> PhpResult<()> {
        self.inner.get_mut().http1_only();
        Ok(())
    }

    /// Only accepts HTTP/2
    #[php]
    pub fn http2_only(&mut self) -> PhpResult<()> {
        self.inner.get_mut().http2_only();
        Ok(())
    }

    /// Returns true if this builder can serve HTTP/1.1 connections
    #[php]
    pub fn is_http1_available(&self) -> bool {
        self.inner.get_ref().is_http1_available()
    }

    /// Returns true if this builder can serve HTTP/2 connections
    #[php]
    pub fn is_http2_available(&self) -> bool {
        self.inner.get_ref().is_http2_available()
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
        let mut h1_builder = self.inner.get_mut().http1();

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
        let mut h2_builder = self.inner.get_mut().http2();

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

/// Convert HttpResponse to hyper Response
pub async fn http_response_to_hyper(mut response: HttpResponse) -> Result<Response<Full<Bytes>>, String> {
    use http_body_util::BodyExt;

    // Take the body from response
    let body = response.take_body()?;

    // Collect all bytes from the body
    let collected = body.collect().await.map_err(|e| e.to_string())?;
    let bytes = collected.to_bytes();

    // Get response parts (status, headers, etc.)
    let resp = response.inner.get_ref();
    let status = resp.status();
    let headers = resp.headers().clone();
    let version = resp.version();

    // Build hyper response with Full body
    let mut hyper_response = Response::new(Full::new(bytes));
    *hyper_response.status_mut() = status;
    *hyper_response.headers_mut() = headers;
    *hyper_response.version_mut() = version;

    Ok(hyper_response)
}
