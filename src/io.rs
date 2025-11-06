/// Core IO utilities for async-php
/// This module provides Rust types that wrap tokio IO traits

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeek, AsyncBufRead, AsyncReadExt, AsyncWriteExt, AsyncSeekExt, AsyncBufReadExt};
use std::io::SeekFrom;

/// AsyncReader wraps Box<dyn AsyncRead + Unpin>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncReader")]
pub struct AsyncReader {
    inner: Box<dyn AsyncRead + Unpin>,
}

#[php_impl]
impl AsyncReader {
    /// Read up to length bytes
    pub fn read(&mut self, length: i64) -> RustFuture {
        let mut buf = vec![0u8; length as usize];

        // We need to move the reader, but we can't because self is borrowed
        // Solution: Use a placeholder and swap
        let mut reader = Box::new(tokio::io::empty()) as Box<dyn AsyncRead + Unpin>;
        std::mem::swap(&mut self.inner, &mut reader);

        let future = async move {
            let n = reader.read(&mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            buf.truncate(n);
            let s = String::from_utf8_lossy(&buf).to_string();
            let mut z = Zval::new();
            z.set_string(&s, false)
                .map_err(|e| format!("set_string error: {:?}", e))?;
            Ok(z)
        };

        RustFuture::new(future)
    }
}

/// AsyncWriter wraps Box<dyn AsyncWrite + Unpin>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncWriter")]
pub struct AsyncWriter {
    inner: Box<dyn AsyncWrite + Unpin>,
}

#[php_impl]
impl AsyncWriter {
    /// Write data
    pub fn write(&mut self, data: String) -> RustFuture {
        let mut writer = Box::new(tokio::io::sink()) as Box<dyn AsyncWrite + Unpin>;
        std::mem::swap(&mut self.inner, &mut writer);

        let future = async move {
            let bytes = data.as_bytes();
            writer.write_all(bytes).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(bytes.len() as i64);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }

    /// Flush buffered data
    pub fn flush(&mut self) -> RustFuture {
        let mut writer = Box::new(tokio::io::sink()) as Box<dyn AsyncWrite + Unpin>;
        std::mem::swap(&mut self.inner, &mut writer);

        let future = async move {
            writer.flush().await.map_err(|e| e.to_string())?;
            let mut z = Zval::new();
            z.set_bool(true);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }
}

/// AsyncSeeker wraps Box<dyn AsyncSeek + Unpin>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncSeeker")]
pub struct AsyncSeeker {
    inner: Box<dyn AsyncSeek + Unpin>,
}

#[php_impl]
impl AsyncSeeker {
    /// Seek to a position
    pub fn seek(&mut self, offset: i64, whence: i64) -> RustFuture {
        let mut seeker = Box::new(tokio::io::empty()) as Box<dyn AsyncSeek + Unpin>;
        std::mem::swap(&mut self.inner, &mut seeker);

        let future = async move {
            let seek_from = match whence {
                0 => SeekFrom::Start(offset as u64),
                1 => SeekFrom::Current(offset),
                2 => SeekFrom::End(offset),
                _ => SeekFrom::Start(offset as u64),
            };

            let new_pos = seeker.seek(seek_from).await.map_err(|e| e.to_string())?;

            let mut z = Zval::new();
            z.set_long(new_pos as i64);
            Ok::<Zval, String>(z)
        };

        RustFuture::new(future)
    }
}

/// AsyncBufReader wraps Box<dyn AsyncBufRead + Unpin>
#[php_class]
#[php(name = "Async\\Kernel\\IO\\AsyncBufReader")]
pub struct AsyncBufReader {
    inner: Box<dyn AsyncBufRead + Unpin>,
}

#[php_impl]
impl AsyncBufReader {
    /// Read a line
    pub fn read_line(&mut self) -> RustFuture {
        let mut reader = Box::new(tokio::io::empty()) as Box<dyn AsyncBufRead + Unpin>;
        std::mem::swap(&mut self.inner, &mut reader);

        let future = async move {
            let mut line = String::new();
            let n = reader.read_line(&mut line).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            let mut z = Zval::new();
            z.set_string(&line, false)
                .map_err(|e| format!("set_string error: {:?}", e))?;
            Ok(z)
        };

        RustFuture::new(future)
    }

    /// Read until delimiter
    pub fn read_until(&mut self, delim: u8) -> RustFuture {
        let mut reader = Box::new(tokio::io::empty()) as Box<dyn AsyncBufRead + Unpin>;
        std::mem::swap(&mut self.inner, &mut reader);

        let future = async move {
            let mut buf = Vec::new();
            let n = reader.read_until(delim, &mut buf).await.map_err(|e| e.to_string())?;

            if n == 0 {
                return Ok::<Zval, String>(Zval::null());
            }

            let s = String::from_utf8_lossy(&buf).to_string();
            let mut z = Zval::new();
            z.set_string(&s, false)
                .map_err(|e| format!("set_string error: {:?}", e))?;
            Ok(z)
        };

        RustFuture::new(future)
    }
}
