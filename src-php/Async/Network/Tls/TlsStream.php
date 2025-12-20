<?php

namespace Async\Network\Tls;

use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO;
use Async\Kernel\Network\TlsStream as KernelTlsStream;
use Fiber;

/**
 * TLS stream for secure communication
 * Implements IO interfaces directly
 */
class TlsStream implements Reader, Writer, Closer
{
    private KernelTlsStream $inner;

    private function __construct(KernelTlsStream $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Connect to a TLS server with default configuration
     */
    public static function connect(string $host, int $port): self
    {
        $future = KernelTlsStream::connect($host, $port);
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to connect to $host:$port");
        }
        return new self($kernelStream);
    }

    /**
     * Connect to a TLS server with custom configuration
     */
    public static function connectWithConfig(string $host, int $port, TlsConfig $config): self
    {
        $future = KernelTlsStream::connect_with_config($host, $port, $config->unwrap());
        $kernelStream = Fiber::suspend($future);
        if (!$kernelStream) {
            throw new \RuntimeException("Failed to connect to $host:$port with custom config");
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

    public function flush(): void
    {
        // TLS streams flush automatically
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }

    /**
     * Get the remote peer address
     */
    public function peerAddr(): string
    {
        return $this->inner->peer_addr();
    }

    /**
     * Get the local address
     */
    public function localAddr(): string
    {
        return $this->inner->local_addr();
    }

    /**
     * Get the TLS protocol version (e.g., "TLSv1.2", "TLSv1.3")
     */
    public function protocolVersion(): string
    {
        return $this->inner->protocol_version();
    }

    /**
     * Get the negotiated cipher suite
     */
    public function cipherSuite(): string
    {
        return $this->inner->cipher_suite();
    }

    /**
     * Get the SNI hostname (not available in current rustls version)
     */
    public function sniHostname(): string
    {
        return $this->inner->sni_hostname();
    }

    /**
     * Get the negotiated ALPN protocol (e.g., "h2", "http/1.1")
     */
    public function alpnProtocol(): string
    {
        return $this->inner->alpn_protocol();
    }

    /**
     * Get the underlying kernel TLS stream
     * @internal
     */
    public function unwrap(): KernelTlsStream
    {
        return $this->inner;
    }

    /**
     * Cast to an IO wrapper based on bitflags.
     *
     * @param int $type Bitflags (IO::READ | IO::WRITE | IO::SEEK | IO::BUF)
     * @return mixed Wrapper IO object (ReaderWrapper/WriterWrapper/ReadWriterWrapper/...)
     */
    public function castTo(int $type)
    {
        $kernelIo = $this->inner->castTo($type);
        return IO::kernelToWrapper($kernelIo);
    }
}
