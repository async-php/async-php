use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::{ServerName, CertificateDer};
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use std::fs::File;
use std::io::{BufReader, Cursor};
use webpki_roots;

// --- TLS Configuration Builder ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TlsConfig")]
pub struct AsyncTlsConfig {
    inner: Arc<ClientConfig>,
}

#[php_impl]
impl AsyncTlsConfig {
    /// Create default TLS config with webpki root certificates
    pub fn default() -> PhpResult<Self> {
        let mut root_store = RootCertStore::empty();

        // Load webpki root certificates
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            inner: Arc::new(config),
        })
    }

    /// Create TLS config without certificate verification (DANGEROUS - only for testing)
    pub fn dangerous_no_verify() -> PhpResult<Self> {
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();

        Ok(Self {
            inner: Arc::new(config),
        })
    }

    /// Create TLS config with custom CA certificate file
    pub fn with_ca_file(ca_file: String) -> PhpResult<Self> {
        let mut root_store = RootCertStore::empty();

        let f = File::open(&ca_file)
            .map_err(|e| format!("Failed to open CA file {}: {}", ca_file, e))?;
        let mut reader = BufReader::new(f);

        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse CA certificates: {}", e))?;

        for cert in certs {
            root_store.add(cert)
                .map_err(|e| format!("Failed to add certificate: {}", e))?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            inner: Arc::new(config),
        })
    }

    /// Create TLS config with custom CA certificate from PEM string
    pub fn with_ca_string(ca_pem: String) -> PhpResult<Self> {
        let mut root_store = RootCertStore::empty();

        let mut reader = Cursor::new(ca_pem.as_bytes());

        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse CA certificates from string: {}", e))?;

        for cert in certs {
            root_store.add(cert)
                .map_err(|e| format!("Failed to add certificate: {}", e))?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            inner: Arc::new(config),
        })
    }

    /// Create TLS config with client certificate authentication
    pub fn with_client_cert(ca_file: Option<String>, cert_file: String, key_file: String) -> PhpResult<Self> {
        let mut root_store = RootCertStore::empty();

        // Load CA certificates if provided
        if let Some(ca_path) = ca_file {
            let f = File::open(&ca_path)
                .map_err(|e| format!("Failed to open CA file {}: {}", ca_path, e))?;
            let mut reader = BufReader::new(f);

            let certs = rustls_pemfile::certs(&mut reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to parse CA certificates: {}", e))?;

            for cert in certs {
                root_store.add(cert)
                    .map_err(|e| format!("Failed to add CA certificate: {}", e))?;
            }
        } else {
            // Use webpki root certificates
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }

        // Load client certificate
        let cert_f = File::open(&cert_file)
            .map_err(|e| format!("Failed to open cert file {}: {}", cert_file, e))?;
        let mut cert_reader = BufReader::new(cert_f);
        let client_certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse client certificate: {}", e))?;

        // Load private key
        let key_f = File::open(&key_file)
            .map_err(|e| format!("Failed to open key file {}: {}", key_file, e))?;
        let mut key_reader = BufReader::new(key_f);
        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| format!("Failed to read private key: {}", e))?
            .ok_or_else(|| "No private key found in file".to_string())?;

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, private_key)
            .map_err(|e| format!("Failed to configure client auth: {}", e))?;

        Ok(Self {
            inner: Arc::new(config),
        })
    }

    /// Create TLS config with client certificate authentication from PEM strings
    pub fn with_client_cert_string(ca_pem: Option<String>, cert_pem: String, key_pem: String) -> PhpResult<Self> {
        let mut root_store = RootCertStore::empty();

        // Load CA certificates if provided
        if let Some(ca_content) = ca_pem {
            let mut reader = Cursor::new(ca_content.as_bytes());

            let certs = rustls_pemfile::certs(&mut reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to parse CA certificates from string: {}", e))?;

            for cert in certs {
                root_store.add(cert)
                    .map_err(|e| format!("Failed to add CA certificate: {}", e))?;
            }
        } else {
            // Use webpki root certificates
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }

        // Load client certificate
        let mut cert_reader = Cursor::new(cert_pem.as_bytes());
        let client_certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse client certificate from string: {}", e))?;

        // Load private key
        let mut key_reader = Cursor::new(key_pem.as_bytes());
        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| format!("Failed to read private key from string: {}", e))?
            .ok_or_else(|| "No private key found in string".to_string())?;

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, private_key)
            .map_err(|e| format!("Failed to configure client auth: {}", e))?;

        Ok(Self {
            inner: Arc::new(config),
        })
    }
}

// Custom certificate verifier that accepts all certificates (DANGEROUS)
#[derive(Debug)]
struct NoVerifier;

impl tokio_rustls::rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: tokio_rustls::rustls::pki_types::UnixTime,
    ) -> Result<tokio_rustls::rustls::client::danger::ServerCertVerified, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
        vec![
            tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA256,
            tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            tokio_rustls::rustls::SignatureScheme::ED25519,
        ]
    }
}

// --- TLS Stream ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\TlsStream")]
pub struct AsyncTlsStream {
    inner: Shared<TlsStream<TcpStream>>,
}

// Internal implementation (not exposed to PHP)
impl AsyncTlsStream {
    /// Internal helper for connecting with config
    async fn connect_with_config_internal(host: String, port: i64, config: &AsyncTlsConfig) -> Zval {
        let connector = TlsConnector::from(config.inner.clone());
        let addr = format!("{}:{}", host, port);

        match TcpStream::connect(&addr).await {
            Ok(tcp) => {
                let server_name = match ServerName::try_from(host.clone()) {
                    Ok(name) => name,
                    Err(_) => return Zval::new(),
                };

                match connector.connect(server_name, tcp).await {
                    Ok(stream) => {
                        let obj = AsyncTlsStream {
                            inner: Shared::new(stream),
                        };
                        ext_php_rs::types::ZendClassObject::new(obj)
                            .into_zval(false)
                            .unwrap_or_else(|_| Zval::new())
                    }
                    Err(_) => Zval::new(),
                }
            }
            Err(_) => Zval::new(),
        }
    }
}

#[php_impl]
impl AsyncTlsStream {
    /// Connect to TLS server with default configuration
    pub fn connect(host: String, port: i64) -> RustFuture {
        let future = async move {
            match AsyncTlsConfig::default() {
                Ok(config) => {
                    Self::connect_with_config_internal(host, port, &config).await
                }
                Err(_e) => {
                    Zval::new() // Failed to create default config
                }
            }
        };
        RustFuture::new(future)
    }

    /// Connect to TLS server with custom configuration
    pub fn connect_with_config(host: String, port: i64, config: &AsyncTlsConfig) -> RustFuture {
        let config = config.inner.clone();
        let future = async move {
            let connector = TlsConnector::from(config);
            let addr = format!("{}:{}", host, port);

            match TcpStream::connect(&addr).await {
                Ok(tcp) => {
                    let server_name = match ServerName::try_from(host.clone()) {
                        Ok(name) => name,
                        Err(_) => return Zval::new(),
                    };

                    match connector.connect(server_name, tcp).await {
                        Ok(stream) => {
                            let obj = AsyncTlsStream {
                                inner: Shared::new(stream),
                            };
                            ext_php_rs::types::ZendClassObject::new(obj)
                                .into_zval(false)
                                .unwrap_or_else(|_| Zval::new())
                        }
                        Err(_) => Zval::new(),
                    }
                }
                Err(_) => Zval::new(),
            }
        };
        RustFuture::new(future)
    }

    /// Read up to length bytes
    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];

            match stream.get_mut().read(&mut buf).await {
                Ok(0) => Ok::<Zval, String>(Zval::null()),
                Ok(n) => {
                    buf.truncate(n);
                    let mut z = Zval::new();
                    z.set_binary(buf);
                    Ok(z)
                }
                Err(e) => Err(e.to_string()),
            }
            .unwrap_or_else(|_| Zval::new())
        };
        RustFuture::new(future)
    }

    /// Write data
    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            match stream.get_mut().write_all(data.as_bytes()).await {
                Ok(_) => {
                    let mut z = Zval::new();
                    z.set_long(data.len() as i64);
                    Ok::<Zval, String>(z)
                }
                Err(e) => Err(e.to_string()),
            }
            .unwrap_or_else(|_| Zval::new())
        };
        RustFuture::new(future)
    }

    /// Shutdown the connection
    pub fn close(&self) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let _ = stream.get_mut().shutdown().await;
            let mut z = Zval::new();
            z.set_bool(true);
            z
        };
        RustFuture::new(future)
    }

    /// Get peer address
    pub fn peer_addr(&self) -> String {
        self.inner
            .get_ref()
            .get_ref()
            .0
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_default()
    }

    /// Get local address
    pub fn local_addr(&self) -> String {
        self.inner
            .get_ref()
            .get_ref()
            .0
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_default()
    }

    /// Get TLS protocol version
    pub fn protocol_version(&self) -> String {
        self.inner
            .get_ref()
            .get_ref()
            .1
            .protocol_version()
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get negotiated cipher suite
    pub fn cipher_suite(&self) -> String {
        self.inner
            .get_ref()
            .get_ref()
            .1
            .negotiated_cipher_suite()
            .map(|cs| cs.suite().as_str().unwrap_or("Unknown").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get SNI hostname (not available after handshake in current rustls version)
    pub fn sni_hostname(&self) -> String {
        // SNI hostname is not available after handshake in rustls 0.23
        // This would require storing it during connection
        String::new()
    }

    /// Check if connection uses ALPN
    pub fn alpn_protocol(&self) -> String {
        self.inner
            .get_ref()
            .get_ref()
            .1
            .alpn_protocol()
            .map(|p| String::from_utf8_lossy(p).to_string())
            .unwrap_or_default()
    }
}
