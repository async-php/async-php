use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// --- Unix Listener ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\UnixListener")]
pub struct AsyncUnixListener {
    inner: Shared<tokio::net::UnixListener>,
}

#[php_impl]
impl AsyncUnixListener {
    /// Bind a Unix domain socket listener to the specified path
    pub fn bind(path: String) -> PhpResult<RustFuture> {
        let future = async move {
            let path_clone = path.clone();
            // Remove file if it exists to avoid EADDRINUSE
            let _ = tokio::fs::remove_file(&path_clone).await;

            let listener = tokio::net::UnixListener::bind(&path_clone).map_err(|e| e.to_string())?;
            let obj = AsyncUnixListener { inner: Shared::new(listener) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUnixListener to Zval: {:?}", e))
        };
        Ok(RustFuture::new(future))
    }

    /// Accept a new incoming connection
    pub fn accept(&self) -> RustFuture {
        let listener = self.inner.clone();
        let future = async move {
            let (stream, _addr) = listener.get_ref().accept().await.map_err(|e| e.to_string())?;
            let obj = AsyncUnixStream { inner: Shared::new(stream) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUnixStream to Zval: {:?}", e))
        };
        RustFuture::new(future)
    }

    /// Get the local socket path this listener is bound to
    pub fn local_addr(&self) -> String {
        self.inner.get_ref()
            .local_addr()
            .ok()
            .and_then(|addr| {
                addr.as_pathname().map(|p| p.to_string_lossy().to_string())
            })
            .unwrap_or_default()
    }
}

// --- Unix Stream ---

#[php_class]
#[php(name = "Async\\Kernel\\Network\\UnixStream")]
pub struct AsyncUnixStream {
    inner: Shared<tokio::net::UnixStream>,
}

#[php_impl]
impl AsyncUnixStream {
    /// Connect to a Unix domain socket at the specified path
    pub fn connect(path: String) -> PhpResult<RustFuture> {
        let future = async move {
            let stream = tokio::net::UnixStream::connect(&path).await.map_err(|e| e.to_string())?;
            let obj = AsyncUnixStream { inner: Shared::new(stream) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Failed to convert AsyncUnixStream to Zval: {:?}", e))
        };
        Ok(RustFuture::new(future))
    }

    /// Read up to length bytes from the stream
    pub fn read(&self, length: usize) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];

            let n = stream.get_mut().read(&mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::new());
            }

            buf.truncate(n);
            let mut z = Zval::new();
            z.set_binary(buf);
            Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Write all bytes from data to the stream
    pub fn write(&self, data: String) -> RustFuture {
        let stream = self.inner.clone();
        let future = async move {
            stream.get_mut().write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(data.len() as i64);
            Ok::<Zval, String>(z)
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
             Ok::<Zval, String>(z)
        };
        RustFuture::new(future)
    }

    /// Get the local socket address
    pub fn local_addr(&self) -> String {
        self.inner.get_ref()
            .local_addr()
            .ok()
            .and_then(|addr| {
                addr.as_pathname().map(|p| p.to_string_lossy().to_string())
            })
            .unwrap_or_default()
    }

    /// Get the remote peer socket address
    pub fn peer_addr(&self) -> String {
        self.inner.get_ref()
            .peer_addr()
            .ok()
            .and_then(|addr| {
                addr.as_pathname().map(|p| p.to_string_lossy().to_string())
            })
            .unwrap_or_default()
    }

    /// Get peer credentials (process ID, user ID, group ID)
    /// Returns an array with keys: pid, uid, gid, or null if not supported/error
    pub fn peer_cred(&self) -> Zval {
        #[cfg(unix)]
        {
            match self.inner.get_ref().peer_cred() {
                Ok(cred) => {
                    let mut map = ext_php_rs::types::ZendHashTable::new();
                    map.insert("pid", cred.pid().unwrap_or(0) as i64).ok();
                    map.insert("uid", cred.uid() as i64).ok();
                    map.insert("gid", cred.gid() as i64).ok();
                    return map.into_zval(false).unwrap_or_else(|_| Zval::new());
                },
                Err(_) => {}
            }
        }
        Zval::new()
    }

    /// Extract as AsyncReader (returns \Async\Kernel\IO\AsyncReader)
    #[php]
    pub fn as_reader(&self) -> crate::io::AsyncReader {
        use crate::io::{AsyncReader, SharedAsyncRead};
        let wrapper = SharedAsyncRead::new(self.inner.clone());
        let trait_object: Shared<Box<dyn tokio::io::AsyncRead + Unpin + Send>> =
            Shared::new(Box::new(wrapper));
        AsyncReader::from_shared(trait_object)
    }

    /// Extract as AsyncWriter (returns \Async\Kernel\IO\AsyncWriter)
    #[php]
    pub fn as_writer(&self) -> crate::io::AsyncWriter {
        use crate::io::{AsyncWriter, SharedAsyncWrite};
        let wrapper = SharedAsyncWrite::new(self.inner.clone());
        let trait_object: Shared<Box<dyn tokio::io::AsyncWrite + Unpin + Send>> =
            Shared::new(Box::new(wrapper));
        AsyncWriter::from_shared(trait_object)
    }
}
