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
if (!defined('CURLOPT_URL')) define('CURLOPT_URL', 10002);
if (!defined('CURLOPT_PORT')) define('CURLOPT_PORT', 3);
if (!defined('CURLOPT_TIMEOUT')) define('CURLOPT_TIMEOUT', 13);
if (!defined('CURLOPT_CONNECTTIMEOUT')) define('CURLOPT_CONNECTTIMEOUT', 78);
if (!defined('CURLOPT_USERAGENT')) define('CURLOPT_USERAGENT', 10018);
if (!defined('CURLOPT_REFERER')) define('CURLOPT_REFERER', 10016);

// HTTP method configuration
if (!defined('CURLOPT_CUSTOMREQUEST')) define('CURLOPT_CUSTOMREQUEST', 10036);
if (!defined('CURLOPT_HTTPGET')) define('CURLOPT_HTTPGET', 80);
if (!defined('CURLOPT_POST')) define('CURLOPT_POST', 47);
if (!defined('CURLOPT_PUT')) define('CURLOPT_PUT', 54);
if (!defined('CURLOPT_NOBODY')) define('CURLOPT_NOBODY', 44);

// Request data
if (!defined('CURLOPT_POSTFIELDS')) define('CURLOPT_POSTFIELDS', 10015);
if (!defined('CURLOPT_POSTFIELDSIZE')) define('CURLOPT_POSTFIELDSIZE', 60);
if (!defined('CURLOPT_HTTPHEADER')) define('CURLOPT_HTTPHEADER', 10023);

// Response handling
if (!defined('CURLOPT_RETURNTRANSFER')) define('CURLOPT_RETURNTRANSFER', 19913);
if (!defined('CURLOPT_HEADER')) define('CURLOPT_HEADER', 42);
if (!defined('CURLOPT_HEADERFUNCTION')) define('CURLOPT_HEADERFUNCTION', 20079);
if (!defined('CURLOPT_WRITEFUNCTION')) define('CURLOPT_WRITEFUNCTION', 20011);
if (!defined('CURLOPT_READFUNCTION')) define('CURLOPT_READFUNCTION', 20012);

// Redirects
if (!defined('CURLOPT_FOLLOWLOCATION')) define('CURLOPT_FOLLOWLOCATION', 52);
if (!defined('CURLOPT_MAXREDIRS')) define('CURLOPT_MAXREDIRS', 68);
if (!defined('CURLOPT_AUTOREFERER')) define('CURLOPT_AUTOREFERER', 58);

// SSL/TLS configuration
if (!defined('CURLOPT_SSL_VERIFYPEER')) define('CURLOPT_SSL_VERIFYPEER', 64);
if (!defined('CURLOPT_SSL_VERIFYHOST')) define('CURLOPT_SSL_VERIFYHOST', 81);
if (!defined('CURLOPT_CAINFO')) define('CURLOPT_CAINFO', 10065);
if (!defined('CURLOPT_CAPATH')) define('CURLOPT_CAPATH', 10097);
if (!defined('CURLOPT_SSLCERT')) define('CURLOPT_SSLCERT', 10025);
if (!defined('CURLOPT_SSLKEY')) define('CURLOPT_SSLKEY', 10087);
if (!defined('CURLOPT_SSLCERTTYPE')) define('CURLOPT_SSLCERTTYPE', 10086);
if (!defined('CURLOPT_SSLKEYTYPE')) define('CURLOPT_SSLKEYTYPE', 10088);
if (!defined('CURLOPT_SSLVERSION')) define('CURLOPT_SSLVERSION', 32);

// Authentication
if (!defined('CURLOPT_USERPWD')) define('CURLOPT_USERPWD', 10005);
if (!defined('CURLOPT_HTTPAUTH')) define('CURLOPT_HTTPAUTH', 107);
if (!defined('CURLOPT_USERNAME')) define('CURLOPT_USERNAME', 10173);
if (!defined('CURLOPT_PASSWORD')) define('CURLOPT_PASSWORD', 10174);

// Proxy
if (!defined('CURLOPT_PROXY')) define('CURLOPT_PROXY', 10004);
if (!defined('CURLOPT_PROXYPORT')) define('CURLOPT_PROXYPORT', 59);
if (!defined('CURLOPT_PROXYTYPE')) define('CURLOPT_PROXYTYPE', 101);
if (!defined('CURLOPT_PROXYUSERPWD')) define('CURLOPT_PROXYUSERPWD', 10006);
if (!defined('CURLOPT_HTTPPROXYTUNNEL')) define('CURLOPT_HTTPPROXYTUNNEL', 61);

// Cookies
if (!defined('CURLOPT_COOKIE')) define('CURLOPT_COOKIE', 10022);
if (!defined('CURLOPT_COOKIEFILE')) define('CURLOPT_COOKIEFILE', 10031);
if (!defined('CURLOPT_COOKIEJAR')) define('CURLOPT_COOKIEJAR', 10082);

// ============================================================================
// CURLINFO_* Constants - Request Information
// ============================================================================

if (!defined('CURLINFO_EFFECTIVE_URL')) define('CURLINFO_EFFECTIVE_URL', 1048577);
if (!defined('CURLINFO_RESPONSE_CODE')) define('CURLINFO_RESPONSE_CODE', 2097154);
if (!defined('CURLINFO_HTTP_CODE')) define('CURLINFO_HTTP_CODE', 2097154); // Alias for RESPONSE_CODE
if (!defined('CURLINFO_TOTAL_TIME')) define('CURLINFO_TOTAL_TIME', 3145731);
if (!defined('CURLINFO_NAMELOOKUP_TIME')) define('CURLINFO_NAMELOOKUP_TIME', 3145732);
if (!defined('CURLINFO_CONNECT_TIME')) define('CURLINFO_CONNECT_TIME', 3145733);
if (!defined('CURLINFO_PRETRANSFER_TIME')) define('CURLINFO_PRETRANSFER_TIME', 3145734);
if (!defined('CURLINFO_STARTTRANSFER_TIME')) define('CURLINFO_STARTTRANSFER_TIME', 3145735);
if (!defined('CURLINFO_REDIRECT_TIME')) define('CURLINFO_REDIRECT_TIME', 3145747);
if (!defined('CURLINFO_REDIRECT_COUNT')) define('CURLINFO_REDIRECT_COUNT', 2097172);
if (!defined('CURLINFO_SIZE_UPLOAD')) define('CURLINFO_SIZE_UPLOAD', 3145744);
if (!defined('CURLINFO_SIZE_DOWNLOAD')) define('CURLINFO_SIZE_DOWNLOAD', 3145736);
if (!defined('CURLINFO_SPEED_DOWNLOAD')) define('CURLINFO_SPEED_DOWNLOAD', 3145737);
if (!defined('CURLINFO_SPEED_UPLOAD')) define('CURLINFO_SPEED_UPLOAD', 3145738);
if (!defined('CURLINFO_HEADER_SIZE')) define('CURLINFO_HEADER_SIZE', 2097163);
if (!defined('CURLINFO_REQUEST_SIZE')) define('CURLINFO_REQUEST_SIZE', 2097164);
if (!defined('CURLINFO_CONTENT_LENGTH_DOWNLOAD')) define('CURLINFO_CONTENT_LENGTH_DOWNLOAD', 3145743);
if (!defined('CURLINFO_CONTENT_LENGTH_UPLOAD')) define('CURLINFO_CONTENT_LENGTH_UPLOAD', 3145744);
if (!defined('CURLINFO_CONTENT_TYPE')) define('CURLINFO_CONTENT_TYPE', 1048594);

// ============================================================================
// CURLE_* Constants - Error Codes
// ============================================================================

if (!defined('CURLE_OK')) define('CURLE_OK', 0);
if (!defined('CURLE_UNSUPPORTED_PROTOCOL')) define('CURLE_UNSUPPORTED_PROTOCOL', 1);
if (!defined('CURLE_FAILED_INIT')) define('CURLE_FAILED_INIT', 2);
if (!defined('CURLE_URL_MALFORMAT')) define('CURLE_URL_MALFORMAT', 3);
if (!defined('CURLE_COULDNT_RESOLVE_PROXY')) define('CURLE_COULDNT_RESOLVE_PROXY', 5);
if (!defined('CURLE_COULDNT_RESOLVE_HOST')) define('CURLE_COULDNT_RESOLVE_HOST', 6);
if (!defined('CURLE_COULDNT_CONNECT')) define('CURLE_COULDNT_CONNECT', 7);
if (!defined('CURLE_REMOTE_ACCESS_DENIED')) define('CURLE_REMOTE_ACCESS_DENIED', 9);
if (!defined('CURLE_HTTP_RETURNED_ERROR')) define('CURLE_HTTP_RETURNED_ERROR', 22);
if (!defined('CURLE_WRITE_ERROR')) define('CURLE_WRITE_ERROR', 23);
if (!defined('CURLE_READ_ERROR')) define('CURLE_READ_ERROR', 26);
if (!defined('CURLE_OPERATION_TIMEDOUT')) define('CURLE_OPERATION_TIMEDOUT', 28);
if (!defined('CURLE_SSL_CONNECT_ERROR')) define('CURLE_SSL_CONNECT_ERROR', 35);
if (!defined('CURLE_TOO_MANY_REDIRECTS')) define('CURLE_TOO_MANY_REDIRECTS', 47);
if (!defined('CURLE_PEER_FAILED_VERIFICATION')) define('CURLE_PEER_FAILED_VERIFICATION', 51);
if (!defined('CURLE_GOT_NOTHING')) define('CURLE_GOT_NOTHING', 52);
if (!defined('CURLE_SSL_ENGINE_NOTFOUND')) define('CURLE_SSL_ENGINE_NOTFOUND', 53);
if (!defined('CURLE_SSL_ENGINE_SETFAILED')) define('CURLE_SSL_ENGINE_SETFAILED', 54);
if (!defined('CURLE_SEND_ERROR')) define('CURLE_SEND_ERROR', 55);
if (!defined('CURLE_RECV_ERROR')) define('CURLE_RECV_ERROR', 56);
if (!defined('CURLE_SSL_CERTPROBLEM')) define('CURLE_SSL_CERTPROBLEM', 58);
if (!defined('CURLE_SSL_CIPHER')) define('CURLE_SSL_CIPHER', 59);
if (!defined('CURLE_SSL_CACERT')) define('CURLE_SSL_CACERT', 60);

// ============================================================================
// CURLM_* Constants - Multi Handle
// ============================================================================

if (!defined('CURLM_OK')) define('CURLM_OK', 0);
if (!defined('CURLM_BAD_HANDLE')) define('CURLM_BAD_HANDLE', 1);
if (!defined('CURLM_BAD_EASY_HANDLE')) define('CURLM_BAD_EASY_HANDLE', 2);
if (!defined('CURLM_OUT_OF_MEMORY')) define('CURLM_OUT_OF_MEMORY', 3);
if (!defined('CURLM_INTERNAL_ERROR')) define('CURLM_INTERNAL_ERROR', 4);
if (!defined('CURLM_CALL_MULTI_PERFORM')) define('CURLM_CALL_MULTI_PERFORM', -1);

// ============================================================================
// CURLAUTH_* Constants - Authentication Methods
// ============================================================================

if (!defined('CURLAUTH_BASIC')) define('CURLAUTH_BASIC', 1);
if (!defined('CURLAUTH_DIGEST')) define('CURLAUTH_DIGEST', 2);
if (!defined('CURLAUTH_GSSNEGOTIATE')) define('CURLAUTH_GSSNEGOTIATE', 4);
if (!defined('CURLAUTH_NTLM')) define('CURLAUTH_NTLM', 8);
if (!defined('CURLAUTH_ANY')) define('CURLAUTH_ANY', -17);
if (!defined('CURLAUTH_ANYSAFE')) define('CURLAUTH_ANYSAFE', -18);

// ============================================================================
// CURLPROXY_* Constants - Proxy Types
// ============================================================================

if (!defined('CURLPROXY_HTTP')) define('CURLPROXY_HTTP', 0);
if (!defined('CURLPROXY_SOCKS4')) define('CURLPROXY_SOCKS4', 4);
if (!defined('CURLPROXY_SOCKS5')) define('CURLPROXY_SOCKS5', 5);

// ============================================================================
// HTTP Version Constants
// ============================================================================

if (!defined('CURL_HTTP_VERSION_NONE')) define('CURL_HTTP_VERSION_NONE', 0);
if (!defined('CURL_HTTP_VERSION_1_0')) define('CURL_HTTP_VERSION_1_0', 1);
if (!defined('CURL_HTTP_VERSION_1_1')) define('CURL_HTTP_VERSION_1_1', 2);
if (!defined('CURL_HTTP_VERSION_2_0')) define('CURL_HTTP_VERSION_2_0', 3);

// ============================================================================
// curl_version() Feature Flags
// ============================================================================

if (!defined('CURL_VERSION_IPV6')) define('CURL_VERSION_IPV6', 1);
if (!defined('CURL_VERSION_KERBEROS4')) define('CURL_VERSION_KERBEROS4', 2);
if (!defined('CURL_VERSION_SSL')) define('CURL_VERSION_SSL', 4);
if (!defined('CURL_VERSION_LIBZ')) define('CURL_VERSION_LIBZ', 8);
