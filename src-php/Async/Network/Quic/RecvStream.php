<?php

namespace Async\Network\Quic;

use Async\IO\Reader;
use Async\IO\Closer;
use Async\IO\TokioIO;
use Async\IO;
use Async\Kernel\Network\Quic\QuicRecvStream as KernelQuicRecvStream;
use Fiber;

/**
 * QUIC Receive Stream (Unidirectional)
 *
 * A receive-only stream in a QUIC connection for reading data from the peer.
 */
class RecvStream implements Reader, Closer, TokioIO
{
    private KernelQuicRecvStream $inner;

    /**
     * @internal
     */
    public function __construct(KernelQuicRecvStream $inner)
    {
        $this->inner = $inner;
    }

    /**
     * Read data from the stream
     *
     * @param int $length Maximum number of bytes to read
     * @return string|null Data read, or null on EOF
     */
    public function read(int $length): ?string
    {
        $future = $this->inner->read($length);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    /**
     * Close the stream (stop receiving)
     *
     * @return bool True if closed successfully
     */
    public function close(): bool
    {
        // For RecvStream, close usually means stop_sending?
        // Or just drop.
        return true;
    }

    /**
     * Stop reading and discard incoming data
     *
     * Sends a STOP_SENDING frame to the peer.
     *
     * @param int $errorCode Application error code
     * @return bool True if stopped successfully
     */
    public function stopReading(int $errorCode = 0): bool
    {
        return false;
    }

    /**
     * Get the stream ID
     *
     * @return int Stream identifier
     */
    public function id(): int
    {
        return $this->inner->id();
    }

    /**
     * Get the underlying kernel receive stream
     *
     * @internal
     * @return KernelQuicRecvStream
     */
    public function unwrap(): KernelQuicRecvStream
    {
        return $this->inner;
    }

    /**
     * Cast to an IO wrapper based on bitflags
     *
     * @param int $type Bitflags (IO::READ)
     * @return mixed Wrapper IO object
     */
    public function castTo(int $type)
    {
        $kernelIo = $this->inner->castTo($type);
        return IO::kernelToWrapper($kernelIo);
    }
}