<?php

namespace Async\Network\Quic;

use Async\Kernel\Network\Quic\QuicListener as KernelQuicListener;
use Fiber;

/**
 * QUIC Listener for accepting incoming QUIC connections
 *
 * QUIC is a modern transport protocol that provides features like:
 * - Built-in TLS 1.3 encryption
 * - Multiplexed streams
 * - Fast connection establishment (0-RTT)
 * - Connection migration
 * - Improved congestion control
 */
class Listener
{
    private KernelQuicListener $inner;

    private function __construct(KernelQuicListener $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Bind to an address to listen for incoming QUIC connections
     *
     * @param string $addr Address to bind to (e.g., "0.0.0.0:443", "[::]:4433")
     * @param array|null $tlsConfig TLS configuration array with keys:
     *   - 'cert_path': Path to certificate file (PEM format)
     *   - 'key_path': Path to private key file (PEM format)
     *   - 'alpn': Array of ALPN protocols (e.g., ['h3', 'h3-29'])
     * @return self
     * @throws \RuntimeException if binding fails
     */
    public static function bind(string $addr, ?array $tlsConfig = null): self
    {
        $future = KernelQuicListener::bind($addr, $tlsConfig ?? []);
        $kernelListener = Fiber::suspend($future);
        if (!$kernelListener) {
            throw new \RuntimeException("Failed to bind QUIC listener to $addr");
        }
        return new self($kernelListener);
    }

    /**
     * Accept a new incoming QUIC connection
     *
     * @return Connection The accepted QUIC connection
     * @throws \RuntimeException if accepting fails
     */
    public function accept(): Connection
    {
        $future = $this->inner->accept();
        $kernelConnection = Fiber::suspend($future);
        if (!$kernelConnection) {
            throw new \RuntimeException("Failed to accept QUIC connection");
        }
        return new Connection($kernelConnection);
    }

    /**
     * Get the local address this listener is bound to
     *
     * @return string Local address (e.g., "127.0.0.1:443")
     */
    public function localAddr(): string
    {
        return $this->inner->localAddr();
    }

    /**
     * Close the QUIC listener
     *
     * @return bool True if closed successfully
     */
    public function close(): bool
    {
        return $this->inner->close();
    }

    /**
     * Get the underlying kernel QUIC listener
     *
     * @internal
     * @return KernelQuicListener
     */
    public function unwrap(): KernelQuicListener
    {
        return $this->inner;
    }
}