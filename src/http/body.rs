/// HTTP Response Body implementation using io.rs abstractions
/// This module provides streaming HTTP body handling with automatic decompression

use crate::io::AsyncReader;
use crate::future::RustFuture;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use hyper::body::Incoming;
use tokio::io::{AsyncRead, AsyncReadExt};
use async_compression::tokio::bufread::{GzipDecoder, DeflateDecoder};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Helper function to convert hyper frame to bytes
fn frame_to_bytes(
    result: Result<hyper::body::Frame<bytes::Bytes>, hyper::Error>,
) -> Result<bytes::Bytes, std::io::Error> {
    result
        .map(|frame| frame.into_data().unwrap_or_default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

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
    /// Create a new response body from hyper Incoming body
    ///
    /// # Arguments
    /// * `body` - The incoming HTTP body stream
    /// * `content_encoding` - Optional encoding: "gzip", "deflate", or None
    pub fn new_internal(body: Incoming, content_encoding: Option<&str>) -> Self {
        use tokio_util::io::StreamReader;
        use http_body_util::BodyStream;
        use futures::StreamExt;

        // Convert hyper body to tokio AsyncRead stream
        let body_stream = BodyStream::new(body);
        let frame_mapper: fn(Result<hyper::body::Frame<bytes::Bytes>, hyper::Error>) -> Result<bytes::Bytes, std::io::Error> = frame_to_bytes;
        let stream_reader = StreamReader::new(body_stream.map(frame_mapper));

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

// ==================== Hyper Body Adapters ====================

/// AsyncReadBody converts any AsyncRead into a hyper Body
///
/// This adapter allows streaming data from any AsyncRead source
/// (including PHP readers via PhpReader) into HTTP requests/responses.
#[allow(dead_code)]
pub struct AsyncReadBody<R> {
    reader: R,
    chunk_size: usize,
}

impl<R: AsyncRead + Unpin> AsyncReadBody<R> {
    /// Create a new AsyncReadBody with default chunk size (8KB)
    #[allow(dead_code)]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            chunk_size: 8192,
        }
    }

    /// Create a new AsyncReadBody with custom chunk size
    #[allow(dead_code)]
    pub fn with_chunk_size(reader: R, chunk_size: usize) -> Self {
        Self { reader, chunk_size }
    }
}

impl<R: AsyncRead + Unpin> hyper::body::Body for AsyncReadBody<R> {
    type Data = bytes::Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        use tokio::io::ReadBuf;

        let mut buf = vec![0u8; self.chunk_size];
        let mut read_buf = ReadBuf::new(&mut buf);

        match Pin::new(&mut self.reader).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let n = read_buf.filled().len();
                if n == 0 {
                    // EOF reached
                    Poll::Ready(None)
                } else {
                    // Return data frame
                    buf.truncate(n);
                    let bytes = bytes::Bytes::from(buf);
                    Poll::Ready(Some(Ok(hyper::body::Frame::data(bytes))))
                }
            }
            Poll::Ready(Err(e)) => {
                Poll::Ready(Some(Err(Box::new(e))))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// SAFETY: AsyncReadBody is safe to send across threads if R is Send
// This is required by hyper's Body trait
unsafe impl<R: AsyncRead + Unpin> Send for AsyncReadBody<R> {}
unsafe impl<R: AsyncRead + Unpin> Sync for AsyncReadBody<R> {}
