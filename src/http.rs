use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval; // Added import
use crate::future::RustFuture;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use bytes::Bytes;
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::rc::Rc;

/// Represents an HTTP Request received from Hyper.
#[php_class]
#[derive(Debug)]
pub struct AsyncHttpRequest {
    #[php(prop)]
    pub method: String,
    #[php(prop)]
    pub uri: String,
    #[php(prop)]
    pub headers: HashMap<String, String>,
    #[php(prop)]
    pub body: String,
}

#[php_impl]
impl AsyncHttpRequest {
    pub fn get_method(&self) -> String {
        self.method.clone()
    }

    pub fn get_uri(&self) -> String {
        self.uri.clone()
    }

    pub fn get_body(&self) -> String {
        self.body.clone()
    }

    pub fn get_header(&self, name: String) -> Option<String> {
        self.headers.get(&name).cloned()
    }
}

/// Represents an HTTP Response to be sent back via Hyper.
#[php_class]
#[derive(Debug)]
pub struct AsyncHttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[php_impl]
impl AsyncHttpResponse {
    pub fn __construct(status: Option<i64>, body: Option<String>) -> Self {
        Self {
            status: status.unwrap_or(200) as u16,
            headers: HashMap::new(),
            body: body.unwrap_or_default(),
        }
    }

    pub fn with_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn with_status(&mut self, status: i64) {
        self.status = status as u16;
    }

    pub fn with_body(&mut self, body: String) {
        self.body = body;
    }
}

/// The HTTP Server implementation using Hyper.
#[php_class]
pub struct AsyncHttpServer;

#[php_impl]
impl AsyncHttpServer {
    /// Starts an HTTP server on the given address.
    /// The handler is a PHP callable: function(AsyncHttpRequest $req): AsyncHttpResponse
    pub fn listen(addr: String, handler: &Zval) -> PhpResult<RustFuture> { // Changed Zval to &Zval
        let handler = Rc::new(handler.shallow_clone()); // Clone it
        let addr = addr.clone();

        let future = async move {
            let listener = match TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to bind to {}: {}", addr, e);
                    return Zval::new();
                }
            };

            println!("Listening on http://{}", addr);

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let io = TokioIo::new(stream);
                let handler_clone = handler.clone();

                // Spawn a task for each connection. 
                // Crucial: Use spawn_local to allow !Send Zvals (PHP objects) to be used.
                tokio::task::spawn_local(async move {
                    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                        let handler_inner = handler_clone.clone();
                        async move {
                            // 1. Read Body (Async)
                            let (parts, body) = req.into_parts();
                            let body_bytes = match body.collect().await {
                                Ok(b) => b.to_bytes(),
                                Err(_) => Bytes::new(),
                            };
                            let body_str = String::from_utf8_lossy(&body_bytes).to_string();

                            // 2. Construct AsyncHttpRequest
                            let mut headers_map = HashMap::new();
                            for (k, v) in parts.headers.iter() {
                                headers_map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
                            }

                            let php_req = AsyncHttpRequest {
                                method: parts.method.to_string(),
                                uri: parts.uri.to_string(),
                                headers: headers_map,
                                body: body_str,
                            };

                            // 3. Convert PHP Request to Zval
                            let req_zval = ext_php_rs::types::ZendClassObject::new(php_req)
                                .into_zval(false)
                                .unwrap_or_else(|_| Zval::new());

                            // 4. Call PHP Handler
                            let result = handler_inner.try_call(vec![&req_zval]);

                            // 5. Process Result -> AsyncHttpResponse
                            let mut response_builder = Response::builder();
                            let mut body_content = String::new();

                            match result {
                                Ok(res_zval) => {
                                    if let Some(php_res) = <&AsyncHttpResponse as ext_php_rs::convert::FromZval>::from_zval(&res_zval) {
                                        response_builder = response_builder.status(StatusCode::from_u16(php_res.status).unwrap_or(StatusCode::OK));
                                        for (k, v) in &php_res.headers {
                                            response_builder = response_builder.header(k, v);
                                        }
                                        body_content = php_res.body.clone();
                                    } else {
                                        // Fallback if return type is not AsyncHttpResponse (e.g. string or null)
                                        if res_zval.is_string() {
                                            body_content = res_zval.string().unwrap_or_default();
                                        } else {
                                             response_builder = response_builder.status(StatusCode::INTERNAL_SERVER_ERROR);
                                             body_content = "Handler returned invalid type".to_string();
                                        }
                                    }
                                }
                                Err(e) => {
                                    response_builder = response_builder.status(StatusCode::INTERNAL_SERVER_ERROR);
                                    body_content = format!("PHP Exception: {:?}", e);
                                }
                            }

                            Ok::<_, hyper::Error>(response_builder
                                .body(Full::new(Bytes::from(body_content)))
                                .unwrap())
                        }
                    });

                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        // eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
            
            #[allow(unreachable_code)]
            Zval::new()
        };

        Ok(RustFuture::new(future))
    }
}