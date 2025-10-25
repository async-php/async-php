<?php

namespace Async;

use Async\Stream\FileStreamWrapper;
use Async\Stream\TcpStreamWrapper;

class Kernel
{
    const HOOK_TCP = 1;
    const HOOK_UDP = 2; // TODO
    const HOOK_UNIX = 4; // TODO
    const HOOK_SSL = 8; // TODO
    const HOOK_FILE = 16;
    const HOOK_ALL = 0x7FFFFFFF;

    private static int $hookedFlags = 0;

    public static function run(callable $main): void
    {
        if (!function_exists('run')) {
            throw new \RuntimeException("Async extension not loaded.");
        }

        $fiber = new \Fiber($main);
        \run($fiber);
    }

    public static function sleep(int $ms): void
    {
        \Fiber::suspend(Time::sleep($ms));
    }

    public static function spawn(callable $task): void
    {
        \go($task);
    }

    public static function setupLog(array $config): void
    {
        \Async\Logger::init($config);
    }

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
