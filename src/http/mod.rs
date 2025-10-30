/// HTTP module for async-php runtime
/// Provides HTTP client and server implementations with support for
/// HTTP/1.1, HTTP/2 and HTTP/3 protocols

pub mod types;
pub mod request;
pub mod response;
pub mod body;
pub mod client;
pub mod server;

// Re-export main types

pub use request::HttpRequest;
pub use response::HttpResponse;
pub use body::HttpResponseBody;
pub use client::HttpClient;
pub use server::HttpServer;
pub(crate) use body::PhpReaderAdapter;