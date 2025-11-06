<?php

namespace Async\FileSystem;

use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO\Seeker;
use Async\Kernel\FileSystem\FileHandle as KernelFileHandle;
use Fiber;

/**
 * FileHandle represents an open file
 * Implements IO interfaces directly
 */
class FileHandle implements Reader, Writer, Closer, Seeker
{
    private KernelFileHandle $inner;

    public function __construct(KernelFileHandle $inner)
    {
        $this->inner = $inner;
    }

    public static function open(string $path, string $mode): self
    {
        $future = KernelFileHandle::open($path, $mode);
        $kernelHandle = Fiber::suspend($future);
        if (!$kernelHandle) {
            throw new \RuntimeException("Failed to open file $path");
        }
        return new self($kernelHandle);
    }

    public function read(int $length): ?string
    {
        $future = $this->inner->read($length);
        $result = Fiber::suspend($future);
        return ($result === false || $result === '') ? null : $result;
    }

    public function write(string $data): int
    {
        $future = $this->inner->write($data);
        $result = Fiber::suspend($future);
        return $result === false ? 0 : (int)$result;
    }

    public function flush(): void
    {
        // Files flush automatically
    }

    public function seek(int $offset, int $whence = self::SEEK_START): int
    {
        $future = $this->inner->seek($offset, $whence);
        $result = Fiber::suspend($future);

        if ($result === false) {
            throw new \RuntimeException("Seek operation failed");
        }

        return (int)$result;
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }
}
