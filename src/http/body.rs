use crate::future::RustFuture;
use async_compression::tokio::bufread::{DeflateDecoder, GzipDecoder};
use ext_php_rs::convert::IntoZval;
/// HTTP Response Body wrapper for streaming response data

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use hyper::body::Incoming;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tokio::sync::Mutex;

type Result<T> = std::result::Result<T, String>;

use futures::StreamExt;

/// Helper function to convert frame results to byte results
fn frame_to_bytes(
    result: std::result::Result<hyper::body::Frame<bytes::Bytes>, hyper::Error>
) -> std::result::Result<bytes::Bytes, std::io::Error> {
    result
        .map(|frame| frame.into_data().unwrap_or_default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Type alias for the stream reader after converting from Incoming
type BodyStreamReader = tokio_util::io::StreamReader<
    futures::stream::Map<
        http_body_util::BodyStream<Incoming>,
        fn(std::result::Result<hyper::body::Frame<bytes::Bytes>, hyper::Error>) -> std::result::Result<bytes::Bytes, std::io::Error>
    >,
    bytes::Bytes
>;

/// Enum to represent different decompression states
enum DecompressionReader {
    None(BodyStreamReader),
    Gzip(Box<GzipDecoder<tokio::io::BufReader<BodyStreamReader>>>),
    Deflate(Box<DeflateDecoder<tokio::io::BufReader<BodyStreamReader>>>),
}

impl DecompressionReader {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            DecompressionReader::None(reader) => reader.read(buf).await,
            DecompressionReader::Gzip(decoder) => decoder.read(buf).await,
            DecompressionReader::Deflate(decoder) => decoder.read(buf).await,
        }
    }

    async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        match self {
            DecompressionReader::None(reader) => reader.read_to_end(buf).await,
            DecompressionReader::Gzip(decoder) => decoder.read_to_end(buf).await,
            DecompressionReader::Deflate(decoder) => decoder.read_to_end(buf).await,
        }
    }
}

// SAFETY: Safe because the async runtime is single-threaded
unsafe impl Send for DecompressionReader {}
unsafe impl Sync for DecompressionReader {}

/// Wrapper for HTTP response body that allows streaming reads
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponseBody")]
pub struct HttpResponseBody {
    reader: Arc<Mutex<Option<DecompressionReader>>>,
    eof: Arc<Mutex<bool>>,
}

// SAFETY: Safe because the async runtime is single-threaded
unsafe impl Send for HttpResponseBody {}
unsafe impl Sync for HttpResponseBody {}

impl HttpResponseBody {
    /// Create a new response body wrapper
    /// content_encoding: Optional "gzip" or "deflate" for automatic decompression
    pub fn new_internal(body: Incoming, content_encoding: Option<&str>) -> Self {
        use tokio_util::io::StreamReader;
        use http_body_util::BodyStream;

        // Convert Incoming (Body) to Stream, then to StreamReader
        let body_stream = BodyStream::new(body);
        let stream_reader = StreamReader::new(body_stream.map(frame_to_bytes as _));

        // Wrap with decompressor if needed
        let reader = match content_encoding {
            Some("gzip") => {
                let buffered = tokio::io::BufReader::new(stream_reader);
                DecompressionReader::Gzip(Box::new(GzipDecoder::new(buffered)))
            }
            Some("deflate") => {
                let buffered = tokio::io::BufReader::new(stream_reader);
                DecompressionReader::Deflate(Box::new(DeflateDecoder::new(buffered)))
            }
            _ => DecompressionReader::None(stream_reader),
        };

        Self {
            reader: Arc::new(Mutex::new(Some(reader))),
            eof: Arc::new(Mutex::new(false)),
        }
    }

    /// Convert bytes to Zval string (binary-safe)
    fn bytes_to_zval(data: Vec<u8>) -> Result<Zval> {
        if data.is_empty() {
            return Ok(Zval::null());
        }

        // PHP strings are binary-safe and can store any byte sequence
        // We use from_utf8_unchecked to preserve raw bytes without validation
        let mut z = Zval::new();
        unsafe {
            let s = std::str::from_utf8_unchecked(&data);
            z.set_string(s, false)
                .map_err(|e| format!("Failed to create string: {:?}", e))?;
        }
        Ok(z)
    }
}

#[php_impl]
impl HttpResponseBody {
    /// Read data from the response body asynchronously
    /// Returns a Future that resolves to a string, or null if EOF
    /// Now automatically decompresses if Content-Encoding is gzip or deflate
    #[php]
    pub fn read(&self, length: i64) -> RustFuture {
        let reader = self.reader.clone();
        let eof = self.eof.clone();

        RustFuture::new(async move {
            let length = length as usize;

            // Early return if already at EOF
            if *eof.lock().await {
                return Ok::<Zval, String>(Zval::null());
            }

            // Get the reader
            let mut reader_guard = reader.lock().await;
            let Some(reader_mut) = reader_guard.as_mut() else {
                *eof.lock().await = true;
                return Ok(Zval::null());
            };

            // Read data from the decompression reader
            let mut buffer = vec![0u8; length];
            match reader_mut.read(&mut buffer).await {
                Ok(0) => {
                    // EOF reached
                    *eof.lock().await = true;
                    Ok(Zval::null())
                }
                Ok(n) => {
                    // Truncate to actual bytes read
                    buffer.truncate(n);
                    Self::bytes_to_zval(buffer)
                }
                Err(e) => Err(format!("Error reading response body: {}", e))
            }
        })
    }

    /// Read all remaining data from the response body
    /// Automatically decompresses if Content-Encoding is gzip or deflate
    #[php]
    pub fn read_all(&self) -> RustFuture {
        let reader = self.reader.clone();
        let eof = self.eof.clone();

        RustFuture::new(async move {
            // Get the reader
            let mut reader_guard = reader.lock().await;
            let Some(reader_mut) = reader_guard.as_mut() else {
                *eof.lock().await = true;
                return Ok(Zval::null());
            };

            // Read all data from the decompression reader
            let mut buffer = Vec::new();
            match reader_mut.read_to_end(&mut buffer).await {
                Ok(_) => {
                    *eof.lock().await = true;
                    Self::bytes_to_zval(buffer)
                }
                Err(e) => Err(format!("Error reading response body: {}", e))
            }
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
