<?php

namespace Async\Network\Quic;

use Async\IO\TokioIO;
use Async\IO;
use Async\Kernel\Network\Quic\QuicConnection as KernelQuicConnection;
use Fiber;

/**
 * QUIC Connection
 *
 * Represents a QUIC connection which supports:
 * - Multiple bidirectional and unidirectional streams
 * - Built-in encryption (TLS 1.3)
 * - Connection migration
 * - 0-RTT resumption
 */
class Connection implements TokioIO
{
    private KernelQuicConnection $inner;

    /**
     * @internal
     */
    public function __construct(KernelQuicConnection $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Connect to a remote QUIC server
     *
     * @param string $addr Server address (e.g., "example.com:443")
     * @param string|null $serverName SNI server name (defaults to addr hostname)
     * @param array|null $config QUIC client configuration:
     *   - 'alpn': Array of ALPN protocols (e.g., ['h3'])
     *   - 'verify_cert': Whether to verify server certificate (default: true)
     *   - 'ca_certs': Path to CA certificate bundle
     * @return self
     * @throws \RuntimeException if connection fails
     */
    public static function connect(string $addr, ?string $serverName = null, ?array $config = null): self
    {
        $cfg = $config ?? [];
        $future = KernelQuicConnection::connect($addr, $serverName, $cfg);
        $kernelConnection = Fiber::suspend($future);
        if (!$kernelConnection) {
            throw new \RuntimeException("Failed to connect to $addr via QUIC");
        }
        return new self($kernelConnection);
    }

    /**
     * Wait for handshake to complete
     *
     * @throws \RuntimeException if handshake fails
     */
    public function handshake(): void
    {
        $future = $this->inner->handshake();
        Fiber::suspend($future);
    }

    /**
     * Open a new bidirectional stream
     *
     * @return Stream Bidirectional QUIC stream for reading and writing
     * @throws \RuntimeException if opening stream fails
     */
    public function openBiStream(): Stream
    {
        $future = $this->inner->openBiStream();
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to open bidirectional stream");
        }
        return new Stream($kernelStream);
    }

    /**
     * Open a new unidirectional stream (send-only)
     *
     * @return SendStream Unidirectional QUIC stream for writing
     * @throws \RuntimeException if opening stream fails
     */
    public function openUniStream(): SendStream
    {
        $future = $this->inner->openUniStream();
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to open unidirectional stream");
        }
        return new SendStream($kernelStream);
    }

    /**
     * Accept the next incoming bidirectional stream
     *
     * @return Stream|null Bidirectional stream, or null if connection closed
     */
    public function acceptBiStream(): ?Stream
    {
        $future = $this->inner->acceptBiStream();
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            return null;
        }
        return new Stream($kernelStream);
    }

    /**
     * Accept the next incoming unidirectional stream
     *
     * @return RecvStream|null Receive-only stream, or null if connection closed
     */
    public function acceptUniStream(): ?RecvStream
    {
        $future = $this->inner->acceptUniStream();
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            return null;
        }
        return new RecvStream($kernelStream);
    }

    /**
     * Get the remote address of this connection
     *
     * @return string Remote address
     */
    public function remoteAddr(): string
    {
        return $this->inner->remoteAddr();
    }

    /**
     * Get the local address of this connection
     *
     * @return string Local address
     */
    public function localAddr(): string
    {
        return $this->inner->localAddr();
    }

    /**
     * Close the QUIC connection gracefully
     *
     * @param int $errorCode Application error code (default: 0)
     * @param string $reason Human-readable reason (default: "")
     * @return bool True if closed successfully
     */
    public function close(int $errorCode = 0, string $reason = ""): bool
    {
        $future = $this->inner->close($errorCode, $reason);
        return (bool)Fiber::suspend($future);
    }

    /**
     * Get the underlying kernel QUIC connection
     *
     * @internal
     * @return KernelQuicConnection
     */
    public function unwrap(): KernelQuicConnection
    {
        return $this->inner;
    }

    /**
     * Cast to an IO wrapper based on bitflags
     *
     * Note: QUIC connections are stream-based, so this returns
     * a wrapper that can handle bidirectional streams.
     *
     * @param int $type Bitflags (IO::READ | IO::WRITE)
     * @return mixed Wrapper IO object
     */
    public function castTo(int $type)
    {
        $kernelIo = $this->inner->castTo($type);
        return IO::kernelToWrapper($kernelIo);
    }
}