/// HTTP module for async-php runtime
/// Provides HTTP client and server implementations with support for
/// HTTP/1.1, HTTP/2 and HTTP/3 protocols

pub mod types;
pub mod request;
pub mod response;
pub mod body;
pub mod message;
pub mod transport;

// Re-export main types

pub use body::HttpResponseBody;
// AsyncReadBody is public for potential future use but not currently exported
#[allow(unused_imports)]
pub(crate) use body::AsyncReadBody;
pub use transport::HttpTransport;
pub use request::HttpRequest;
pub use response::HttpResponse;
