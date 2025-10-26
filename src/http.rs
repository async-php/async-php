use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::convert::FromZvalMut;
use crate::future::RustFuture;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full, StreamBody};
use bytes::{Bytes, BytesMut};
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use tracing::{info, error};
use tokio::sync::mpsc;
use futures::StreamExt; 

// ======================================================================================
// HTTP Body Streams
// ======================================================================================

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\RequestBody")]
#[derive(Debug)]
pub struct AsyncHttpRequestBody {
    inner: Arc<Mutex<Option<hyper::body::Incoming>>>,
    buffer: Arc<Mutex<BytesMut>>,
}

#[php_impl]
impl AsyncHttpRequestBody {
    pub fn read(&self, length: usize) -> RustFuture {
        let inner_mutex = self.inner.clone();
        let buffer_mutex = self.buffer.clone();
        
        let future = async move {
            let mut buffer = buffer_mutex.lock().unwrap();
            
            if buffer.len() >= length {
                let chunk = buffer.split_to(length);
                let s = String::from_utf8_lossy(&chunk).to_string();
                let mut z = Zval::new();
                z.set_string(&s, false).unwrap();
                return z;
            }

            let mut inner_opt = inner_mutex.lock().unwrap();
            if let Some(body) = inner_opt.as_mut() {
                while buffer.len() < length {
                    match body.frame().await {
                        Some(Ok(frame)) => {
                            if let Ok(data) = frame.into_data() {
                                buffer.extend_from_slice(&data);
                            }
                        }
                        Some(Err(_)) => break, 
                        None => break, 
                    }
                }
            }
            
            let read_len = std::cmp::min(length, buffer.len());
            let chunk = buffer.split_to(read_len);
            let s = String::from_utf8_lossy(&chunk).to_string();
            let mut z = Zval::new();
            z.set_string(&s, false).unwrap();
            z
        };
        RustFuture::new(future)
    }
}

// ======================================================================================
// Request & Response
// ======================================================================================

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\Request")]
#[derive(Debug)]
pub struct AsyncHttpRequest {
    #[php(prop)]
    pub method: String,
    #[php(prop)]
    pub uri: String,
    #[php(prop)]
    pub headers: HashMap<String, String>,
    
    body_stream: Option<AsyncHttpRequestBody>,

    pub response_zval: Option<Zval>,
}

#[php_impl]
impl AsyncHttpRequest {
    pub fn get_method(&self) -> String { self.method.clone() }
    pub fn get_uri(&self) -> String { self.uri.clone() } 
    
    pub fn get_header(&self, name: String) -> Option<String> {
        self.headers.get(&name).cloned()
    }

    pub fn get_body(&self) -> Option<Zval> {
         if let Some(bs) = &self.body_stream {
             let obj = AsyncHttpRequestBody {
                 inner: bs.inner.clone(),
                 buffer: bs.buffer.clone(),
             };
             Some(ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or(Zval::new()))
         } else {
             None
         }
    }

    pub fn get_response(&self) -> Zval {
        self.response_zval.as_ref().map(|z| z.shallow_clone()).unwrap_or(Zval::new())
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\Response")]
#[derive(Debug)]
pub struct AsyncHttpResponse {
    #[php(prop)]
    pub status: u16,
    #[php(prop)]
    pub headers: HashMap<String, String>,
    
    pub body_stream: RefCell<Option<mpsc::Receiver<String>>>,
    #[php(prop)]
    pub body_string: String, 

    pub request_zval: Option<Zval>,
    
    pub stream_sender: Option<mpsc::Sender<String>>,
}

#[php_impl]
impl AsyncHttpResponse {
    pub fn __construct() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body_stream: RefCell::new(None),
            body_string: String::new(),
            request_zval: None,
            stream_sender: None,
        }
    }

    pub fn with_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn with_status(&mut self, status: i64) {
        self.status = status as u16;
    }

    pub fn with_body(&mut self, body: String) {
        self.body_string = body;
    }
    
    pub fn init_stream(&mut self) {
        let (tx, rx) = mpsc::channel(16);
        self.stream_sender = Some(tx);
        *self.body_stream.borrow_mut() = Some(rx);
    }

    pub fn write(&self, data: String) -> RustFuture {
        if let Some(tx) = &self.stream_sender {
            let tx = tx.clone();
            let future = async move {
                let _ = tx.send(data).await;
                Zval::new()
            };
            RustFuture::new(future)
        } else {
             RustFuture::new(async { Zval::new() })
        }
    }
    
    pub fn end(&self) -> RustFuture {
         
         RustFuture::new(async { Zval::new() })
    }

    pub fn get_request(&self) -> Zval {
        self.request_zval.as_ref().map(|z| z.shallow_clone()).unwrap_or(Zval::new())
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\Server")]
pub struct AsyncHttpServer;

#[php_impl]
impl AsyncHttpServer {
    pub fn listen(addr: String, handler: &Zval, config: Option<HashMap<String, String>>) -> PhpResult<RustFuture> {
        let handler = Rc::new(handler.shallow_clone());
        let addr_str = addr.clone();
        
        let _enable_http3 = if let Some(c) = config {
             c.get("enable_http3").map(|v| v == "true").unwrap_or(false)
        } else {
            false
        };

        let future = async move {
            let listener = match TcpListener::bind(&addr_str).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind TCP {}: {}", addr_str, e);
                    return Zval::new();
                }
            };

            info!("HTTP Server listening on http://{}", addr_str);

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let io = TokioIo::new(stream);
                let handler_clone = handler.clone();

                tokio::task::spawn_local(async move {
                    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                        let handler_inner = handler_clone.clone();
                        async move {
                            let (parts, body_incoming) = req.into_parts();
                            
                            let body_stream_obj = AsyncHttpRequestBody {
                                inner: Arc::new(Mutex::new(Some(body_incoming))),
                                buffer: Arc::new(Mutex::new(BytesMut::new())),
                            };

                            let mut headers_map = HashMap::new();
                            for (k, v) in parts.headers.iter() {
                                headers_map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
                            }

                            let php_req = AsyncHttpRequest {
                                method: parts.method.to_string(),
                                uri: parts.uri.to_string(),
                                headers: headers_map,
                                body_stream: Some(body_stream_obj),
                                response_zval: None,
                            };

                            let php_res = AsyncHttpResponse {
                                status: 200,
                                headers: HashMap::new(),
                                body_stream: RefCell::new(None),
                                body_string: String::new(),
                                request_zval: None,
                                stream_sender: None,
                            };
                            
                            let mut req_zval = ext_php_rs::types::ZendClassObject::new(php_req).into_zval(false).unwrap();
                            let mut res_zval = ext_php_rs::types::ZendClassObject::new(php_res).into_zval(false).unwrap();
                            
                            // Use FromZvalMut to get mutable access to the object
                            if let Some(obj) = <&mut ext_php_rs::types::ZendObject>::from_zval_mut(&mut req_zval) {
                                let _ = obj.set_property("response_zval", res_zval.shallow_clone());
                            }
                            if let Some(obj) = <&mut ext_php_rs::types::ZendObject>::from_zval_mut(&mut res_zval) {
                                let _ = obj.set_property("request_zval", req_zval.shallow_clone());
                            }

                            let result = handler_inner.try_call(vec![&req_zval]);
                            
                            let final_res_zval = if let Ok(r) = result {
                                if r.is_object() { r } else { res_zval }
                            } else {
                                res_zval
                            };
                            
                            let mut response_builder = Response::builder();
                            let mut body_stream_receiver = None;
                            let mut body_string = String::new();

                            if let Some(php_res_ref) = <&AsyncHttpResponse as ext_php_rs::convert::FromZval>::from_zval(&final_res_zval) {
                                response_builder = response_builder.status(StatusCode::from_u16(php_res_ref.status).unwrap_or(StatusCode::OK));
                                for (k, v) in &php_res_ref.headers {
                                    response_builder = response_builder.header(k, v);
                                }
                                
                                if let Ok(mut cell) = php_res_ref.body_stream.try_borrow_mut() {
                                    body_stream_receiver = cell.take();
                                }
                                body_string = php_res_ref.body_string.clone();
                            }

                            if let Some(rx) = body_stream_receiver {
                                let stream = futures::stream::unfold(rx, |mut rx| async move {
                                    match rx.recv().await {
                                        Some(data) => Some((Ok::<_, std::io::Error>(hyper::body::Frame::data(Bytes::from(data))), rx)),
                                        None => None,
                                    }
                                });
                                let body = StreamBody::new(stream);
                                Ok::<_, hyper::Error>(response_builder.body(http_body_util::BodyExt::boxed(body)).unwrap())
                            } else {
                                let body = Full::new(Bytes::from(body_string))
                                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Impossible"));
                                Ok::<_, hyper::Error>(response_builder.body(http_body_util::BodyExt::boxed(body)).unwrap())
                            }
                        }
                    });

                    if let Err(_) = http1::Builder::new().serve_connection(io, service).await {}
                });
            }
            Zval::new()
        };

        Ok(RustFuture::new(future))
    }
}