/// HTTP Module - Full-featured HTTP client based on reqwest
///
/// This module provides a complete HTTP client implementation with:
/// - Automatic connection pooling
/// - Redirect handling
/// - Cookie management
/// - TLS/SSL configuration
/// - Streaming request/response bodies
/// - JSON support
/// - Form data
/// - Multipart uploads

mod client;
mod request;
mod response;
mod server;

pub use client::HttpClient;
pub use request::HttpRequest;
pub use response::HttpResponse;
pub use server::HttpServer;
