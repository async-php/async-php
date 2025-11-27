use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;
use quinn::{Endpoint, ServerConfig, Incoming, Connection};
use rustls_pki_types::CertificateDer;
use std::net::SocketAddr;
use crate::future::RustFuture;
use crate::util::Shared;

/// QUIC Listener for accepting HTTP/3 connections
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicListener")]
pub struct AsyncQuicListener {
    endpoint: Shared<Option<Endpoint>>,
}

unsafe impl Send for AsyncQuicListener {}
unsafe impl Sync for AsyncQuicListener {}

#[php_impl]
impl AsyncQuicListener {
    /// Bind a QUIC listener with TLS certificates
    ///
    /// # Parameters
    /// - `addr`: Address to bind to (e.g., "127.0.0.1:4433")
    /// - `cert_path`: Path to TLS certificate file
    /// - `key_path`: Path to private key file
    ///
    /// # Example
    /// ```php
    /// $listener = QuicListener::bind('127.0.0.1:4433', 'cert.pem', 'key.pem');
    /// ```
    #[php]
    pub fn bind(addr: String, cert_path: String, key_path: String) -> PhpResult<Self> {
        use std::fs::File;
        use std::io::BufReader;

        // Parse address
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| PhpException::default(format!("Invalid address: {}", e)))?;

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
            .map_err(|e| PhpException::default(format!("Failed to parse key: {}", e)))?
            .ok_or_else(|| PhpException::default("No private key found in key file".to_string()))?;

        // Create server config
        let server_config = ServerConfig::with_single_cert(certs, key)
            .map_err(|e| PhpException::default(format!("Failed to create server config: {}", e)))?;

        // Create QUIC endpoint
        let endpoint = Endpoint::server(server_config, socket_addr)
            .map_err(|e| PhpException::default(format!("Failed to bind endpoint: {}", e)))?;

        Ok(Self {
            endpoint: Shared::new(Some(endpoint)),
        })
    }

    /// Accept a new QUIC connection
    ///
    /// Returns a QuicConnection when a client connects
    ///
    /// # Example
    /// ```php
    /// while (true) {
    ///     $conn = $listener->accept();
    ///     go(fn() => $server->serve($conn, $handler));
    /// }
    /// ```
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

    /// Get the local address the listener is bound to
    #[php]
    pub fn local_addr(&self) -> PhpResult<String> {
        let endpoint_ref = self.endpoint.get_ref();
        let endpoint = endpoint_ref.as_ref()
            .ok_or_else(|| PhpException::default("Listener has been closed".to_string()))?;

        Ok(endpoint.local_addr()
            .map_err(|e| PhpException::default(format!("Failed to get local address: {}", e)))?
            .to_string())
    }

    /// Close the listener
    #[php]
    pub fn close(&mut self) {
        *self.endpoint.get_mut() = None;
    }
}

/// QUIC Connection
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Quic\\QuicConnection")]
pub struct AsyncQuicConnection {
    pub(crate) incoming: Shared<Option<Incoming>>,
    pub(crate) connection: Shared<Option<Connection>>,
}

unsafe impl Send for AsyncQuicConnection {}
unsafe impl Sync for AsyncQuicConnection {}

impl AsyncQuicConnection {
    /// Get the established connection (accepts the incoming connection if needed)
    pub async fn get_connection(&self) -> Result<Connection, String> {
        // Check if already connected
        {
            let conn_ref = self.connection.get_ref();
            if let Some(conn) = conn_ref.as_ref() {
                return Ok(conn.clone());
            }
        }

        // Accept the incoming connection
        let incoming = {
            let incoming_ref = self.incoming.get_mut();
            incoming_ref.take()
                .ok_or_else(|| "Connection already established".to_string())?
        };

        let conn = incoming.accept()
            .map_err(|e| format!("Failed to accept connection: {}", e))?
            .await
            .map_err(|e| format!("Failed to establish connection: {}", e))?;

        // Store the connection
        {
            let conn_ref = self.connection.get_mut();
            *conn_ref = Some(conn.clone());
        }

        Ok(conn)
    }
}

#[php_impl]
impl AsyncQuicConnection {
    /// Get the remote address of this connection
    #[php]
    pub fn remote_addr(&self) -> PhpResult<String> {
        let conn_ref = self.connection.get_ref();
        let conn = conn_ref.as_ref()
            .ok_or_else(|| PhpException::default("Connection not yet established".to_string()))?;

        Ok(conn.remote_address().to_string())
    }

    /// Close the connection
    #[php]
    pub fn close(&mut self, error_code: Option<i64>, reason: Option<String>) {
        let conn_ref = self.connection.get_mut();
        if let Some(conn) = conn_ref.as_ref() {
            let code = quinn::VarInt::from_u32(error_code.unwrap_or(0) as u32);
            conn.close(
                code,
                reason.unwrap_or_default().as_bytes()
            );
        }
        *conn_ref = None;
    }
}
