/// Minimal HTTP module
///
/// This module provides a thin Rust wrapper around hyper-util.
/// All business logic (redirects, retries, auth, cookies) should be
/// implemented in PHP for maximum flexibility.

mod body;
mod response;
mod client;

pub use body::HttpResponseBody;
pub use response::HttpResponse;
pub use client::HttpClient;
