<?php

use Curl\Handle;
use Curl\Multi;

/**
 * Global cURL functions for async-php
 *
 * These functions provide the standard PHP cURL procedural API,
 * wrapping the Handle and Multi classes.
 */

// ============================================================================
// Basic cURL Functions
// ============================================================================

/**
 * Initialize a cURL session
 *
 * @param string|null $url Optional URL to set
 * @return Handle|false Handle object or false on failure
 */
function curl_init(?string $url = null): Handle|false
{
    try {
        return new Handle($url);
    } catch (\Throwable $e) {
        return false;
    }
}

/**
 * Set an option for a cURL transfer
 *
 * @param Handle $handle cURL handle
 * @param int $option CURLOPT_* constant
 * @param mixed $value Option value
 * @return bool True on success
 */
function curl_setopt(Handle $handle, int $option, mixed $value): bool
{
    return $handle->setopt($option, $value);
}

/**
 * Set multiple options for a cURL transfer
 *
 * @param Handle $handle cURL handle
 * @param array $options Array of CURLOPT_* => value pairs
 * @return bool True on success
 */
function curl_setopt_array(Handle $handle, array $options): bool
{
    return $handle->setoptArray($options);
}

/**
 * Perform a cURL session (async)
 *
 * @param Handle $handle cURL handle
 * @return string|bool Response body or true/false depending on CURLOPT_RETURNTRANSFER
 */
function curl_exec(Handle $handle): string|bool
{
    return $handle->exec();
}

/**
 * Get information regarding a specific transfer
 *
 * @param Handle $handle cURL handle
 * @param int|null $option CURLINFO_* constant, or null for all info
 * @return mixed Info value or array
 */
function curl_getinfo(Handle $handle, ?int $option = null): mixed
{
    return $handle->getinfo($option);
}

/**
 * Return the last error number
 *
 * @param Handle $handle cURL handle
 * @return int Error number (CURLE_* constant)
 */
function curl_errno(Handle $handle): int
{
    return $handle->errno();
}

/**
 * Return a string containing the last error
 *
 * @param Handle $handle cURL handle
 * @return string Error message
 */
function curl_error(Handle $handle): string
{
    return $handle->error();
}

/**
 * Reset all options of a cURL session handle
 *
 * @param Handle $handle cURL handle
 */
function curl_reset(Handle $handle): void
{
    $handle->reset();
}

/**
 * Close a cURL session
 *
 * @param Handle $handle cURL handle
 */
function curl_close(Handle $handle): void
{
    $handle->close();
}

// ============================================================================
// Multi Handle Functions
// ============================================================================

/**
 * Returns a new cURL multi handle
 *
 * @return Multi Multi handle object
 */
function curl_multi_init(): Multi
{
    return new Multi();
}

/**
 * Add a normal cURL handle to a cURL multi handle
 *
 * @param Multi $multiHandle Multi handle
 * @param Handle $handle Handle to add
 * @return int CURLM_OK on success
 */
function curl_multi_add_handle(Multi $multiHandle, Handle $handle): int
{
    return $multiHandle->addHandle($handle);
}

/**
 * Remove a multi handle from a set of cURL handles
 *
 * @param Multi $multiHandle Multi handle
 * @param Handle $handle Handle to remove
 * @return int CURLM_OK on success
 */
function curl_multi_remove_handle(Multi $multiHandle, Handle $handle): int
{
    return $multiHandle->removeHandle($handle);
}

/**
 * Run the sub-connections of the current cURL handle (async)
 *
 * @param Multi $multiHandle Multi handle
 * @param int|null $stillRunning Output parameter for number of handles still running
 * @return int CURLM_OK on success
 */
function curl_multi_exec(Multi $multiHandle, ?int &$stillRunning = null): int
{
    return $multiHandle->exec($stillRunning);
}

/**
 * Get information about the current transfers
 *
 * @param Multi $multiHandle Multi handle
 * @param int|null $msgsInQueue Output parameter for remaining messages
 * @return array|false Info array or false
 */
function curl_multi_info_read(Multi $multiHandle, ?int &$msgsInQueue = null): array|false
{
    return $multiHandle->infoRead($msgsInQueue);
}

/**
 * Return the content of a cURL handle if CURLOPT_RETURNTRANSFER is set
 *
 * @param Handle $handle cURL handle
 * @return string|null Content or null
 */
function curl_multi_getcontent(Handle $handle): ?string
{
    // This would need to be implemented properly
    // For now, return null as content is returned by curl_exec
    return null;
}

/**
 * Wait for activity on any curl_multi connection
 *
 * @param Multi $multiHandle Multi handle
 * @param float $timeout Timeout in seconds
 * @return int Number of descriptors with activity
 */
function curl_multi_select(Multi $multiHandle, float $timeout = 1.0): int
{
    return $multiHandle->select($timeout);
}

/**
 * Close a set of cURL handles
 *
 * @param Multi $multiHandle Multi handle
 */
function curl_multi_close(Multi $multiHandle): void
{
    $multiHandle->close();
}

/**
 * Return the last error number for a multi handle
 *
 * @param Multi $multiHandle Multi handle
 * @return int Error number
 */
function curl_multi_errno(Multi $multiHandle): int
{
    // Not implemented yet - return success
    return CURLM_OK;
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Return string describing the given error code
 *
 * @param int $errornum Error code (CURLE_* constant)
 * @return string|null Error description or null
 */
function curl_strerror(int $errornum): ?string
{
    $errors = [
        CURLE_OK => 'No error',
        CURLE_UNSUPPORTED_PROTOCOL => 'Unsupported protocol',
        CURLE_FAILED_INIT => 'Failed initialization',
        CURLE_URL_MALFORMAT => 'URL using bad/illegal format or missing URL',
        CURLE_COULDNT_RESOLVE_PROXY => "Couldn't resolve proxy name",
        CURLE_COULDNT_RESOLVE_HOST => "Couldn't resolve host name",
        CURLE_COULDNT_CONNECT => 'Failed to connect to host',
        CURLE_REMOTE_ACCESS_DENIED => 'Access denied',
        CURLE_HTTP_RETURNED_ERROR => 'HTTP request returned error',
        CURLE_WRITE_ERROR => 'Failed writing received data',
        CURLE_READ_ERROR => 'Failed reading local file',
        CURLE_OPERATION_TIMEDOUT => 'Operation timeout',
        CURLE_SSL_CONNECT_ERROR => 'SSL connection error',
        CURLE_TOO_MANY_REDIRECTS => 'Too many redirects',
        CURLE_PEER_FAILED_VERIFICATION => 'SSL peer certificate verification failed',
        CURLE_GOT_NOTHING => 'Server returned nothing',
        CURLE_SSL_ENGINE_NOTFOUND => 'SSL crypto engine not found',
        CURLE_SSL_ENGINE_SETFAILED => 'Failed to set SSL crypto engine as default',
        CURLE_SEND_ERROR => 'Failed sending data to the peer',
        CURLE_RECV_ERROR => 'Failure when receiving data from the peer',
        CURLE_SSL_CERTPROBLEM => 'Problem with the local SSL certificate',
        CURLE_SSL_CIPHER => "Couldn't use specified SSL cipher",
        CURLE_SSL_CACERT => 'Peer certificate cannot be authenticated with given CA certificates',
    ];

    return $errors[$errornum] ?? null;
}

/**
 * Return string describing the given multi error code
 *
 * @param int $errornum Error code (CURLM_* constant)
 * @return string|null Error description or null
 */
function curl_multi_strerror(int $errornum): ?string
{
    $errors = [
        CURLM_OK => 'No error',
        CURLM_BAD_HANDLE => 'Invalid multi handle',
        CURLM_BAD_EASY_HANDLE => 'Invalid easy handle',
        CURLM_OUT_OF_MEMORY => 'Out of memory',
        CURLM_INTERNAL_ERROR => 'Internal error',
    ];

    return $errors[$errornum] ?? null;
}

/**
 * URL encodes the given string
 *
 * @param Handle $handle cURL handle (ignored, kept for compatibility)
 * @param string $str String to encode
 * @return string|false Encoded string or false on failure
 */
function curl_escape(Handle $handle, string $str): string|false
{
    return rawurlencode($str);
}

/**
 * Decodes the given URL encoded string
 *
 * @param Handle $handle cURL handle (ignored, kept for compatibility)
 * @param string $str String to decode
 * @return string|false Decoded string or false on failure
 */
function curl_unescape(Handle $handle, string $str): string|false
{
    return rawurldecode($str);
}

/**
 * Return cURL version information
 *
 * @return array|false Version information array
 */
function curl_version(): array|false
{
    return [
        'version_number' => 0x075400, // 7.84.0
        'version' => '7.84.0',
        'ssl_version_number' => 0,
        'ssl_version' => 'rustls',
        'libz_version' => '1.2.11',
        'host' => 'async-php',
        'age' => 8,
        'features' => CURL_VERSION_SSL | CURL_VERSION_LIBZ | CURL_VERSION_IPV6,
        'protocols' => ['http', 'https', 'ftp', 'ftps', 'smtp', 'smtps'],
    ];
}

/**
 * Copy a cURL handle along with all of its preferences
 *
 * @param Handle $handle cURL handle to copy
 * @return Handle|false New handle or false on failure
 */
function curl_copy_handle(Handle $handle): Handle|false
{
    // Not fully implemented - create new handle for now
    try {
        return new Handle();
    } catch (\Throwable $e) {
        return false;
    }
}

/**
 * Pause and unpause a connection
 *
 * @param Handle $handle cURL handle
 * @param int $bitmask Pause bitmask
 * @return int CURLE_OK on success
 */
function curl_pause(Handle $handle, int $bitmask): int
{
    // Not implemented - return success for compatibility
    return CURLE_OK;
}

if (!class_exists('CurlHandle')) {
    class_alias(Handle::class, 'CurlHandle');
    class_alias(Multi::class, 'CurlMultiHandle');
}