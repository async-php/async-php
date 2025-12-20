<?php

namespace Async\Network\Tcp;

use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO\ReaderFrom;
use Async\IO\WriterTo;
use Async\IO;
use Async\Kernel\Network\TcpStream as KernelTcpStream;
use Fiber;

/**
 * Socket represents a TCP connection
 * Implements IO interfaces directly
 */
class Socket implements Reader, Writer, Closer, ReaderFrom, WriterTo
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

    public function flush(): void
    {
        // TCP streams flush automatically
    }

    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }

    public function remoteAddr(): string
    {
        return $this->inner->peerAddr();
    }

    public function localAddr(): string
    {
        return $this->inner->localAddr();
    }

    /**
     * Peek at incoming data without removing it from the buffer
     */
    public function peek(int $length): ?string
    {
        $future = $this->inner->peek($length);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    /**
     * Get the value of the TCP_NODELAY option
     */
    public function nodelay(): bool
    {
        return $this->inner->nodelay();
    }

    /**
     * Set the value of the TCP_NODELAY option
     */
    public function setNodelay(bool $nodelay): bool
    {
        return $this->inner->set_nodelay($nodelay);
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
     * Get the value of the SO_LINGER option
     * @return int Linger timeout in seconds, or -1 if disabled
     */
    public function linger(): int
    {
        return $this->inner->linger();
    }

    /**
     * Set the value of the SO_LINGER option
     * @param int $secs Linger timeout in seconds, or -1 to disable
     */
    public function setLinger(int $secs): bool
    {
        return $this->inner->set_linger($secs);
    }

    /**
     * @deprecated Use remoteAddr() instead
     */
    public function getPeerAddress(): string
    {
        return $this->remoteAddr();
    }

    public function readFrom(Reader $reader): int
    {
        $totalWritten = 0;
        $bufferSize = 8192;

        while (true) {
            $data = $reader->read($bufferSize);
            if ($data === null || $data === '') {
                break;
            }

            $n = $this->write($data);
            $totalWritten += $n;

            if ($n < strlen($data)) {
                break;
            }
        }

        return $totalWritten;
    }

    public function writeTo(Writer $writer): int
    {
        $totalWritten = 0;
        $bufferSize = 8192;

        while (true) {
            $data = $this->read($bufferSize);
            if ($data === null || $data === '') {
                break;
            }

            $n = $writer->write($data);
            $totalWritten += $n;

            if ($n < strlen($data)) {
                break;
            }
        }

        return $totalWritten;
    }

    /**
     * Cast underlying kernel stream into a Kernel IO wrapper by bitflags.
     *
     * @return mixed Kernel IO object (AsyncReader/AsyncWriter/AsyncReadWriter/...)
     */
    public function castTo(int $type)
    {
        return $this->inner->castTo($type);
    }
}
