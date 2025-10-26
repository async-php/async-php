<?php

namespace Async\Network\Unix;

use Async\Kernel\Network\UnixStream as KernelUnixStream;
use Fiber;

class Socket
{
    private KernelUnixStream $inner;

    public function __construct(KernelUnixStream $inner)
    {
        $this->inner = $inner;
    }

    public static function connect(string $path): self
    {
        $future = KernelUnixStream::connect($path);
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to connect to $path");
        }
        return new self($kernelStream);
    }

    public function read(int $length = 1024): string|false
    {
        $future = $this->inner->read($length);
        return Fiber::suspend($future);
    }

    public function write(string $data): int|false
    {
        $future = $this->inner->write($data);
        return Fiber::suspend($future);
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return Fiber::suspend($future);
    }
}