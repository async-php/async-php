/// Core IO interfaces inspired by Go's io package
/// This module defines the fundamental IO interface traits that can be
/// implemented by various components in the async-php runtime

use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::class::RegisteredClass;

fn get_reader_ce() -> &'static ClassEntry {
    PhpInterfaceReader::get_metadata().ce()
}

fn get_writer_ce() -> &'static ClassEntry {
    PhpInterfaceWriter::get_metadata().ce()
}

fn get_closer_ce() -> &'static ClassEntry {
    PhpInterfaceCloser::get_metadata().ce()
}

fn get_seeker_ce() -> &'static ClassEntry {
    PhpInterfaceSeeker::get_metadata().ce()
}

fn get_byte_reader_ce() -> &'static ClassEntry {
    PhpInterfaceByteReader::get_metadata().ce()
}

/// Reader is the interface that wraps the basic Read method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\Reader")]
#[allow(dead_code)]
pub trait Reader {
    /// Read reads up to length bytes. It returns the data read as string and any error encountered.
    /// Even if Read returns length < requested, it may use all of buffer as scratch space.
    /// If some data is available but not length bytes, Read conventionally returns what is available.
    fn read(&mut self, length: i64) -> PhpResult<Option<String>>;
}

/// Writer is the interface that wraps the basic Write method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\Writer")]
#[allow(dead_code)]
pub trait Writer {
    /// Write writes data bytes to the underlying data stream.
    /// It returns the number of bytes written and any error encountered that caused the write to stop early.
    fn write(&mut self, data: String) -> PhpResult<i64>;

    /// Flush writes any buffered data to the underlying io.Writer.
    fn flush(&mut self) -> PhpResult<()>;
}

/// Closer is the interface that wraps the basic Close method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\Closer")]
#[allow(dead_code)]
pub trait Closer {
    /// Close closes the underlying resource, returns true on success
    fn close(&mut self) -> PhpResult<bool>;
}

/// ReaderAt is the interface that wraps the basic ReadAt method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ReaderAt")]
#[allow(dead_code)]
pub trait ReaderAt {
    /// ReadAt reads data starting at offset in the underlying input source.
    /// It returns the data read as string and any error encountered.
    fn read_at(&self, offset: i64, length: i64) -> PhpResult<Option<String>>;
}

/// WriterAt is the interface that wraps the basic WriteAt method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\WriterAt")]
#[allow(dead_code)]
pub trait WriterAt {
    /// WriteAt writes data to the underlying data stream at offset.
    /// It returns the number of bytes written and any error encountered.
    fn write_at(&self, offset: i64, data: String) -> PhpResult<i64>;
}

/// Seeker is the interface that wraps the basic Seek method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\Seeker")]
#[allow(dead_code)]
pub trait Seeker {
    /// Seek sets the offset for the next Read or Write to offset, interpreted
    /// according to whence: SeekStart means relative to the start of the file,
    /// SeekCurrent means relative to the current offset, and SeekEnd means
    /// relative to the end. Seek returns the new offset relative to the start of
    /// the file and an error, if any.
    fn seek(&mut self, offset: i64, whence: i64) -> PhpResult<i64>;
}

/// ReaderFrom is the interface that wraps the ReadFrom method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ReaderFrom")]
#[allow(dead_code)]
pub trait ReaderFrom {
    /// ReadFrom reads data from r until EOF or error.
    /// The return value is the number of bytes read.
    fn read_from(&mut self, reader: &mut Zval) -> PhpResult<i64>;
}

/// WriterTo is the interface that wraps the WriteTo method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\WriterTo")]
#[allow(dead_code)]
pub trait WriterTo {
    /// WriteTo writes data to w until there's no more data to write or when an error occurs.
    /// The return value is the number of bytes written.
    fn write_to(&self, writer: &mut Zval) -> PhpResult<i64>;
}

/// ByteReader is the interface that wraps the ReadByte method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ByteReader")]
#[allow(dead_code)]
pub trait ByteReader {
    /// ReadByte reads and returns a single byte. If no byte is available, returns -1.
    fn read_byte(&mut self) -> PhpResult<i32>;
}

/// StringReader is the interface that wraps the ReadString method.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\StringReader")]
#[allow(dead_code)]
pub trait StringReader {
    /// ReadString reads until the first occurrence of delim in the input,
    /// returning a string containing the data up to and including the delimiter.
    fn read_string(&mut self, delim: i32) -> PhpResult<String>;
}

// ==========================================
// Composite / Derived Interfaces
// ==========================================

/// ReadCloser is the interface that combines Reader and Closer.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ReadCloser")]
#[php(extends(ce = get_reader_ce, stub = "Async\\Kernel\\IO\\Reader"))]
#[php(extends(ce = get_closer_ce, stub = "Async\\Kernel\\IO\\Closer"))]
#[allow(dead_code)]
pub trait ReadCloser: Reader + Closer {}

/// WriteCloser is the interface that combines Writer and Closer.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\WriteCloser")]
#[php(extends(ce = get_writer_ce, stub = "Async\\Kernel\\IO\\Writer"))]
#[php(extends(ce = get_closer_ce, stub = "Async\\Kernel\\IO\\Closer"))]
#[allow(dead_code)]
pub trait WriteCloser: Writer + Closer {}

/// ReadSeeker is the interface that combines Reader and Seeker.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ReadSeeker")]
#[php(extends(ce = get_reader_ce, stub = "Async\\Kernel\\IO\\Reader"))]
#[php(extends(ce = get_seeker_ce, stub = "Async\\Kernel\\IO\\Seeker"))]
#[allow(dead_code)]
pub trait ReadSeeker: Reader + Seeker {}

/// WriteSeeker is the interface that combines Writer and Seeker.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\WriteSeeker")]
#[php(extends(ce = get_writer_ce, stub = "Async\\Kernel\\IO\\Writer"))]
#[php(extends(ce = get_seeker_ce, stub = "Async\\Kernel\\IO\\Seeker"))]
#[allow(dead_code)]
pub trait WriteSeeker: Writer + Seeker {}

/// ReadWriter is the interface that combines Reader and Writer.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ReadWriter")]
#[php(extends(ce = get_reader_ce, stub = "Async\\Kernel\\IO\\Reader"))]
#[php(extends(ce = get_writer_ce, stub = "Async\\Kernel\\IO\\Writer"))]
#[allow(dead_code)]
pub trait ReadWriter: Reader + Writer {}

/// ReadWriteSeeker is the interface that combines Reader, Writer, and Seeker.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ReadWriteSeeker")]
#[php(extends(ce = get_reader_ce, stub = "Async\\Kernel\\IO\\Reader"))]
#[php(extends(ce = get_writer_ce, stub = "Async\\Kernel\\IO\\Writer"))]
#[php(extends(ce = get_seeker_ce, stub = "Async\\Kernel\\IO\\Seeker"))]
#[allow(dead_code)]
pub trait ReadWriteSeeker: Reader + Writer + Seeker {}

/// ByteScanner is the interface that adds UnreadByte and ReadBytes methods.
#[php_interface]
#[php(name = "Async\\Kernel\\IO\\ByteScanner")]
#[php(extends(ce = get_byte_reader_ce, stub = "Async\\Kernel\\IO\\ByteReader"))]
#[allow(dead_code)]
pub trait ByteScanner: ByteReader {
    /// UnreadByte unreads the last byte. Only the immediately previous byte can be unread.
    fn unread_byte(
        &mut self
    ) -> PhpResult<()>;
    /// ReadBytes reads until the first occurrence of delim in the input,
    /// returning a string containing the data up to and including the delimiter.
    fn read_bytes(&mut self, delim: i32) -> PhpResult<String>;
}
