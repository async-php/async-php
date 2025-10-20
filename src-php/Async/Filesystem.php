<?php

namespace Async;

use AsyncFilesystem;
use AsyncFileHandle;
use Fiber;
use RustFuture;

class Filesystem
{
    // --- Stateless Operations ---

    public static function exists(string $path): bool
    {
        return Fiber::suspend(AsyncFilesystem::exists($path));
    }

    public static function isFile(string $path): bool
    {
        return Fiber::suspend(AsyncFilesystem::isFile($path));
    }

    public static function isDir(string $path): bool
    {
        return Fiber::suspend(AsyncFilesystem::isDir($path));
    }

    public static function unlink(string $path): bool
    {
        return Fiber::suspend(AsyncFilesystem::unlink($path));
    }

    public static function rename(string $from, string $to): bool
    {
        return Fiber::suspend(AsyncFilesystem::rename($from, $to));
    }

    public static function copy(string $from, string $to): bool
    {
        return Fiber::suspend(AsyncFilesystem::copy($from, $to));
    }

    public static function mkdir(string $path): bool
    {
        return Fiber::suspend(AsyncFilesystem::mkdir($path));
    }

    public static function rmdir(string $path): bool
    {
        return Fiber::suspend(AsyncFilesystem::rmdir($path));
    }

    public static function size(string $path): int|false
    {
        return Fiber::suspend(AsyncFilesystem::size($path));
    }

    public static function getContents(string $path): string|false
    {
        return Fiber::suspend(AsyncFilesystem::getContents($path));
    }

    public static function putContents(string $path, string $contents): bool
    {
        return Fiber::suspend(AsyncFilesystem::putContents($path, $contents));
    }

    // --- Stateful Stream Operations (fopen replacement) ---

    public static function open(string $path, string $mode = 'r'): ?FileStream
    {
        $handle = Fiber::suspend(AsyncFileHandle::open($path, $mode));
        if (!$handle) return null;
        return new FileStream($handle);
    }
}

class FileStream
{
    private AsyncFileHandle $handle;

    public function __construct(AsyncFileHandle $handle)
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
