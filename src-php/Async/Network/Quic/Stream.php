<?php

namespace Async\Network\Quic;

use Async\IO\Reader;
use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO\TokioIO;
use Async\IO;
use Async\Kernel\Network\Quic\QuicStream as KernelQuicStream;
use Fiber;

/**
 * QUIC Bidirectional Stream
 *
 * A bidirectional stream in a QUIC connection that can both send and receive data.
 * Each stream is independent and multiplexed over the same QUIC connection.
 */
class Stream implements Reader, Writer, Closer, TokioIO
{
    private KernelQuicStream $inner;

    /**
     * @internal
     */
    public function __construct(KernelQuicStream $inner)
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
     * Write data to the stream
     *
     * @param string $data Data to write
     * @return int Number of bytes written
     */
    public function write(string $data): int
    {
        $future = $this->inner->write($data);
        $result = Fiber::suspend($future);
        return $result ?: 0;
    }

    /**
     * Flush any buffered data
     *
     * For QUIC streams, this ensures data is sent in the next packet.
     */
    public function flush(): void
    {
        $future = $this->inner->flush();
        Fiber::suspend($future);
    }

    /**
     * Close the stream gracefully
     *
     * Sends a FIN to indicate no more data will be sent.
     *
     * @return bool True if closed successfully
     */
    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }

    /**
     * Finish writing (send FIN) without closing the read side
     *
     * After calling this, you can still read from the stream but cannot write.
     *
     * @return bool True if finished successfully
     */
    public function finish(): bool
    {
        $future = $this->inner->finish();
        return (bool)Fiber::suspend($future);
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
        // TODO: Implement in Rust
        return false;
    }

    /**
     * Reset the stream with an error code
     *
     * Abruptly terminates the stream, discarding unsent data.
     *
     * @param int $errorCode Application error code
     * @return bool True if reset successfully
     */
    public function reset(int $errorCode = 0): bool
    {
         // TODO: Implement in Rust
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
     * Get the underlying kernel QUIC stream
     *
     * @internal
     * @return KernelQuicStream
     */
    public function unwrap(): KernelQuicStream
    {
        return $this->inner;
    }

    /**
     * Cast to an IO wrapper based on bitflags
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