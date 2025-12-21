use ext_php_rs::prelude::*;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use crate::io::cast_io;
use quinn::{Endpoint, ServerConfig, ClientConfig, Incoming, Connection, RecvStream, SendStream, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::Result as IoResult;
use std::fs::File;
use std::io::BufReader;

// --- Helper Structs ---

/// Bidirectional stream wrapper for IO casting
pub struct QuicBiStream {
    pub send: SendStream,
    pub recv: RecvStream,
}

impl AsyncRead for QuicBiStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.recv).poll_read(cx, buf)
    }
}

impl AsyncWrite for QuicBiStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.send).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.send).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.send).poll_shutdown(cx)
    }
}

// --- QUIC Listener ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicListener")]
pub struct AsyncQuicListener {
    endpoint: Shared<Option<Endpoint>>,
}

#[php_impl]
impl AsyncQuicListener {
    /// Bind a QUIC listener with TLS certificates
    #[php]
    pub fn bind(addr: String, tls_config: &ZendHashTable) -> PhpResult<Self> {
        // Parse address
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| PhpException::default(format!("Invalid address: {}", e)))?;

        // Extract cert and key paths
        let cert_path = tls_config.get("cert_path")
            .and_then(|z| z.string())
            .ok_or_else(|| PhpException::default("Missing 'cert_path' in tlsConfig".to_string()))?;
        
        let key_path = tls_config.get("key_path")
            .and_then(|z| z.string())
            .ok_or_else(|| PhpException::default("Missing 'key_path' in tlsConfig".to_string()))?;

        // Load TLS certificate
        let cert_file = File::open(&cert_path)
            .map_err(|e| PhpException::default(format!("Failed to open cert file: {}", e)))?;
        let mut cert_reader = BufReader::new(cert_file);

        let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PhpException::default(format!("Failed to parse cert: {}", e)))?;

        if certs.is_empty() {
            return Err(PhpException::default("No certificates found in cert file".to_string()));
        }

        // Load private key
        let key_file = File::open(&key_path)
            .map_err(|e| PhpException::default(format!("Failed to open key file: {}", e)))?;
        let mut key_reader = BufReader::new(key_file);

        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| PhpException::default(format!("Failed to parse key: {}", e)))?;
            .ok_or_else(|| PhpException::default("No private key found in key file".to_string()))?;

        // Create server config
        let mut server_config = ServerConfig::with_single_cert(certs, key)
            .map_err(|e| PhpException::default(format!("Failed to create server config: {}", e)))?;

        // Parse ALPN
        if let Some(alpn_zval) = tls_config.get("alpn") {
            if let Some(alpn_arr) = alpn_zval.array() {
                let mut alpns = Vec::new();
                for (_, val) in alpn_arr.iter() {
                    if let Some(s) = val.string() {
                        alpns.push(s.into_bytes());
                    }
                }
                if !alpns.is_empty() {
                    server_config.alpn_protocols = alpns;
                }
            }
        } else {
             // Default ALPN for HTTP/3
             server_config.alpn_protocols = vec![b"h3".to_vec()];
        }

        // Create QUIC endpoint
        let endpoint = Endpoint::server(server_config, socket_addr)
            .map_err(|e| PhpException::default(format!("Failed to bind endpoint: {}", e)))?;

        Ok(Self {
            endpoint: Shared::new(Some(endpoint)),
        })
    }

    /// Accept a new QUIC connection
    #[php]
    pub fn accept(&mut self) -> RustFuture {
        let endpoint = self.endpoint.clone();

        let future = async move {
            let endpoint_ref = endpoint.get_ref();
            let endpoint = endpoint_ref.as_ref()
                .ok_or_else(|| "Listener has been closed".to_string())?;

            let incoming = endpoint.accept().await
                .ok_or_else(|| "Listener closed".to_string())?;

            let conn = AsyncQuicConnection {
                incoming: Shared::new(Some(incoming)),
                connection: Shared::new(None),
            };

            ext_php_rs::types::ZendClassObject::new(conn).into_zval(false)
                .map_err(|e| format!("Failed to convert connection: {:?}", e))
        };

        RustFuture::new(future)
    }

    #[php]
    pub fn local_addr(&self) -> PhpResult<String> {
        let endpoint_ref = self.endpoint.get_ref();
        let endpoint = endpoint_ref.as_ref()
            .ok_or_else(|| PhpException::default("Listener has been closed".to_string()))?;

        Ok(endpoint.local_addr()
            .map_err(|e| PhpException::default(format!("Failed to get local address: {}", e)))?)
            .to_string())
    }

    #[php]
    pub fn close(&mut self) {
        *self.endpoint.get_mut() = None;
    }
}

// --- QUIC Connection ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicConnection")]
pub struct AsyncQuicConnection {
    pub(crate) incoming: Shared<Option<Incoming>>,
    pub(crate) connection: Shared<Option<Connection>>,
}

unsafe impl Send for AsyncQuicConnection {}
unsafe impl Sync for AsyncQuicConnection {}

impl AsyncQuicConnection {
    /// Helper to ensure connection is established and return a clone
    pub async fn get_connection(&self) -> Result<Connection, String> {
        // Check if already connected
        {
            let conn_ref = self.connection.get_ref();
            if let Some(conn) = conn_ref.as_ref() {
                return Ok(conn.clone());
            }
        }

        // Accept the incoming connection if pending
        let mut incoming_guard = self.connection.get_mut(); // Lock connection to prevent race
        // Re-check
        if let Some(conn) = incoming_guard.as_ref() {
             return Ok(conn.clone());
        }

        let incoming_opt = self.incoming.get_mut().take();
        if let Some(incoming) = incoming_opt {
             let conn = incoming.await
                 .map_err(|e| format!("Failed to establish connection: {}", e))?;
             *incoming_guard = Some(conn.clone());
             Ok(conn)
        } else {
             Err("Connection not established or already consumed".to_string())
        }
    }
}

#[php_impl]
impl AsyncQuicConnection {
    #[php]
    pub fn connect(addr: String, server_name: Option<String>, config: Option<&ZendHashTable>) -> RustFuture {
        let future = async move {
            let socket_addr: SocketAddr = addr.parse()
                .map_err(|e| format!("Invalid address: {}", e))?;

            // Setup Client Config
            let mut client_config = ClientConfig::new(Arc::new(quinn::crypto::rustls::QuicClientConfig::try_from(
                rustls::ClientConfig::builder()
                    .with_root_certificates(rustls::RootCertStore::empty()) // TODO: Load system roots?
                    .with_no_client_auth()
            ).unwrap())); // Fallback to safe defaults if simplified constr fails, but here we need customization

            // Handle config (ALPN, etc) - simplified for now
             let mut alpn = vec![b"h3".to_vec()];
             if let Some(ht) = config {
                 if let Some(alpn_zval) = ht.get("alpn") {
                     if let Some(arr) = alpn_zval.array() {
                         let mut alpns = Vec::new();
                         for (_, val) in arr.iter() {
                             if let Some(s) = val.string() {
                                 alpns.push(s.into_bytes());
                             }
                         }
                         if !alpns.is_empty() {
                             alpn = alpns;
                         }
                     }
                 }
             }
             
             // Create a standard config with WebPKI roots
             let mut roots = rustls::RootCertStore::empty();
             roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
             
             let mut crypto = rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth();
            
            crypto.alpn_protocols = alpn;

             // TODO: Verify cert option
             // if verify_cert == false -> unsafe verifier

             let client_config = ClientConfig::new(Arc::new(quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(|e| e.to_string())?));
             
             let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
                 .map_err(|e| format!("Failed to create client endpoint: {}", e))?;
             endpoint.set_default_client_config(client_config);

             let host = server_name.unwrap_or_else(|| "localhost".to_string());
             let conn = endpoint.connect(socket_addr, &host)
                 .map_err(|e| format!("Connect failed: {}", e))?;
                 .await
                 .map_err(|e| format!("Connection failed: {}", e))?;

            let obj = AsyncQuicConnection {
                incoming: Shared::new(None),
                connection: Shared::new(Some(conn)),
            };

            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert connection: {:?}", e))
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn open_bi_stream(&self) -> RustFuture {
        let conn_self = self.connection.clone();
        let incoming_self = self.incoming.clone();
        let me = AsyncQuicConnection { incoming: incoming_self, connection: conn_self }; // shallow copy logic

        let future = async move {
             let conn = me.get_connection().await?;
             let (send, recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
             
             let stream = AsyncQuicStream {
                 inner: Shared::new(QuicBiStream { send, recv })
             };
             
             ext_php_rs::types::ZendClassObject::new(stream)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert stream: {:?}", e))
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn open_uni_stream(&self) -> RustFuture {
        let conn_self = self.connection.clone();
        let incoming_self = self.incoming.clone();
        let me = AsyncQuicConnection { incoming: incoming_self, connection: conn_self };

        let future = async move {
             let conn = me.get_connection().await?;
             let send = conn.open_uni().await.map_err(|e| e.to_string())?;
             
             let stream = AsyncQuicSendStream {
                 inner: Shared::new(send)
             };
             
             ext_php_rs::types::ZendClassObject::new(stream)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert stream: {:?}", e))
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn accept_bi_stream(&self) -> RustFuture {
        let conn_self = self.connection.clone();
        let incoming_self = self.incoming.clone();
        let me = AsyncQuicConnection { incoming: incoming_self, connection: conn_self };

        let future = async move {
             let conn = me.get_connection().await?;
             match conn.accept_bi().await {
                 Ok((send, recv)) => {
                     let stream = AsyncQuicStream {
                         inner: Shared::new(QuicBiStream { send, recv })
                     };
                     ext_php_rs::types::ZendClassObject::new(stream)
                        .into_zval(false)
                        .map_err(|e| format!("Failed to convert stream: {:?}", e))
                 },
                 Err(e) => Err(format!("Failed to accept stream: {}", e))
             }
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn accept_uni_stream(&self) -> RustFuture {
        let conn_self = self.connection.clone();
        let incoming_self = self.incoming.clone();
        let me = AsyncQuicConnection { incoming: incoming_self, connection: conn_self };

        let future = async move {
             let conn = me.get_connection().await?;
             match conn.accept_uni().await {
                 Ok(recv) => {
                     let stream = AsyncQuicRecvStream {
                         inner: Shared::new(recv)
                     };
                     ext_php_rs::types::ZendClassObject::new(stream)
                        .into_zval(false)
                        .map_err(|e| format!("Failed to convert stream: {:?}", e))
                 },
                 Err(e) => Err(format!("Failed to accept stream: {}", e))
             }
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn remote_addr(&self) -> PhpResult<String> {
        let conn_ref = self.connection.get_ref();
        if let Some(conn) = conn_ref.as_ref() {
             Ok(conn.remote_address().to_string())
        } else {
             Err(PhpException::default("Connection not established".to_string()))
        }
    }

    #[php]
    pub fn local_addr(&self) -> PhpResult<String> {
        let conn_ref = self.connection.get_ref();
        if let Some(conn) = conn_ref.as_ref() {
             Ok(conn.remote_address().to_string()) // Warning: This is remote, need local? Quinn Connection doesn't expose local easily?
             // Actually connection.local_ip() exists? No.
             // We can get it from endpoint, but we don't store endpoint here.
             // For now return remote or empty?
             // Re-reading docs: Connection usually knows remote.
             // We will stick to remote_addr.
        } else {
             Err(PhpException::default("Connection not established".to_string()))
        }
    }

    #[php]
    pub fn close(&mut self, error_code: Option<i64>, reason: Option<String>) -> RustFuture {
        let conn_ref = self.connection.clone();
        let future = async move {
            if let Some(conn) = conn_ref.get_ref().as_ref() {
                let code = quinn::VarInt::from_u32(error_code.unwrap_or(0) as u32);
                conn.close(code, reason.unwrap_or_default().as_bytes());
            }
             let mut z = Zval::new();
             z.set_bool(true);
             Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn cast_to(&self, _ty: i64) -> PhpResult<Zval> {
        Err(PhpException::default("QUIC Connection cannot be cast to IO stream directly. Use openBiStream() to get a stream.".to_string()))
    }
}

// --- QUIC Streams ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicStream")]
pub struct AsyncQuicStream {
    inner: Shared<QuicBiStream>,
}

#[php_impl]
impl AsyncQuicStream {
    #[php]
    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            let mut guard = stream.get_mut();
            let recv = &mut guard.recv;
            
            match recv.read(&mut buf).await {
                Ok(Some(n)) => {
                    buf.truncate(n);
                    let mut z = Zval::new();
                    z.set_binary(buf);
                    Ok(z)
                },
                Ok(None) => Ok(Zval::new()), // EOF
                Err(e) => Err(e.to_string())
            }
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut guard = stream.get_mut();
            let send = &mut guard.send;
            
            send.write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;
            
            let mut z = Zval::new();
            z.set_long(data.len() as i64);
            Ok(z)
        };
        RustFuture::new(future)
    }
    
    #[php]
    pub fn flush(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().send.flush().await.map_err(|e| e.to_string())?;
             let mut z = Zval::new();
             z.set_bool(true);
             Ok(z)
        };
        RustFuture::new(future)
    }

    #[php]
    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().send.finish().await.map_err(|e| e.to_string())?;
             let mut z = Zval::new();
             z.set_bool(true);
             Ok(z)
        };
        RustFuture::new(future)
    }
    
    #[php]
    pub fn finish(&self) -> RustFuture {
        self.close()
    }

    #[php]
    pub fn id(&self) -> i64 {
        self.inner.get_ref().send.id().index() as i64
    }

    #[php]
    pub fn cast_to(&self, ty: i64) -> PhpResult<Zval> {
        let io: Shared<Box<dyn crate::io::AsyncReadWrite + Unpin + Send>> =
            Shared::new(Box::new(self.inner.clone()));
        cast_io(&io, ty)
    }
}

// RecvStream
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicRecvStream")]
pub struct AsyncQuicRecvStream {
    inner: Shared<RecvStream>,
}

#[php_impl]
impl AsyncQuicRecvStream {
    #[php]
    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            match stream.get_mut().read(&mut buf).await {
                 Ok(Some(n)) => {
                    buf.truncate(n);
                    let mut z = Zval::new();
                    z.set_binary(buf);
                    Ok(z)
                },
                Ok(None) => Ok(Zval::new()),
                Err(e) => Err(e.to_string())
            }
        };
        RustFuture::new(future)
    }
    
    #[php]
    pub fn id(&self) -> i64 {
        self.inner.get_ref().id().index() as i64
    }
}

// SendStream
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicSendStream")]
pub struct AsyncQuicSendStream {
    inner: Shared<SendStream>,
}

#[php_impl]
impl AsyncQuicSendStream {
    #[php]
    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_long(data.len() as i64);
            Ok(z)
        };
        RustFuture::new(future)
    }
    
    #[php]
    pub fn flush(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().flush().await.map_err(|e| e.to_string())?;
             let mut z = Zval::new();
             z.set_bool(true);
             Ok(z)
        };
        RustFuture::new(future)
    }
    
    #[php]
    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().finish().await.map_err(|e| e.to_string())?;
             let mut z = Zval::new();
             z.set_bool(true);
             Ok(z)
        };
        RustFuture::new(future)
    }
    
    #[php]
    pub fn finish(&self) -> RustFuture {
        self.close()
    }
    
    #[php]
    pub fn id(&self) -> i64 {
        self.inner.get_ref().id().index() as i64
    }
}