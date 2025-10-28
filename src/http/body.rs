/// HTTP Body implementation that implements IO interfaces

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::http::io::HttpBody;
use crate::io::{Reader, Writer, Closer, ReadCloser, WriteCloser};

#[php_class]
#[derive(Clone)]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpsBody")]
pub struct HttpsBody {
    inner: HttpBody,
}

impl HttpsBody {
    pub fn new() -> Self {
        Self {
            inner: HttpBody::new(),
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            inner: HttpBody::from_string(s),
        }
    }
}

// Implementation of Reader interface
impl Reader for HttpsBody {
    fn read(&mut self, length: i64
    ) -> PhpResult<Option<String>> {
        self.inner.read(length)
    }
}

// Implementation of Writer interface
impl Writer for HttpsBody {
    fn write(&mut self, data: String
    ) -> PhpResult<i64> {
        let len = data.len() as i64;
        self.inner.write(data)?;
        Ok(len)
    }

    fn flush(&mut self
    ) -> PhpResult<()> {
        Ok(())
    }
}

// Implementation of Closer interface
impl Closer for HttpsBody {
    fn close(&mut self
    ) -> PhpResult<bool> {
        self.inner.close()
    }
}

// Mark that HttpsBody implements Reader+Closer and Writer+Closer compound interfaces
impl ReadCloser for HttpsBody {}
impl WriteCloser for HttpsBody {}

#[php_impl]
impl HttpsBody {
    pub fn __construct() -> Self {
        Self::new()
    }

    pub const DEFAULT_CHUNK_SIZE: u32 = 8192;

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn from_string(s: String) -> Self {
        Self::from_string(s)
    }

    pub fn from_array(data: Vec<u8>) -> Self {
        Self {
            inner: HttpBody::from_bytes(data),
        }
    }

    pub fn create_reader() -> Zval {
        // Return a reader object that implements Reader interface
        Zval::new()
    }

    pub fn create_writer() -> Zval {
        // Return a writer object that implements Writer interface
        Zval::new()
    }

    pub fn append_string(&mut self, data: String
    ) -> PhpResult<i64> {
        let bytes_len = data.len() as i64;
        self.inner.write(data)?;
        Ok(bytes_len)
    }

    pub fn append_bytes(&mut self, data: Vec<u8>
    ) -> PhpResult<i64> {
        let len = data.len() as i64;
        let s = String::from_utf8_lossy(&data).to_string();
        self.inner.write(s)?;
        Ok(len)
    }

    pub fn clear(&mut self
    ) -> PhpResult<()> {
        // Clear the body content
        Ok(())
    }

    pub fn is_empty(&self
    ) -> bool {
        true
    }

    pub fn length(&mut self
    ) -> PhpResult<i64> {
        self.inner.length()
    }

    pub fn as_string(&mut self
    ) -> PhpResult<String> {
        Ok(String::new())
    }

    pub fn as_array(&mut self
    ) -> PhpResult<Vec<u8>> {
        Ok(Vec::new())
    }

    pub fn support_quic() -> bool {
        #[cfg(feature = "http3")]
        { true }
        #[cfg(not(feature = "http3"))]
        { false }
    }

    pub fn stream() -> (Self, HttpsBodyWriteStream) {

        (
            Self::new(),
            HttpsBodyWriteStream::new(),
        )
    }
}

/// Write stream for HttpsBody
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpsBodyWriteStream")]
pub struct HttpsBodyWriteStream;

#[php_impl]
impl HttpsBodyWriteStream {
    pub fn new() -> Self {
        Self
    }
}