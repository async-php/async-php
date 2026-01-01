<?php

namespace Async;

use Async\Time;
use Async\Stream\TcpStreamWrapper;
use Async\Stream\UdpStreamWrapper;
use Async\Stream\UnixStreamWrapper;
use Async\Stream\TlsStreamWrapper;
use Async\Stream\FileStreamWrapper;
use Async\Stream\HttpStreamWrapper;

class Kernel
{
    const HOOK_TCP = 1;
    const HOOK_UDP = 2; 
    const HOOK_UNIX = 4; 
    const HOOK_SSL = 8; 
    const HOOK_FILE = 16;
    const HOOK_HTTP = 32;
    const HOOK_ALL = 0x7FFFFFFF;

    private static int $hookedFlags = 0;

    public static function run(callable $main): void
    {
        if (!class_exists('Async\Kernel\Runtime')) {
            throw new \RuntimeException("Async extension not loaded.");
        }

        $fiber = new \Fiber($main);
        \Async\Kernel\Runtime::block_on($fiber);
    }

    public static function spawn(callable $task): int
    {
        $fiber = new \Fiber($task);
        return \Async\Kernel\Runtime::spawn($fiber);
    }

    public static function setupLog(array $config): void
    {
        \Async\Kernel\Logger::init($config);
    }

    public static function enableCoroutine(int $flags = self::HOOK_ALL): void
    {
        // Ensure wrappers are loaded before registering
        
        if (($flags & self::HOOK_TCP) && !self::isHooked(self::HOOK_TCP)) {
            self::hook('tcp', TcpStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_TCP;
        }

        if (($flags & self::HOOK_UDP) && !self::isHooked(self::HOOK_UDP)) {
            self::hook('udp', UdpStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_UDP;
        }

        if (($flags & self::HOOK_UNIX) && !self::isHooked(self::HOOK_UNIX)) {
            self::hook('unix', UnixStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_UNIX;
        }

        if (($flags & self::HOOK_SSL) && !self::isHooked(self::HOOK_SSL)) {
            self::hook('ssl', TlsStreamWrapper::class);
            self::hook('tls', TlsStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_SSL;
        }

        if (($flags & self::HOOK_FILE) && !self::isHooked(self::HOOK_FILE)) {
            self::hook('file', FileStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_FILE;
        }

        if (($flags & self::HOOK_HTTP) && !self::isHooked(self::HOOK_HTTP)) {
            self::hook('http', HttpStreamWrapper::class);
            self::hook('https', HttpStreamWrapper::class);
            self::$hookedFlags |= self::HOOK_HTTP;
        }
    }

    private static function isHooked(int $flag): bool
    {
        return (self::$hookedFlags & $flag) === $flag;
    }

    private static function hook(string $protocol, string $class): void
    {
        if (!class_exists($class, true)) {
            throw new \RuntimeException("Failed to load stream wrapper class: $class");
        }
        // Ignore warnings if wrapper doesn't exist or can't be unregistered (will fail on register then)
        @stream_wrapper_unregister($protocol);
        stream_wrapper_register($protocol, $class);
    }
}