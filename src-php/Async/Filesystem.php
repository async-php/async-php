<?php

namespace Async;

use Async\Driver\Filesystem as DriverFilesystem;
use Async\Driver\FileHandle as DriverFileHandle;
use Fiber;
use RustFuture;

class Filesystem
{
    // --- Stateless Operations ---

    public static function exists(string $path): bool
    {
        return Fiber::suspend(DriverFilesystem::exists($path));
    }

    public static function isFile(string $path): bool
    {
        return Fiber::suspend(DriverFilesystem::isFile($path));
    }

    public static function isDir(string $path): bool
    {
        return Fiber::suspend(DriverFilesystem::isDir($path));
    }

    public static function unlink(string $path): bool
    {
        return Fiber::suspend(DriverFilesystem::unlink($path));
    }

    public static function rename(string $from, string $to): bool
    {
        return Fiber::suspend(DriverFilesystem::rename($from, $to));
    }

    public static function copy(string $from, string $to): bool
    {
        return Fiber::suspend(DriverFilesystem::copy($from, $to));
    }

    public static function mkdir(string $path): bool
    {
        return Fiber::suspend(DriverFilesystem::mkdir($path));
    }

    public static function rmdir(string $path): bool
    {
        return Fiber::suspend(DriverFilesystem::rmdir($path));
    }

    public static function size(string $path): int|false
    {
        return Fiber::suspend(DriverFilesystem::size($path));
    }

    public static function getContents(string $path): string|false
    {
        return Fiber::suspend(DriverFilesystem::getContents($path));
    }

    public static function putContents(string $path, string $contents): bool
    {
        return Fiber::suspend(DriverFilesystem::putContents($path, $contents));
    }

    // --- Stateful Stream Operations (fopen replacement) ---

    public static function open(string $path, string $mode = 'r'): ?FileStream
    {
        $handle = Fiber::suspend(DriverFileHandle::open($path, $mode));
        if (!$handle) return null;
        return new FileStream($handle);
    }
}

class FileStream
{
    private DriverFileHandle $handle;

    public function __construct(DriverFileHandle $handle)
    {
        $this->handle = $handle;
    }

    public function read(int $length = 8192): string|false
    {
        return Fiber::suspend($this->handle->read($length));
    }

    public function write(string $data): int|false
    {
        return Fiber::suspend($this->handle->write($data));
    }

    public function seek(int $offset): int|false
    {
        return Fiber::suspend($this->handle->seek($offset));
    }

    public function close(): bool
    {
        return Fiber::suspend($this->handle->close());
    }
}
