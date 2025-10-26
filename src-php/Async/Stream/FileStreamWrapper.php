<?php

namespace Async\Stream;

use Async\Kernel\FileSystem as KernelFileSystem;
use Async\Kernel\FileSystem\FileHandle;
use Fiber;

class FileStreamWrapper
{
    public $context;
    private ?FileHandle $handle = null;
    
    private ?array $dirEntries = null;
    private int $dirIndex = 0;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        if (strpos($path, 'file://') === 0) {
            $path = substr($path, 7);
        }
        
        $this->handle = Fiber::suspend(FileHandle::open($path, $mode));
        
        if ($this->handle && ($options & STREAM_USE_PATH)) {
             $opened_path = $path;
        }
        
        return $this->handle !== null;
    }

    public function stream_read(int $count): string|false
    {
        if (!$this->handle) return false;
        return Fiber::suspend($this->handle->read($count));
    }

    public function stream_write(string $data): int|false
    {
        if (!$this->handle) return false;
        return Fiber::suspend($this->handle->write($data));
    }
    
    public function stream_seek(int $offset, int $whence = SEEK_SET): bool 
    {
         if ($whence !== SEEK_SET) return false; 
         $pos = Fiber::suspend($this->handle->seek($offset));
         return $pos !== false;
    }
    
    public function stream_tell(): int
    {
         return 0; 
    }
    
    public function stream_eof(): bool
    {
        return false; 
    }
    
    public function stream_stat(): array|false
    {
        return [
            'dev' => 0, 'ino' => 0, 'mode' => 0100644, 'nlink' => 1,
            'uid' => 0, 'gid' => 0, 'rdev' => 0, 'size' => 0,
            'atime' => 0, 'mtime' => 0, 'ctime' => 0, 'blksize' => 4096, 'blocks' => 1,
        ];
    }
    
    public function stream_close(): void
    {
        if ($this->handle) {
            Fiber::suspend($this->handle->close());
            $this->handle = null;
        }
    }
    
    // Directory Ops
    public function dir_opendir(string $path, int $options): bool
    {
        if (strpos($path, 'file://') === 0) {
            $path = substr($path, 7);
        }
        $entries = Fiber::suspend(KernelFileSystem::scandir($path));
        if (!is_array($entries)) return false;
        
        $this->dirEntries = $entries;
        $this->dirIndex = 0;
        return true;
    }
    
    public function dir_readdir(): string|false
    {
        if (!$this->dirEntries || !isset($this->dirEntries[$this->dirIndex])) {
            return false;
        }
        return $this->dirEntries[$this->dirIndex++];
    }
    
    public function dir_rewinddir(): bool
    {
        $this->dirIndex = 0;
        return true;
    }
    
    public function dir_closedir(): bool
    {
        $this->dirEntries = null;
        return true;
    }
    
    public function url_stat(string $path, int $flags): array|false
    {
        if (strpos($path, 'file://') === 0) {
            $path = substr($path, 7);
        }
        
        $size = Fiber::suspend(KernelFileSystem::size($path));
        $isDir = Fiber::suspend(KernelFileSystem::isDir($path));
        $isFile = Fiber::suspend(KernelFileSystem::isFile($path));
        
        if ($size === false && !$isDir && !$isFile) return false;
        
        $mode = 0;
        if ($isDir) $mode |= 0040000;
        if ($isFile) $mode |= 0100000;
        
        return [
            'dev' => 0, 'ino' => 0, 'mode' => $mode, 'nlink' => 1,
            'uid' => 0, 'gid' => 0, 'rdev' => 0, 'size' => $size ?: 0,
            'atime' => 0, 'mtime' => 0, 'ctime' => 0, 'blksize' => 4096, 'blocks' => 1,
        ];
    }
    
    public function unlink(string $path): bool
    {
        if (strpos($path, 'file://') === 0) $path = substr($path, 7);
        return Fiber::suspend(KernelFileSystem::unlink($path));
    }
    
    public function rename(string $path_from, string $path_to): bool
    {
        if (strpos($path_from, 'file://') === 0) $path_from = substr($path_from, 7);
        if (strpos($path_to, 'file://') === 0) $path_to = substr($path_to, 7);
        return Fiber::suspend(KernelFileSystem::rename($path_from, $path_to));
    }
    
    public function mkdir(string $path, int $mode, int $options): bool
    {
        if (strpos($path, 'file://') === 0) $path = substr($path, 7);
        return Fiber::suspend(KernelFileSystem::mkdir($path));
    }
    
    public function rmdir(string $path, int $options): bool
    {
        if (strpos($path, 'file://') === 0) $path = substr($path, 7);
        return Fiber::suspend(KernelFileSystem::rmdir($path));
    }
}
