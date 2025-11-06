<?php

namespace Async\Network\Tcp;

use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\Kernel\Network\TcpStream as KernelTcpStream;
use Fiber;

class Socket implements Reader, Writer, Closer
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

    public function read(int $length): ?string
    {
        $future = $this->inner->read($length);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    public function write(string $data): int
    {
        $future = $this->inner->write($data);
        $result = Fiber::suspend($future);
        return $result ?: 0;
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return Fiber::suspend($future);
    }

    public function remoteAddr(): string
    {
        return $this->inner->peer_addr();
    }

    public function localAddr(): string
    {
        return $this->inner->local_addr();
    }

    /**
     * @deprecated Use remoteAddr() instead
     */
    public function getPeerAddress(): string
    {
        return $this->remoteAddr();
    }
}
