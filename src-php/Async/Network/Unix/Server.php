<?php

namespace Async\Network\Unix;

use Async\Kernel\Network\UnixListener as KernelUnixListener;
use Fiber;

class Server
{
    private KernelUnixListener $inner;

    public function __construct(KernelUnixListener $inner)
    {
        $this->inner = $inner;
    }

    public static function bind(string $path): self
    {
        $future = KernelUnixListener::bind($path);
        $kernelListener = Fiber::suspend($future);
        if (!$kernelListener) {
            throw new \RuntimeException("Failed to bind to $path");
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
}