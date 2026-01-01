use ext_php_rs::prelude::*;

#[php_const]
#[php(name = "ASYNC_READ")]
pub const IO_READ: i64 = 1;

#[php_const]
#[php(name = "ASYNC_WRITE")]
pub const IO_WRITE: i64 = 2;

#[php_const]
#[php(name = "ASYNC_SEEK")]
pub const IO_SEEK: i64 = 4;

#[php_const]
#[php(name = "ASYNC_BUF")]
pub const IO_BUF: i64 = 8;

mod future;
mod io;
mod bytes;
mod net;
mod fs;
mod http;
mod channel;
mod pdo;
mod curl;
mod redis;
mod time;
mod util;
mod logger;
mod runtime;


use future::RustFuture;
use io::{
    AsyncReader, AsyncWriter, AsyncSeeker, AsyncBufReader,
    AsyncReadWriter, AsyncReadSeeker, AsyncWriteSeeker, AsyncReadWriteSeeker,
};
use bytes::{BytesReader, BytesWriter};
use net::{AsyncTcpListener, AsyncTcpStream, AsyncTlsConfig, AsyncTlsStream, AsyncUdpSocket, AsyncUnixListener, AsyncUnixStream, AsyncQuicListener, AsyncQuicConnection, AsyncQuicStream, AsyncQuicRecvStream, AsyncQuicSendStream};
use fs::{AsyncFilesystem, AsyncFileHandle};
use channel::AsyncChannel;
use pdo::{
    AsyncPdoMySql, AsyncPdoPgSql, AsyncPdoMySqlTransaction, AsyncPdoPgSqlTransaction
};
use curl::{CurlHandle, CurlMulti};
use redis::AsyncRedisClient;
use time::{AsyncTime, AsyncTicker};
use logger::AsyncLogger;
use runtime::context::AsyncContext;

// Export PHP IO bridge types for external use
pub use io::PhpIo;

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    let module = module
        .class::<AsyncContext>()
        .class::<RustFuture>()
        .class::<AsyncTcpListener>()
        .class::<AsyncTcpStream>()
        .class::<AsyncUdpSocket>()
        .class::<AsyncUnixListener>()
        .class::<AsyncUnixStream>()
        .class::<AsyncTlsConfig>()
        .class::<AsyncTlsStream>()
        .class::<AsyncQuicListener>()
        .class::<AsyncQuicConnection>()
        .class::<AsyncQuicStream>()
        .class::<AsyncQuicRecvStream>()
        .class::<AsyncQuicSendStream>()
        .class::<AsyncFilesystem>()
        .class::<AsyncFileHandle>()
        .class::<http::HttpClient>()
        .class::<http::HttpRequest>()
        .class::<http::HttpResponse>()
        .class::<http::HttpServer>()
        .class::<http::Http3Server>()
        .class::<http::AsyncSocketIo>()
        .class::<http::AsyncSocket>()
        .class::<AsyncChannel>()
        .class::<AsyncPdoMySql>()
        .class::<AsyncPdoMySqlTransaction>()
        .class::<AsyncPdoPgSql>()
        .class::<AsyncPdoPgSqlTransaction>()
        .class::<CurlHandle>() // cURL support
        .class::<CurlMulti>()
        .class::<AsyncRedisClient>() // Redis support
        .class::<AsyncTime>()
        .class::<AsyncTicker>() // Ticker for periodic operations
        .class::<AsyncLogger>() // Register AsyncLogger
        .class::<AsyncReader>() // IO types
        .class::<AsyncWriter>()
        .class::<AsyncSeeker>()
        .class::<AsyncBufReader>()
        .class::<AsyncReadWriter>() // Combined IO types
        .class::<AsyncReadSeeker>()
        .class::<AsyncWriteSeeker>()
        .class::<AsyncReadWriteSeeker>()
        .class::<BytesReader>() // In-memory IO
        .class::<BytesWriter>()
        .class::<PhpIo>() // PHP IO bridge
        .class::<runtime::AsyncRuntime>()
        .constant(wrap_constant!(IO_READ))
        .constant(wrap_constant!(IO_WRITE))
        .constant(wrap_constant!(IO_SEEK))
        .constant(wrap_constant!(IO_BUF));
    
    crate::pdo::functions::register(module)
}
