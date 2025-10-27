use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use hyper::{Body, Client, Method, Request};
use hyper_rustls::HttpsConnectorBuilder;
use std::collections::HashMap;

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\Client")]
pub struct AsyncHttpClient;

#[php_impl]
impl AsyncHttpClient {
    pub fn request(method: String, url: String, headers: Option<HashMap<String, String>>, body: Option<String>) -> RustFuture {
        let future = async move {
            let https = HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build();

            let client: Client<_, Body> = Client::builder().http1_only().build(https);

            let mut req_builder = Request::builder()
                .method(method.parse::<Method>().unwrap_or(Method::GET))
                .uri(url.clone());

            if let Some(map) = headers.clone() {
                for (k, v) in map {
                    req_builder = req_builder.header(k, v);
                }
            }

            let req = req_builder
                .body(Body::from(body.unwrap_or_default()))
                .map_err(|e| e.to_string())?;

            let resp = client.request(req).await.map_err(|e| e.to_string())?;
            let status = resp.status().as_u16() as i64;

            let mut headers_ht = ZendHashTable::new();
            for (k, v) in resp.headers().iter() {
                if let Ok(val_str) = v.to_str() {
                    let name = k.as_str().to_ascii_lowercase();
                    let _ = headers_ht.insert(name, val_str.to_string());
                }
            }

            let bytes = hyper::body::to_bytes(resp.into_body()).await.map_err(|e| e.to_string())?;
            let body_str = String::from_utf8_lossy(&bytes).to_string();

            let mut arr = ZendHashTable::new();
            let _ = arr.insert("status", status);
            let _ = arr.insert("body", body_str);
            let _ = arr.insert("headers", headers_ht);

            Ok::<Zval, String>(arr.into_zval(false).unwrap_or_else(|_| Zval::new()))
        };

        RustFuture::new(future)
    }
}
