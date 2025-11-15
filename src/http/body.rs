/// HTTP Response Body implementation using io.rs abstractions
/// This module provides streaming HTTP body handling with automatic decompression

use crate::io::AsyncReader;
use crate::future::RustFuture;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use tokio::io::{AsyncRead, AsyncReadExt};
use async_compression::tokio::bufread::{GzipDecoder, DeflateDecoder};

/// HTTP Response Body wrapper for streaming response data
///
/// This class wraps an HTTP response body and provides async methods to read data.
/// It automatically handles decompression for gzip and deflate encodings.
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponseBody")]
pub struct HttpResponseBody {
    reader: AsyncReader,
}

impl HttpResponseBody {
    /// Create response body from reqwest Response
    pub fn from_reqwest(response: reqwest::Response, content_encoding: Option<&str>) -> Self {
        use futures::StreamExt;

        // Convert reqwest body stream to bytes stream
        let stream = response.bytes_stream().map(|result| {
            result.map_err(std::io::Error::other)
        });

        let stream_reader = tokio_util::io::StreamReader::new(stream);

        // Wrap with decompressor if needed
        let reader: Box<dyn AsyncRead + Unpin> = match content_encoding {
            Some("gzip") => {
                let buffered = tokio::io::BufReader::new(stream_reader);
                Box::new(GzipDecoder::new(buffered))
            }
            Some("deflate") => {
                let buffered = tokio::io::BufReader::new(stream_reader);
                Box::new(DeflateDecoder::new(buffered))
            }
            _ => Box::new(stream_reader),
        };

        Self {
            reader: AsyncReader::new(reader),
        }
    }

    /// Get the underlying AsyncReader for internal use
    pub fn as_reader(&self) -> &AsyncReader {
        &self.reader
    }
}

#[php_impl]
impl HttpResponseBody {
    /// Read up to `length` bytes from the response body
    ///
    /// Returns a Future that resolves to:
    /// - A binary string with the data read (may be less than length bytes)
    /// - null if EOF is reached
    ///
    /// # Example (PHP)
    /// ```php
    /// $data = await $body->read(8192);
    /// if ($data === null) {
    ///     echo "EOF reached\n";
    /// }
    /// ```
    pub fn read(&mut self, length: i64) -> RustFuture {
        self.reader.read(length)
    }

    /// Read all remaining data from the response body
    ///
    /// Returns a Future that resolves to:
    /// - A binary string with all remaining data
    /// - null if no data available (already at EOF)
    ///
    /// # Example (PHP)
    /// ```php
    /// $allData = await $body->read_all();
    /// ```
    pub fn read_all(&mut self) -> RustFuture {
        let reader = self.reader.as_tokio();

        RustFuture::new(async move {
            let mut buf = Vec::new();
            reader.get_mut().read_to_end(&mut buf).await
                .map_err(|e| e.to_string())?;

            if buf.is_empty() {
                return Ok::<Zval, String>(Zval::null());
            }

            let mut z = Zval::new();
            z.set_binary(buf);
            Ok(z)
        })
    }

    /// Close the response body
    ///
    /// This is a no-op since the reader will be automatically dropped.
    /// Provided for API compatibility.
    pub fn close(&self) -> bool {
        true
    }
}

/// Adapter to read from PHP AsyncReader as tokio::io::AsyncRead
pub struct PhpBodyReader {
    reader: Zval,
    chunk_size: usize,
}

impl PhpBodyReader {
    pub(crate) fn new(reader: Zval) -> Self {
        Self {
            reader,
            chunk_size: 8192,
        }
    }
}

impl tokio::io::AsyncRead for PhpBodyReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let to_read = std::cmp::min(buf.remaining(), this.chunk_size) as i64;

        let mut length_zval = Zval::new();
        length_zval.set_long(to_read);

        let data_zval = this
            .reader
            .try_call_method("read", vec![&length_zval])
            .map_err(|e| std::io::Error::other(format!("PHP read failed: {:?}", e)))?;

        if data_zval.is_null() {
            return std::task::Poll::Ready(Ok(()));
        }

        if let Some(bytes) = data_zval.binary() {
            buf.put_slice(&bytes);
        } else if let Some(s) = data_zval.str() {
            buf.put_slice(s.as_bytes());
        } else {
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Read did not return string or binary",
            )));
        }

        std::task::Poll::Ready(Ok(()))
    }
}

unsafe impl Send for PhpBodyReader {}
unsafe impl Sync for PhpBodyReader {}
