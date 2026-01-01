use crate::http::{HttpRequest, HttpResponse};
use crate::net::AsyncQuicConnection;
use crate::future::RustFuture;
use bytes::{Bytes, Buf};
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use h3::server::Connection;
use http::{Request, Response};
use http_body_util::BodyExt;
use http_body_util::Full;
use crate::util::Shared;

/// HTTP/3 Server using QUIC
///
/// Unlike HTTP/1.1 and HTTP/2 which run over TCP, HTTP/3 runs over QUIC (UDP).
/// This server works with QuicListener and QuicConnection.
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\Http3Server")]
pub struct Http3Server {}

unsafe impl Send for Http3Server {}
unsafe impl Sync for Http3Server {}

#[php_impl]
impl Http3Server {
    /// Create a new HTTP/3 server
    #[php]
    pub fn __construct() -> Self {
        Self {}
    }

    /// Serve HTTP/3 requests on a QUIC connection
    ///
    /// # Parameters
    /// - `conn`: QuicConnection - The QUIC connection from QuicListener::accept()
    /// - `handler`: PHP callable - Function to handle requests, signature: fn(HttpRequest): HttpResponse
    ///
    /// # Example
    /// ```php
    /// $server = new Http3Server();
    /// $listener = QuicListener::bind('127.0.0.1:4433', 'cert.pem', 'key.pem');
    ///
    /// while (true) {
    ///     $conn = $listener->accept();
    ///     go(function() use ($server, $conn, $handler) {
    ///         $server->serve($conn, $handler);
    ///     });
    /// }
    /// ```
    #[php]
    pub fn serve(&self, quic_conn: &AsyncQuicConnection, handler: &mut Zval) -> RustFuture {
        let quic_conn = quic_conn.clone_connection();
        let handler_clone = handler.shallow_clone();

        let future = async move {
            // Get the QUIC connection
            let conn = quic_conn.get_connection().await
                .map_err(|e| format!("Failed to get QUIC connection: {}", e))?;

            // Create HTTP/3 connection
            let mut h3_conn = Connection::new(h3_quinn::Connection::new(conn))
                .await
                .map_err(|e| format!("Failed to create HTTP/3 connection: {}", e))?;

            // Handle requests on this connection
            loop {
                match h3_conn.accept().await {
                    Ok(Some(req_resolver)) => {
                        let handler = handler_clone.shallow_clone();

                        crate::fiber::context::spawn_local(async move {
                            // Resolve the request to get headers and stream
                            let (req, mut stream) = match req_resolver.resolve_request().await {
                                Ok(resolved) => resolved,
                                Err(e) => {
                                    eprintln!("Failed to resolve request: {:?}", e);
                                    return;
                                }
                            };

                            if let Err(e) = handle_request(req, &mut stream, handler).await {
                                eprintln!("Request handling error: {}", e);
                            }
                        });
                    }
                    Ok(None) => {
                        // Connection closed
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error accepting request: {:?}", e);
                        break;
                    }
                }
            }

            let mut z = Zval::new();
            z.set_bool(true);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }
}

impl AsyncQuicConnection {
    pub(crate) fn clone_connection(&self) -> Self {
        Self {
            incoming: self.incoming.clone(),
            connection: self.connection.clone(),
        }
    }
}

/// Handle a single HTTP/3 request
async fn handle_request(
    req: Request<()>,
    stream: &mut h3::server::RequestStream<h3_quinn::BidiStream<bytes::Bytes>, Bytes>,
    handler: Zval,
) -> Result<(), String> {
    // Read the request body
    let mut body_bytes = Vec::new();
    while let Some(mut chunk) = stream.recv_data().await
        .map_err(|e| format!("Failed to read request body: {:?}", e))?
    {
        let len = chunk.remaining();
        let bytes = chunk.copy_to_bytes(len);
        body_bytes.extend_from_slice(&bytes);
    }

    // Convert h3 request to our HttpRequest
    let (parts, _) = req.into_parts();
    let full_body = Full::new(Bytes::from(body_bytes))
        .map_err(|e: std::convert::Infallible| match e {})
        .boxed();

    let http_request = http::Request::from_parts(parts, full_body);
    let http_request = HttpRequest {
        inner: Shared::new(http_request),
    };

    // Call PHP handler
    let req_zval = ext_php_rs::types::ZendClassObject::new(http_request)
        .into_zval(false)
        .map_err(|e| format!("Failed to convert request: {:?}", e))?;

    let response = handler.try_call(vec![&req_zval])
        .map_err(|e| format!("Handler call failed: {:?}", e))?;

    // Extract HttpResponse
    let http_response_ref: &HttpResponse = response.extract()
        .ok_or_else(|| "Handler did not return HttpResponse".to_string())?;

    // Clone the response
    let mut http_response = HttpResponse {
        inner: http_response_ref.inner.clone(),
    };

    // Convert HttpResponse to h3 Response
    let response_ref = http_response.inner.get_ref();
    let status = response_ref.status();
    let headers = response_ref.headers().clone();

    // Build h3 response
    let mut resp_builder = Response::builder().status(status);
    for (name, value) in &headers {
        resp_builder = resp_builder.header(name, value);
    }
    let resp = resp_builder.body(())
        .map_err(|e| format!("Failed to build response: {}", e))?;

    // Send response headers
    stream.send_response(resp).await
        .map_err(|e| format!("Failed to send response: {:?}", e))?;

    // Send body
    let body = http_response.take_body()
        .map_err(|e| format!("Failed to take body: {}", e))?;
    let collected = body.collect().await
        .map_err(|e| format!("Failed to collect body: {}", e))?;
    let body_bytes = collected.to_bytes();

    if !body_bytes.is_empty() {
        stream.send_data(body_bytes).await
            .map_err(|e| format!("Failed to send body: {:?}", e))?;
    }

    stream.finish().await
        .map_err(|e| format!("Failed to finish stream: {:?}", e))?;

    Ok(())
}
