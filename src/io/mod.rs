/// Core IO utilities for async-php
///
/// This module provides Rust types that wrap tokio IO traits and
/// bridge them to PHP through ext-php-rs

// Public modules
pub mod traits;
pub mod wrappers;
pub mod combined;
pub mod shared_impl;
pub mod cast;

// Internal modules
mod php_bridge;
mod php_io;

// Re-export main types for public API
pub use traits::{AsyncReadSeek, AsyncReadWrite, AsyncReadWriteSeek};
pub use wrappers::{AsyncReader, AsyncSeeker, AsyncWriter};
pub use combined::{AsyncBufReader, AsyncReadSeeker, AsyncReadWriter, AsyncReadWriteSeeker, AsyncWriteSeeker};
pub use cast::cast_io;
pub use php_io::{
    PhpBufReader, PhpReadSeeker, PhpReadWriteSeeker, PhpReadWriter, PhpReader, PhpSeeker,
    PhpWriteSeeker, PhpWriter,
};
