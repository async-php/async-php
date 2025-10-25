<?php

namespace Async;

use Async\Stream\TcpStreamWrapper;
use Async\Stream\FileStreamWrapper;

class Runtime
{
    const HOOK_TCP = 1;
    const HOOK_UDP = 2; // TODO
    const HOOK_UNIX = 4; // TODO
    const HOOK_SSL = 8; // TODO
    const HOOK_FILE = 16;
    const HOOK_ALL = 0x7FFFFFFF;

    private static int $hookedFlags = 0;

    public static function enableCoroutine(int $flags = self::HOOK_ALL): void
    {
        // Ensure wrappers are loaded before registering
        if (!class_exists(TcpStreamWrapper::class)) {
             require_once __DIR__ . '/Stream/TcpStreamWrapper.php';
        }
        if (!class_exists(FileStreamWrapper::class)) {
             require_once __DIR__ . '/Stream/FileStreamWrapper.php';
        }

        if (($flags & self::HOOK_TCP) && !self::isHooked(self::HOOK_TCP)) {
            self::hook('tcp', TcpStreamWrapper::class);
            
            // Also hook stream_socket_client if TCP is hooked
            // Using function table overwrite via Rust extension
            // Replacement function must be a global function. 
            // We will define a stub in this file.
            
            \override_function('stream_socket_client', 'Async\\stream_socket_client_stub');
            
            self::$hookedFlags |= self::HOOK_TCP;
        }
        
        if (($flags & self::HOOK_FILE) && !self::isHooked(self::HOOK_FILE)) {
            self::hook('file', FileStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_FILE;
        }
    }
    
    private static function isHooked(int $flag): bool
    {
        return (self::$hookedFlags & $flag) === $flag;
    }
    
    private static function hook(string $protocol, string $class): void
    {
        // Ignore warnings if wrapper doesn't exist or can't be unregistered (will fail on register then)
        @stream_wrapper_unregister($protocol);
        stream_wrapper_register($protocol, $class);
    }
}

// Stub function in global namespace (but inside namespace Async in this file? No, needs to be global or FQDN)
// Let's define it in Async namespace and pass FQDN to override_function.

function stream_socket_client_stub(
    string $address, 
    &$errno = null, 
    &$errstr = null, 
    ?float $timeout = null, 
    int $flags = STREAM_CLIENT_CONNECT, 
    $context = null
) {
    // Simply delegate to fopen, which is already hooked by TcpStreamWrapper!
    // TcpStreamWrapper supports tcp:// 
    
    // Note: context handling is basic here.
    $mode = ($flags & STREAM_CLIENT_CONNECT) ? 'r+' : 'r'; // Simplified
    
    // fopen returns false on failure, stream_socket_client implies false too.
    // But TcpStreamWrapper::stream_open returns bool.
    
    // We need to handle timeout logic if provided (passed to wrapper options?).
    // For now, just call fopen.
    
    if ($context) {
        return fopen($address, $mode, false, $context);
    } else {
        return fopen($address, $mode);
    }
}
