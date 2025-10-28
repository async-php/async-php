/// Common HTTP types and utilities

use ext_php_rs::prelude::*;

/// HTTP status codes as constants
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\StatusCodes")]
pub struct StatusCodes;

#[php_impl]
impl StatusCodes {
    pub const CONTINUE: i32 = 100;
    pub const SWITCHING_PROTOCOLS: i32 = 101;

    pub const OK: i32 = 200;
    pub const CREATED: i32 = 201;
    pub const ACCEPTED: i32 = 202;
    pub const NO_CONTENT: i32 = 204;

    pub const MULTIPLE_CHOICES: i32 = 300;
    pub const MOVED_PERMANENTLY: i32 = 301;
    pub const FOUND: i32 = 302;
    pub const SEE_OTHER: i32 = 303;
    pub const NOT_MODIFIED: i32 = 304;
    pub const TEMPORARY_REDIRECT: i32 = 307;
    pub const PERMANENT_REDIRECT: i32 = 308;

    pub const BAD_REQUEST: i32 = 400;
    pub const UNAUTHORIZED: i32 = 401;
    pub const FORBIDDEN: i32 = 403;
    pub const NOT_FOUND: i32 = 404;
    pub const METHOD_NOT_ALLOWED: i32 = 405;
    pub const NOT_ACCEPTABLE: i32 = 406;
    pub const REQUEST_TIMEOUT: i32 = 408;
    pub const CONFLICT: i32 = 409;
    pub const GONE: i32 = 410;
    pub const LENGTH_REQUIRED: i32 = 411;
    pub const PAYLOAD_TOO_LARGE: i32 = 413;
    pub const URI_TOO_LONG: i32 = 414;
    pub const UNSUPPORTED_MEDIA_TYPE: i32 = 415;
    pub const RANGE_NOT_SATISFIABLE: i32 = 416;
    pub const UNPROCESSABLE_ENTITY: i32 = 422;
    pub const TOO_MANY_REQUESTS: i32 = 429;

    pub const INTERNAL_SERVER_ERROR: i32 = 500;
    pub const NOT_IMPLEMENTED: i32 = 501;
    pub const BAD_GATEWAY: i32 = 502;
    pub const SERVICE_UNAVAILABLE: i32 = 503;
    pub const GATEWAY_TIMEOUT: i32 = 504;

    pub fn is_success(status: i32) -> bool {
        (200..300).contains(&status)
    }

    pub fn is_redirect(status: i32) -> bool {
        (300..400).contains(&status)
    }

    pub fn is_client_error(status: i32) -> bool {
        (400..500).contains(&status)
    }

    pub fn is_server_error(status: i32) -> bool {
        (500..600).contains(&status)
    }

    pub fn is_error(status: i32) -> bool {
        status >= 400
    }
}