<?php

namespace Async;

use Async\Kernel\FileSystem as KernelFileSystem;
use Fiber;

class FileSystem
{
    public static function getContents(string $path): string|false
    {
        $future = KernelFileSystem::getContents($path);
        return Fiber::suspend($future);
    }

    public static function putContents(string $path, string $contents): bool
    {
        $future = KernelFileSystem::putContents($path, $contents);
        return Fiber::suspend($future);
    }

    public static function exists(string $path): bool
    {
        $future = KernelFileSystem::exists($path);
        return Fiber::suspend($future);
    }
    
    public static function isFile(string $path): bool
    {
        $future = KernelFileSystem::isFile($path);
        return Fiber::suspend($future);
    }

    public static function isDir(string $path): bool
    {
        $future = KernelFileSystem::isDir($path);
        return Fiber::suspend($future);
    }

    public static function unlink(string $path): bool
    {
        $future = KernelFileSystem::unlink($path);
        return Fiber::suspend($future);
    }

    public static function rename(string $from, string $to): bool
    {
        $future = KernelFileSystem::rename($from, $to);
        return Fiber::suspend($future);
    }

    public static function copy(string $from, string $to): bool
    {
        $future = KernelFileSystem::copy($from, $to);
        return Fiber::suspend($future);
    }

    public static function mkdir(string $path): bool
    {
        $future = KernelFileSystem::mkdir($path);
        return Fiber::suspend($future);
    }

    public static function rmdir(string $path): bool
    {
        $future = KernelFileSystem::rmdir($path);
        return Fiber::suspend($future);
    }

    public static function size(string $path): int|false
    {
        $future = KernelFileSystem::size($path);
        return Fiber::suspend($future);
    }
    
    public static function scandir(string $path): array|false
    {
        $future = KernelFileSystem::scandir($path);
        return Fiber::suspend($future);
    }
}
