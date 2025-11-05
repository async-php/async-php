/// HTTP module for async-php runtime
/// Provides HTTP client and server implementations with support for
/// HTTP/1.1, HTTP/2 and HTTP/3 protocols

pub mod types;
pub mod request;
pub mod response;
pub mod body;
pub mod client;
pub mod server;
pub mod auth;
pub mod cookies;
pub mod retry;
pub mod metrics;

// Re-export main types

pub use body::HttpResponseBody;
pub(crate) use body::PhpReaderAdapter;
pub use client::HttpClient;
pub use request::HttpRequest;
pub use response::HttpResponse;
pub use server::HttpServer;
