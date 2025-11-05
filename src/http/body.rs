use std::io;
use std::pin::Pin;
/// HTTP Response Body wrapper for streaming response data

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use hyper::body::Incoming;
use http_body_util::BodyExt;
use std::sync::Arc;
use std::task::{Context, Poll};
use ext_php_rs::convert::IntoZval;
use tokio::io::{AsyncRead, ReadBuf};
use tokio::sync::Mutex;
use async_compression::tokio::bufread::{GzipDecoder, DeflateDecoder};

type Result<T> = std::result::Result<T, String>;

/// Wrapper for HTTP response body that allows streaming reads
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponseBody")]
pub struct HttpResponseBody {
    body: Arc<Mutex<Option<Incoming>>>,
    buffer: Arc<Mutex<Vec<u8>>>,
    eof: Arc<Mutex<bool>>,
    content_encoding: Option<String>, // Store encoding for decompression in read_all
}

// SAFETY: Safe because the async runtime is single-threaded
unsafe impl Send for HttpResponseBody {}
unsafe impl Sync for HttpResponseBody {}

impl HttpResponseBody {
    /// Create a new response body wrapper
    /// content_encoding: Optional "gzip" or "deflate" for automatic decompression
    pub fn new_internal(body: Incoming, content_encoding: Option<&str>) -> Self {
        Self {
            body: Arc::new(Mutex::new(Some(body))),
            buffer: Arc::new(Mutex::new(Vec::new())),
            eof: Arc::new(Mutex::new(false)),
            content_encoding: content_encoding.map(|s| s.to_string()),
        }
    }

    /// Decompress data based on content encoding
    async fn decompress(data: Vec<u8>, encoding: &str) -> Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;

        match encoding {
            "gzip" => {
                let cursor = std::io::Cursor::new(data);
                let mut decoder = GzipDecoder::new(tokio::io::BufReader::new(cursor));
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).await
                    .map_err(|e| format!("Gzip decompression failed: {}", e))?;
                Ok(decompressed)
            }
            "deflate" => {
                let cursor = std::io::Cursor::new(data);
                let mut decoder = DeflateDecoder::new(tokio::io::BufReader::new(cursor));
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).await
                    .map_err(|e| format!("Deflate decompression failed: {}", e))?;
                Ok(decompressed)
            }
            _ => Ok(data), // Unknown encoding, return as-is
        }
    }

    /// Convert bytes to Zval string (binary-safe)
    fn bytes_to_zval(data: Vec<u8>) -> Result<Zval> {
        if data.is_empty() {
            return Ok(Zval::null());
        }

        // PHP strings are binary-safe and can store any byte sequence
        // We use from_utf8_unchecked to preserve raw bytes without validation
        // This is necessary for binary data (though now decompressed)
        let mut z = Zval::new();
        unsafe {
            let s = std::str::from_utf8_unchecked(&data);
            z.set_string(s, false)
                .map_err(|e| format!("Failed to create string: {:?}", e))?;
        }
        Ok(z)
    }

    /// Read next frame from body into buffer
    async fn read_frame_into_buffer(
        incoming: &mut Incoming,
        buffer: &mut Vec<u8>,
    ) -> Result<bool> {
        match incoming.frame().await {
            Some(Ok(frame)) => {
                if let Some(chunk) = frame.data_ref() {
                    buffer.extend_from_slice(chunk);
                }
                Ok(true)
            }
            Some(Err(e)) => Err(format!("Error reading response body: {}", e)),
            None => Ok(false), // EOF
        }
    }
}

#[php_impl]
impl HttpResponseBody {
    /// Read data from the response body asynchronously
    /// Returns a Future that resolves to a string, or null if EOF
    /// Note: Returns raw data without decompression (use read_all for automatic decompression)
    #[php]
    pub fn read(&self, length: i64) -> RustFuture {
        let body = self.body.clone();
        let buffer = self.buffer.clone();
        let eof = self.eof.clone();

        RustFuture::new(async move {
            let length = length as usize;

            // Early return if already at EOF
            if *eof.lock().await {
                return Ok::<Zval, String>(Zval::null());
            }

            let mut buf = buffer.lock().await;

            // Return from buffer if enough data is available
            if buf.len() >= length {
                return Self::bytes_to_zval(buf.drain(..length).collect());
            }

            // Need to read more data from the body
            let mut body_guard = body.lock().await;
            let Some(mut incoming) = body_guard.take() else {
                *eof.lock().await = true;
                return Self::bytes_to_zval(buf.drain(..).collect());
            };

            // Read frames until we have enough data or reach EOF
            loop {
                let has_more = Self::read_frame_into_buffer(&mut incoming, &mut buf).await?;

                if !has_more {
                    // EOF reached
                    *eof.lock().await = true;
                    return Self::bytes_to_zval(buf.drain(..).collect());
                }

                // Check if we have enough data now
                if buf.len() >= length {
                    *body_guard = Some(incoming);
                    return Self::bytes_to_zval(buf.drain(..length).collect());
                }
            }
        })
    }

    /// Read all remaining data from the response body
    /// Automatically decompresses if Content-Encoding is gzip or deflate
    #[php]
    pub fn read_all(&self) -> RustFuture {
        let body = self.body.clone();
        let buffer = self.buffer.clone();
        let eof = self.eof.clone();
        let content_encoding = self.content_encoding.clone();

        RustFuture::new(async move {
            let mut buf = buffer.lock().await;
            let mut body_guard = body.lock().await;

            if let Some(incoming) = body_guard.take() {
                let collected = incoming.collect().await
                    .map_err(|e| format!("Error reading response body: {}", e))?;
                buf.extend_from_slice(&collected.to_bytes());
            }

            *eof.lock().await = true;

            // Decompress if needed
            let data = buf.drain(..).collect::<Vec<u8>>();
            let final_data = if let Some(encoding) = content_encoding {
                Self::decompress(data, &encoding).await?
            } else {
                data
            };

            Self::bytes_to_zval(final_data)
        })
    }

    /// Close the response body
    #[php]
    pub fn close(&self) -> bool {
        true
    }
}

/// AsyncRead adapter for PHP Reader interface
/// This allows streaming data from PHP without loading everything into memory
pub(crate) struct PhpReaderAdapter {
    reader: Zval,
    chunk_size: usize,
}

// SAFETY: This is safe because the entire async runtime runs on a single thread.
// The Zval will never be accessed from multiple threads concurrently.
unsafe impl Send for PhpReaderAdapter {}
unsafe impl Sync for PhpReaderAdapter {}

impl PhpReaderAdapter {
    pub(crate) fn new(reader: Zval) -> Self {
        Self {
            reader,
            chunk_size: 8192, // 8KB chunks
        }
    }
}

impl AsyncRead for PhpReaderAdapter {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let to_read = std::cmp::min(buf.remaining(), this.chunk_size) as i64;

        // Call PHP's read method (safe because we're on the same thread)
        let data_zval = this.reader
            .try_call_method("read", vec![&to_read.into_zval(false).unwrap()])
            .map_err(|e| io::Error::other(format!("PHP read failed: {:?}", e)))?;

        // EOF check
        if data_zval.is_null() {
            return Poll::Ready(Ok(()));
        }

        // Extract string data
        let data_str = data_zval.string()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Read did not return string"))?;

        buf.put_slice(data_str.as_bytes());
        Poll::Ready(Ok(()))
    }
}

/// Hyper Body adapter for PHP Reader interface
/// This allows streaming HTTP response data from PHP Reader objects
pub(crate) struct PhpReaderBody {
    reader: Zval,
    chunk_size: usize,
    eof: bool,
}

// SAFETY: Safe because the async runtime is single-threaded
unsafe impl Send for PhpReaderBody {}
unsafe impl Sync for PhpReaderBody {}

impl PhpReaderBody {
    pub(crate) fn new(reader: Zval) -> Self {
        Self {
            reader,
            chunk_size: 8192, // 8KB chunks
            eof: false,
        }
    }
}

impl hyper::body::Body for PhpReaderBody {
    type Data = bytes::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<std::result::Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();

        // Already at EOF
        if this.eof {
            return Poll::Ready(None);
        }

        // Call PHP's read method
        let length_zval = (this.chunk_size as i64).into_zval(false)
            .map_err(|e| Box::new(std::io::Error::other(format!("Failed to create length zval: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        let data_zval = this.reader
            .try_call_method("read", vec![&length_zval])
            .map_err(|e| Box::new(std::io::Error::other(format!("PHP read failed: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        // Check for EOF (null return)
        if data_zval.is_null() {
            this.eof = true;
            return Poll::Ready(None);
        }

        // Extract string data
        let data_str = data_zval.string()
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Read did not return string")) as Box<dyn std::error::Error + Send + Sync>)?;

        // Empty string also signals EOF
        if data_str.is_empty() {
            this.eof = true;
            return Poll::Ready(None);
        }

        // Convert to Bytes and wrap in Frame
        let bytes = bytes::Bytes::copy_from_slice(data_str.as_bytes());
        Poll::Ready(Some(Ok(hyper::body::Frame::data(bytes))))
    }
}
