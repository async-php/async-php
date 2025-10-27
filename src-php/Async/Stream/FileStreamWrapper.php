<?php

namespace Async\Stream;

use Async\Kernel\FileSystem as KernelFileSystem;
use Async\Kernel\FileSystem\FileHandle;
use Fiber;

class FileStreamWrapper
{
    public $context;
    private ?FileHandle $handle = null;
    private ?string $path = null;
    private int $position = 0;
    private bool $eof = false;
    private ?int $size = null;
    
    private ?array $dirEntries = null;
    private int $dirIndex = 0;

    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool
    {
        $path = $this->stripScheme($path);

        $this->handle = Fiber::suspend(FileHandle::open($path, $mode));
        if (!$this->handle) {
            $this->path = null;
            return false;
        }

        $this->path = $path;
        $this->position = 0;
        $this->eof = false;

        // Preload size for EOF/tell, best effort.
        $this->size = Fiber::suspend(KernelFileSystem::size($path));
        if ($this->size === false) {
            $this->size = null;
        }
        // For append modes, pointer starts at end.
        if (str_contains($mode, 'a') && $this->size !== null) {
            $this->position = $this->size;
        }

        if ($this->handle && ($options & STREAM_USE_PATH)) {
             $opened_path = $path;
        }
        
        return $this->handle !== null;
    }

    public function stream_read(int $count): string|false
    {
        if (!$this->handle) return false;
        $data = Fiber::suspend($this->handle->read($count));
        if ($data === false) {
            return false;
        }

        $len = strlen($data);
        $this->position += $len;
        if ($len === 0) {
            $this->eof = true;
        } elseif ($this->size !== null && $this->position >= $this->size) {
            $this->eof = true;
        }

        return $data;
    }

    public function stream_write(string $data): int|false
    {
        if (!$this->handle) return false;
        $written = Fiber::suspend($this->handle->write($data));
        if (!is_int($written)) {
            return false;
        }
        $this->position += $written;
        $this->size = $this->size === null ? $this->position : max($this->size, $this->position);
        $this->eof = false;
        return $written;
    }
    
    public function stream_seek(int $offset, int $whence = SEEK_SET): bool 
    {
         if (!$this->handle) return false;

         $target = $offset;
         if ($whence === SEEK_CUR) {
             $target = $this->position + $offset;
         } elseif ($whence === SEEK_END) {
             if ($this->size === null) {
                 $size = Fiber::suspend(KernelFileSystem::size($this->path ?? ''));
                 $this->size = $size === false ? null : $size;
             }
             if ($this->size === null) return false;
             $target = $this->size + $offset;
         }

         if ($target < 0) return false;

         $pos = Fiber::suspend($this->handle->seek($target));
         if ($pos === false) return false;

         $this->position = $target;
         $this->eof = ($this->size !== null && $this->position >= $this->size);
         return true;
    }
    
    public function stream_tell(): int
    {
         return $this->position;
    }
    
    public function stream_eof(): bool
    {
        return $this->eof;
    }
    
    public function stream_stat(): array|false
    {
        if ($this->path === null) {
            return false;
        }
        return $this->url_stat($this->path, 0);
    }
    
    public function stream_close(): void
    {
        if ($this->handle) {
            Fiber::suspend($this->handle->close());
            $this->handle = null;
        }
    }

    public function stream_flush(): bool
    {
        // Underlying async writes are not buffered in PHP space.
        return true;
    }

    public function stream_set_option(int $option, int $arg1, int $arg2): bool
    {
        // Async FS is non-blocking; honor API by accepting options.
        return match ($option) {
            STREAM_OPTION_BLOCKING,
            STREAM_OPTION_READ_TIMEOUT,
            STREAM_OPTION_WRITE_BUFFER => true,
            default => false,
        };
    }
    
    // Directory Ops
    public function dir_opendir(string $path, int $options): bool
    {
        $path = $this->stripScheme($path);
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
        $path = $this->stripScheme($path);
        
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
        $path = $this->stripScheme($path);
        return Fiber::suspend(KernelFileSystem::unlink($path));
    }
    
    public function rename(string $path_from, string $path_to): bool
    {
        $path_from = $this->stripScheme($path_from);
        $path_to = $this->stripScheme($path_to);
        return Fiber::suspend(KernelFileSystem::rename($path_from, $path_to));
    }
    
    public function mkdir(string $path, int $mode, int $options): bool
    {
        $path = $this->stripScheme($path);
        return Fiber::suspend(KernelFileSystem::mkdir($path));
    }
    
    public function rmdir(string $path, int $options): bool
    {
        $path = $this->stripScheme($path);
        return Fiber::suspend(KernelFileSystem::rmdir($path));
    }

    private function stripScheme(string $path): string
    {
        return str_starts_with($path, 'file://') ? substr($path, 7) : $path;
    }
}
