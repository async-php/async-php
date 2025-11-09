<?php

namespace Async\Network\Unix;

use Async\Kernel\Network\UnixListener as KernelUnixListener;
use Fiber;

/**
 * Unix Domain Socket Listener for accepting incoming connections
 */
class Listener
{
    private KernelUnixListener $inner;

    private function __construct(KernelUnixListener $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Bind to a path to listen for incoming connections
     * Automatically removes existing socket file if present
     */
    public static function bind(string $path): self
    {
        $future = KernelUnixListener::bind($path);
        $kernelListener = Fiber::suspend($future);
        if (!$kernelListener) {
            throw new \RuntimeException("Failed to bind to $path");
        }
        return new self($kernelListener);
    }

    /**
     * Accept a new incoming connection
     * @return Socket The accepted Unix socket
     */
    public function accept(): Socket
    {
        $future = $this->inner->accept();
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to accept connection");
        }
        return new Socket($kernelStream);
    }

    /**
     * Get the local socket path this listener is bound to
     */
    public function localAddr(): string
    {
        return $this->inner->local_addr();
    }

    /**
     * Get the underlying kernel Unix listener
     * @internal
     */
    public function unwrap(): KernelUnixListener
    {
        return $this->inner;
    }
}
