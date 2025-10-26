<?php

namespace Async\Network\Tcp;

use Async\Kernel\Network\TcpStream as KernelTcpStream;
use Fiber;

class Socket
{
    private KernelTcpStream $inner;

    public function __construct(KernelTcpStream $inner)
    {
        $this->inner = $inner;
    }

    public static function connect(string $addr): self
    {
        $future = KernelTcpStream::connect($addr);
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to connect to $addr");
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
    
    public function getPeerAddress(): string
    {
        return $this->inner->peer_addr();
    }
}
