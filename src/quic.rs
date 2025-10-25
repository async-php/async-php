use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use quinn::{Endpoint, ServerConfig};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// Helper to generate self-signed cert for testing
fn generate_self_signed_cert() -> (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let key = cert.signing_key.serialize_der(); // Fixed field name
    let cert = cert.cert.der().clone();
    (vec![CertificateDer::from(cert)], PrivateKeyDer::try_from(key).unwrap())
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\QuicServer")]
pub struct AsyncQuicServer {
    endpoint: Endpoint,
}

#[php_impl]
impl AsyncQuicServer {
    pub fn bind(addr: String) -> PhpResult<RustFuture> {
        let future = async move {
            let (certs, key) = generate_self_signed_cert();
            
            let mut server_crypto = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .expect("bad certificate/key");
                
            server_crypto.alpn_protocols = vec![b"hq-29".to_vec()]; 

            let quic_server_config = quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto).unwrap();
            let server_config = ServerConfig::with_crypto(Arc::new(quic_server_config));

            let endpoint = Endpoint::server(server_config, addr.parse().unwrap()).unwrap();
            
            let obj = AsyncQuicServer { endpoint };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .unwrap_or_else(|_| Zval::new())
        };
        Ok(RustFuture::new(future))
    }

    /// Accept a new incoming QUIC connection
    pub fn accept(&self) -> RustFuture {
        let endpoint = self.endpoint.clone();
        let future = async move {
            if let Some(incoming) = endpoint.accept().await {
                 match incoming.await {
                     Ok(connection) => {
                         let obj = AsyncQuicConnection { inner: connection };
                         ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                     }
                     Err(_) => Zval::new(),
                 }
            } else {
                Zval::new()
            }
        };
        RustFuture::new(future)
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\QuicConnection")]
pub struct AsyncQuicConnection {
    inner: quinn::Connection,
}

#[php_impl]
impl AsyncQuicConnection {
    /// Accept a uni-directional stream
    pub fn accept_uni(&self) -> RustFuture {
        let conn = self.inner.clone();
        let future = async move {
            match conn.accept_uni().await {
                Ok(mut stream) => {
                    let data = stream.read_to_end(1024 * 64).await.unwrap_or_default();
                    let s = String::from_utf8_lossy(&data).to_string();
                    let mut z = Zval::new();
                    z.set_string(&s, false).unwrap();
                    z
                },
                Err(_) => Zval::new(),
            }
        };
        RustFuture::new(future)
    }
    
    /// Open a uni-directional stream and send data
    pub fn open_uni(&self, data: String) -> RustFuture {
        let conn = self.inner.clone();
        let future = async move {
            match conn.open_uni().await {
                Ok(mut stream) => {
                    let _ = stream.write_all(data.as_bytes()).await;
                    let _ = stream.finish();
                    let mut z = Zval::new();
                    z.set_bool(true);
                    z
                }
                Err(_) => {
                     let mut z = Zval::new();
                     z.set_bool(false);
                     z
                }
            }
        };
        RustFuture::new(future)
    }
}
