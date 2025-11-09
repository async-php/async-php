<?php

namespace Async\Network\Tcp;

use Async\Kernel\Network\TcpListener as KernelTcpListener;
use Fiber;

/**
 * TCP Listener for accepting incoming connections
 */
class Listener
{
    private KernelTcpListener $inner;

    private function __construct(KernelTcpListener $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Bind to an address to listen for incoming connections
     */
    public static function bind(string $addr): self
    {
        $future = KernelTcpListener::bind($addr);
        $kernelListener = Fiber::suspend($future);
        if (!$kernelListener) {
            throw new \RuntimeException("Failed to bind to $addr");
        }
        return new self($kernelListener);
    }

    /**
     * Accept a new incoming connection
     * @return Socket The accepted TCP socket
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
     * Get the local address this listener is bound to
     */
    public function localAddr(): string
    {
        return $this->inner->local_addr();
    }

    /**
     * Get the value of the IP_TTL option
     */
    public function ttl(): int
    {
        return $this->inner->ttl();
    }

    /**
     * Set the value of the IP_TTL option
     */
    public function setTtl(int $ttl): bool
    {
        return $this->inner->set_ttl($ttl);
    }

    /**
     * Get the underlying kernel TCP listener
     * @internal
     */
    public function unwrap(): KernelTcpListener
    {
        return $this->inner;
    }
}
