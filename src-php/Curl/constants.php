<?php
/**
 * cURL constants for async-php
 *
 * This file defines all CURLOPT_*, CURLINFO_*, CURLE_*, CURLM_* and other cURL constants
 * to maintain compatibility with standard PHP curl extension.
 */

// ============================================================================
// CURLOPT_* Constants - Request Options
// ============================================================================

// URL and request configuration
const CURLOPT_URL = 10002;
const CURLOPT_PORT = 3;
const CURLOPT_TIMEOUT = 13;
const CURLOPT_CONNECTTIMEOUT = 78;
const CURLOPT_USERAGENT = 10018;
const CURLOPT_REFERER = 10016;

// HTTP method configuration
const CURLOPT_CUSTOMREQUEST = 10036;
const CURLOPT_HTTPGET = 80;
const CURLOPT_POST = 47;
const CURLOPT_PUT = 54;
const CURLOPT_NOBODY = 44;

// Request data
const CURLOPT_POSTFIELDS = 10015;
const CURLOPT_POSTFIELDSIZE = 60;
const CURLOPT_HTTPHEADER = 10023;

// Response handling
const CURLOPT_RETURNTRANSFER = 19913;
const CURLOPT_HEADER = 42;
const CURLOPT_HEADERFUNCTION = 20079;
const CURLOPT_WRITEFUNCTION = 20011;
const CURLOPT_READFUNCTION = 20012;

// Redirects
const CURLOPT_FOLLOWLOCATION = 52;
const CURLOPT_MAXREDIRS = 68;
const CURLOPT_AUTOREFERER = 58;

// SSL/TLS configuration
const CURLOPT_SSL_VERIFYPEER = 64;
const CURLOPT_SSL_VERIFYHOST = 81;
const CURLOPT_CAINFO = 10065;
const CURLOPT_CAPATH = 10097;
const CURLOPT_SSLCERT = 10025;
const CURLOPT_SSLKEY = 10087;
const CURLOPT_SSLCERTTYPE = 10086;
const CURLOPT_SSLKEYTYPE = 10088;
const CURLOPT_SSLVERSION = 32;

// Authentication
const CURLOPT_USERPWD = 10005;
const CURLOPT_HTTPAUTH = 107;
const CURLOPT_USERNAME = 10173;
const CURLOPT_PASSWORD = 10174;

// Proxy
const CURLOPT_PROXY = 10004;
const CURLOPT_PROXYPORT = 59;
const CURLOPT_PROXYTYPE = 101;
const CURLOPT_PROXYUSERPWD = 10006;
const CURLOPT_HTTPPROXYTUNNEL = 61;

// Cookies
const CURLOPT_COOKIE = 10022;
const CURLOPT_COOKIEFILE = 10031;
const CURLOPT_COOKIEJAR = 10082;

// ============================================================================
// CURLINFO_* Constants - Request Information
// ============================================================================

const CURLINFO_EFFECTIVE_URL = 1048577;
const CURLINFO_RESPONSE_CODE = 2097154;
const CURLINFO_HTTP_CODE = 2097154; // Alias for RESPONSE_CODE
const CURLINFO_TOTAL_TIME = 3145731;
const CURLINFO_NAMELOOKUP_TIME = 3145732;
const CURLINFO_CONNECT_TIME = 3145733;
const CURLINFO_PRETRANSFER_TIME = 3145734;
const CURLINFO_STARTTRANSFER_TIME = 3145735;
const CURLINFO_REDIRECT_TIME = 3145747;
const CURLINFO_REDIRECT_COUNT = 2097172;
const CURLINFO_SIZE_UPLOAD = 3145744;
const CURLINFO_SIZE_DOWNLOAD = 3145736;
const CURLINFO_SPEED_DOWNLOAD = 3145737;
const CURLINFO_SPEED_UPLOAD = 3145738;
const CURLINFO_HEADER_SIZE = 2097163;
const CURLINFO_REQUEST_SIZE = 2097164;
const CURLINFO_CONTENT_LENGTH_DOWNLOAD = 3145743;
const CURLINFO_CONTENT_LENGTH_UPLOAD = 3145744;
const CURLINFO_CONTENT_TYPE = 1048594;

// ============================================================================
// CURLE_* Constants - Error Codes
// ============================================================================

const CURLE_OK = 0;
const CURLE_UNSUPPORTED_PROTOCOL = 1;
const CURLE_FAILED_INIT = 2;
const CURLE_URL_MALFORMAT = 3;
const CURLE_COULDNT_RESOLVE_PROXY = 5;
const CURLE_COULDNT_RESOLVE_HOST = 6;
const CURLE_COULDNT_CONNECT = 7;
const CURLE_REMOTE_ACCESS_DENIED = 9;
const CURLE_HTTP_RETURNED_ERROR = 22;
const CURLE_WRITE_ERROR = 23;
const CURLE_READ_ERROR = 26;
const CURLE_OPERATION_TIMEDOUT = 28;
const CURLE_SSL_CONNECT_ERROR = 35;
const CURLE_TOO_MANY_REDIRECTS = 47;
const CURLE_PEER_FAILED_VERIFICATION = 51;
const CURLE_GOT_NOTHING = 52;
const CURLE_SSL_ENGINE_NOTFOUND = 53;
const CURLE_SSL_ENGINE_SETFAILED = 54;
const CURLE_SEND_ERROR = 55;
const CURLE_RECV_ERROR = 56;
const CURLE_SSL_CERTPROBLEM = 58;
const CURLE_SSL_CIPHER = 59;
const CURLE_SSL_CACERT = 60;

// ============================================================================
// CURLM_* Constants - Multi Handle
// ============================================================================

const CURLM_OK = 0;
const CURLM_BAD_HANDLE = 1;
const CURLM_BAD_EASY_HANDLE = 2;
const CURLM_OUT_OF_MEMORY = 3;
const CURLM_INTERNAL_ERROR = 4;
const CURLM_CALL_MULTI_PERFORM = -1;

// ============================================================================
// CURLAUTH_* Constants - Authentication Methods
// ============================================================================

const CURLAUTH_BASIC = 1;
const CURLAUTH_DIGEST = 2;
const CURLAUTH_GSSNEGOTIATE = 4;
const CURLAUTH_NTLM = 8;
const CURLAUTH_ANY = -17;
const CURLAUTH_ANYSAFE = -18;

// ============================================================================
// CURLPROXY_* Constants - Proxy Types
// ============================================================================

const CURLPROXY_HTTP = 0;
const CURLPROXY_SOCKS4 = 4;
const CURLPROXY_SOCKS5 = 5;

// ============================================================================
// HTTP Version Constants
// ============================================================================

const CURL_HTTP_VERSION_NONE = 0;
const CURL_HTTP_VERSION_1_0 = 1;
const CURL_HTTP_VERSION_1_1 = 2;
const CURL_HTTP_VERSION_2_0 = 3;

// ============================================================================
// curl_version() Feature Flags
// ============================================================================

const CURL_VERSION_IPV6 = 1;
const CURL_VERSION_KERBEROS4 = 2;
const CURL_VERSION_SSL = 4;
const CURL_VERSION_LIBZ = 8;
