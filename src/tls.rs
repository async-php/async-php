use ext_php_rs::prelude::*;
use crate::future::RustFuture;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::fs::File;
use std::io::BufReader;

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TlsStream")]
pub struct AsyncTlsStream {
    inner: Rc<RefCell<TlsStream<TcpStream>>>
}

#[php_impl]
impl AsyncTlsStream {
    pub fn connect(host: String, port: i64, ca_file: Option<String>) -> RustFuture {
        let future = async move {
            let mut root_store = RootCertStore::empty();
            
            if let Some(path) = ca_file {
                if let Ok(f) = File::open(path) {
                     let mut reader = BufReader::new(f);
                     for cert in rustls_pemfile::certs(&mut reader) {
                         if let Ok(c) = cert {
                            let _ = root_store.add(c);
                         }
                     }
                }
            }

            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            
            let connector = TlsConnector::from(Arc::new(config));
            let addr = format!("{}:{}", host, port);

             match TcpStream::connect(&addr).await {
                 Ok(tcp) => {
                     let domain_str = host.clone();
                     let domain = ServerName::try_from(domain_str.as_str())
                        .unwrap_or_else(|_| ServerName::try_from("example.com").unwrap())
                        .to_owned();
                     
                     match connector.connect(domain, tcp).await {
                         Ok(stream) => {
                            let obj = AsyncTlsStream { inner: Rc::new(RefCell::new(stream)) };
                            ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                         },
                         Err(_) => Zval::new()
                     }
                 },
                 Err(_) => Zval::new()
             }
        };
        RustFuture::new(future)
    }

    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            if let Ok(mut lock) = stream.try_borrow_mut() {
                match lock.read(&mut buf).await {
                    Ok(0) => {
                         let mut z = Zval::new();
                         z.set_string("", false).unwrap();
                         z
                    },
                    Ok(n) => {
                        buf.truncate(n);
                        let mut z = Zval::new();
                        z.set_binary(buf);
                        z
                    }
                    Err(_) => {
                         let mut z = Zval::new();
                         z.set_bool(false);
                         z
                    }
                }
            } else {
                 let mut z = Zval::new();
                 z.set_bool(false);
                 z
            }
        };
        RustFuture::new(future)
    }
    
    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            if let Ok(mut lock) = stream.try_borrow_mut() {
                match lock.write_all(data.as_bytes()).await {
                    Ok(_) => {
                        let mut z = Zval::new();
                        z.set_long(data.len() as i64);
                        z
                    },
                    Err(_) => {
                         let mut z = Zval::new();
                         z.set_bool(false);
                         z
                    }
                }
            } else {
                 let mut z = Zval::new();
                 z.set_bool(false);
                 z
            }
        };
        RustFuture::new(future)
    }
    
    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
             let mut z = Zval::new();
             if let Ok(mut lock) = stream.try_borrow_mut() {
                 let _ = lock.shutdown().await;
                 z.set_bool(true);
             } else {
                 z.set_bool(false);
             }
             z
        };
        RustFuture::new(future)
    }
}
