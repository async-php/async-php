/// HTTP Response Body implementation using io.rs abstractions
///
/// Note: reqwest automatically decompresses gzip/deflate/brotli responses by default,
/// so we don't need to handle decompression manually.

use crate::io::AsyncReader;
use crate::future::RustFuture;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use tokio::io::AsyncReadExt;

/// HTTP Response Body wrapper for streaming response data
///
/// This class wraps an HTTP response body and provides async methods to read data.
/// Decompression is handled automatically by reqwest.
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\HttpResponseBody")]
pub struct HttpResponseBody {
    reader: AsyncReader,
}

impl HttpResponseBody {
    /// Create response body from reqwest Response
    ///
    /// Note: reqwest's bytes_stream() already returns decompressed data
    /// when automatic decompression is enabled (which is the default).
    pub fn from_reqwest(response: reqwest::Response) -> Self {
        use futures::StreamExt;

        // Convert reqwest body stream to bytes stream
        // reqwest automatically decompresses the stream, so we just need to convert it to AsyncRead
        let stream = response.bytes_stream().map(|result| {
            result.map_err(std::io::Error::other)
        });

        let stream_reader = tokio_util::io::StreamReader::new(stream);

        Self {
            reader: AsyncReader::new(Box::new(stream_reader)),
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
        let reader = self.reader.get_inner();

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
