<?php

namespace Async\Network\Tcp;

use Async\Kernel\Network\TcpListener as KernelTcpListener;
use Fiber;

class Server
{
    private KernelTcpListener $inner;

    public function __construct(KernelTcpListener $inner)
    {
        $this->inner = $inner;
    }

    public static function bind(string $addr): self
    {
        $future = KernelTcpListener::bind($addr);
        $kernelListener = Fiber::suspend($future);
        if (!$kernelListener) {
            throw new \RuntimeException("Failed to bind to $addr");
        }
        return new self($kernelListener);
    }

    public function accept(): Socket
    {
        $future = $this->inner->accept();
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Accept failed");
        }
        return new Socket($kernelStream);
    }

    public function getLocalAddress(): string
    {
        return $this->inner->local_addr();
    }
}