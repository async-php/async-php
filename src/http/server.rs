use crate::util::Shared;
use crate::http::{HttpRequest, HttpResponse};
use crate::io::{AsyncReadWriter, AsyncReadWrite};
use crate::future::RustFuture;
use crate::http::socketio::AsyncSocketIo;
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
use socketioxide::layer::SocketIoLayer;
use tower::Layer;

#[derive(Clone)]
pub struct LocalExecutor;

// Implement Hyper's Executor trait
impl<Fut> hyper::rt::Executor<Fut> for LocalExecutor
where
    Fut: Future + 'static,
{
    fn execute(&self, fut: Fut) {
        crate::runtime::context::spawn_local(fut);
    }
}

/// Adapter to convert Shared<Box<dyn AsyncReadWrite + Unpin + Send>> to tokio::io traits
struct SharedIoAdapter {
    inner: Shared<Box<dyn AsyncReadWrite + Unpin + Send>>,
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
    type Response = Response<http_body_util::combinators::BoxBody<Bytes, std::io::Error>>;
    type Error = std::io::Error;
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let handler = self.handler.clone();

        Box::pin(async move {
            let make_error = |msg: String| -> std::io::Error {
                std::io::Error::new(std::io::ErrorKind::Other, msg)
            };

            let http_request = http_request_from_hyper(req).await
                .map_err(|e| {
                    eprintln!("Failed to convert request: {}", e);
                    make_error(e)
                })?;

            let response = {
                let handler_guard = handler.lock().unwrap();

                let req_zval = ext_php_rs::types::ZendClassObject::new(http_request)
                    .into_zval(false)
                    .map_err(|e| {
                        eprintln!("Failed to convert HttpRequest to Zval: {:?}", e);
                        make_error(format!("{:?}", e))
                    })?;

                handler_guard.try_call(vec![&req_zval])
                    .map_err(|e| {
                        eprintln!("Failed to call PHP handler: {:?}", e);
                        make_error(format!("{:?}", e))
                    })? 
            };

            let http_response_ref: &HttpResponse = response
                .extract()
                .ok_or_else(|| {
                    eprintln!("Handler did not return HttpResponse");
                    make_error("Handler did not return HttpResponse".to_string())
                })?;

            let http_response = HttpResponse {
                inner: http_response_ref.inner.clone(),
            };

            let hyper_response = http_response_to_hyper(http_response).await
                .map_err(|e| {
                    eprintln!("Failed to convert response: {}", e);
                    make_error(e)
                })?;

            // Map the body error to std::io::Error
            let (parts, body) = hyper_response.into_parts();
            let boxed_body = body.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)).boxed();
            
            Ok(Response::from_parts(parts, boxed_body))
        })
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpServer")]
pub struct HttpServer {
    pub(super) conn_builder: Shared<auto::Builder<LocalExecutor>>,
    pub(super) socket_io_layer: Option<SocketIoLayer>,
}

unsafe impl Send for HttpServer {}
unsafe impl Sync for HttpServer {}

#[php_impl]
impl HttpServer {
    #[php]
    pub fn __construct() -> Self {
        let builder = auto::Builder::new(LocalExecutor);
        Self {
            conn_builder: Shared::new(builder),
            socket_io_layer: None,
        }
    }

    #[php]
    pub fn with_socket_io(&mut self, socket_io: &AsyncSocketIo) {
        self.socket_io_layer = socket_io.layer.clone();
    }


    #[php]
    pub fn http1_only(&mut self) -> PhpResult<()> {
        let builder = std::mem::replace(
            self.conn_builder.get_mut(),
            auto::Builder::new(LocalExecutor)
        );
        *self.conn_builder.get_mut() = builder.http1_only();
        Ok(())
    }

    #[php]
    pub fn http2_only(&mut self) -> PhpResult<()> {
        let builder = std::mem::replace(
            self.conn_builder.get_mut(),
            auto::Builder::new(LocalExecutor)
        );
        *self.conn_builder.get_mut() = builder.http2_only();
        Ok(())
    }

    #[php]
    pub fn is_http1_available(&self) -> bool {
        self.conn_builder.get_ref().is_http1_available()
    }

    #[php]
    pub fn is_http2_available(&self) -> bool {
        self.conn_builder.get_ref().is_http2_available()
    }

    #[php]
    pub fn http1(&mut self, options: &ZendHashTable) -> PhpResult<()> {
        let mut h1_builder = self.conn_builder.get_mut().http1();

        if let Some(val) = options.get("auto_date_header") {
            if let Some(enabled) = val.bool() { h1_builder.auto_date_header(enabled); }
        }
        if let Some(val) = options.get("half_close") {
            if let Some(enabled) = val.bool() { h1_builder.half_close(enabled); }
        }
        if let Some(val) = options.get("keep_alive") {
            if let Some(enabled) = val.bool() { h1_builder.keep_alive(enabled); }
        }
        if let Some(val) = options.get("title_case_headers") {
            if let Some(enabled) = val.bool() { h1_builder.title_case_headers(enabled); }
        }
        if let Some(val) = options.get("ignore_invalid_headers") {
            if let Some(enabled) = val.bool() { h1_builder.ignore_invalid_headers(enabled); }
        }
        if let Some(val) = options.get("preserve_header_case") {
            if let Some(enabled) = val.bool() { h1_builder.preserve_header_case(enabled); }
        }
        if let Some(val) = options.get("max_headers") {
            if let Some(max) = val.long() { h1_builder.max_headers(max as usize); }
        }
        if let Some(val) = options.get("header_read_timeout") {
            if val.is_null() {
                h1_builder.header_read_timeout(None);
            } else if let Some(seconds) = val.double() {
                h1_builder.header_read_timeout(Some(Duration::from_secs_f64(seconds)));
            }
        }
        if let Some(val) = options.get("writev") {
            if let Some(enabled) = val.bool() { h1_builder.writev(enabled); }
        }
        if let Some(val) = options.get("max_buf_size") {
            if let Some(size) = val.long() { h1_builder.max_buf_size(size as usize); }
        }
        if let Some(val) = options.get("pipeline_flush") {
            if let Some(enabled) = val.bool() { h1_builder.pipeline_flush(enabled); }
        }
        Ok(())
    }

    #[php]
    pub fn http2(&mut self, options: &ZendHashTable) -> PhpResult<()> {
        let mut h2_builder = self.conn_builder.get_mut().http2();

        if let Some(val) = options.get("max_pending_accept_reset_streams") {
            if val.is_null() { h2_builder.max_pending_accept_reset_streams(None); }
            else if let Some(max) = val.long() { h2_builder.max_pending_accept_reset_streams(Some(max as usize)); }
        }
        if let Some(val) = options.get("max_local_error_reset_streams") {
            if val.is_null() { h2_builder.max_local_error_reset_streams(None); }
            else if let Some(max) = val.long() { h2_builder.max_local_error_reset_streams(Some(max as usize)); }
        }
        if let Some(val) = options.get("initial_stream_window_size") {
            if val.is_null() { h2_builder.initial_stream_window_size(None); }
            else if let Some(size) = val.long() { h2_builder.initial_stream_window_size(Some(size as u32)); }
        }
        if let Some(val) = options.get("initial_connection_window_size") {
            if val.is_null() { h2_builder.initial_connection_window_size(None); }
            else if let Some(size) = val.long() { h2_builder.initial_connection_window_size(Some(size as u32)); }
        }
        if let Some(val) = options.get("adaptive_window") {
            if let Some(enabled) = val.bool() { h2_builder.adaptive_window(enabled); }
        }
        if let Some(val) = options.get("max_frame_size") {
            if val.is_null() { h2_builder.max_frame_size(None); }
            else if let Some(size) = val.long() { h2_builder.max_frame_size(Some(size as u32)); }
        }
        if let Some(val) = options.get("max_concurrent_streams") {
            if val.is_null() { h2_builder.max_concurrent_streams(None); }
            else if let Some(max) = val.long() { h2_builder.max_concurrent_streams(Some(max as u32)); }
        }
        if let Some(val) = options.get("keep_alive_interval") {
            if val.is_null() { h2_builder.keep_alive_interval(None); }
            else if let Some(seconds) = val.double() { h2_builder.keep_alive_interval(Some(Duration::from_secs_f64(seconds))); }
        }
        if let Some(val) = options.get("keep_alive_timeout") {
            if let Some(seconds) = val.double() { h2_builder.keep_alive_timeout(Duration::from_secs_f64(seconds)); }
        }
        if let Some(val) = options.get("max_send_buf_size") {
            if let Some(size) = val.long() { h2_builder.max_send_buf_size(size as usize); }
        }
        if let Some(val) = options.get("enable_connect_protocol") {
            if let Some(true) = val.bool() { h2_builder.enable_connect_protocol(); }
        }
        if let Some(val) = options.get("max_header_list_size") {
            if let Some(size) = val.long() { h2_builder.max_header_list_size(size as u32); }
        }
        if let Some(val) = options.get("auto_date_header") {
            if let Some(enabled) = val.bool() { h2_builder.auto_date_header(enabled); }
        }
        Ok(())
    }

    #[php]
    pub fn serve(&self, io: &AsyncReadWriter, handler: &mut Zval) -> RustFuture {
        let builder = self.conn_builder.clone();
        let io_inner = io.get_inner();
        let handler_clone = handler.shallow_clone();
        let service = PhpHandlerService {
            handler: std::sync::Arc::new(std::sync::Mutex::new(handler_clone)),
        };
        let socket_io_layer = self.socket_io_layer.clone();

        let future = async move {
            let io_adapter = hyper_util::rt::TokioIo::new(SharedIoAdapter { inner: io_inner });
            
            let result = if let Some(layer) = socket_io_layer {
                builder
                    .get_ref()
                    .serve_connection_with_upgrades(io_adapter, layer.layer(service))
                    .await
            } else {
                builder
                    .get_ref()
                    .serve_connection_with_upgrades(io_adapter, service)
                    .await
            };

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

pub async fn http_request_from_hyper(req: Request<Incoming>) -> Result<HttpRequest, String> {
    use futures::TryStreamExt;
    use http_body::Frame;
    use http_body_util::StreamBody;
    use sync_wrapper::SyncStream;
    use std::io;

    let (parts, body) = req.into_parts();
    let stream = body
        .into_data_stream()
        .map_ok(Frame::data)
        .map_err(|e| Box::new(io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send>);
    let boxed_body = StreamBody::new(SyncStream::new(stream)).boxed();
    let http_request = http::Request::from_parts(parts, boxed_body);

    Ok(HttpRequest {
        inner: Shared::new(http_request),
    })
}

pub async fn http_response_to_hyper(
    mut response: HttpResponse,
) -> Result<Response<http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>>, String> {
    let body = response.take_body()?;
    let resp = response.inner.get_ref();
    let status = resp.status();
    let headers = resp.headers().clone();
    let version = resp.version();
    let mut hyper_response = Response::new(body);
    *hyper_response.status_mut() = status;
    *hyper_response.headers_mut() = headers;
    *hyper_response.version_mut() = version;
    Ok(hyper_response)
}
