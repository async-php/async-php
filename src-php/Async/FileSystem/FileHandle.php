<?php

namespace Async\FileSystem;

use Async\Kernel\FileSystem\FileHandle as KernelFileHandle;
use Fiber;

class FileHandle
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

    public function read(int $length): string|false
    {
        $future = $this->inner->read($length);
        return Fiber::suspend($future);
    }

    public function write(string $data): int|false
    {
        $future = $this->inner->write($data);
        return Fiber::suspend($future);
    }
    
    public function seek(int $offset): int|false
    {
        $future = $this->inner->seek($offset);
        return Fiber::suspend($future);
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return Fiber::suspend($future);
    }
}
