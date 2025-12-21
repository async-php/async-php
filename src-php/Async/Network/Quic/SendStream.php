<?php

namespace Async\Network\Quic;

use Async\IO\Writer;
use Async\IO\Closer;
use Async\IO\TokioIO;
use Async\IO;
use Async\Kernel\Network\Quic\QuicSendStream as KernelQuicSendStream;
use Fiber;

/**
 * QUIC Send Stream (Unidirectional)
 *
 * A send-only stream in a QUIC connection for transmitting data to the peer.
 */
class SendStream implements Writer, Closer, TokioIO
{
    private KernelQuicSendStream $inner;

    /**
     * @internal
     */
    public function __construct(KernelQuicSendStream $inner)
    {
        $this->inner = $inner;
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
     */
    public function flush(): void
    {
        $future = $this->inner->flush();
        Fiber::suspend($future);
    }

    /**
     * Close the stream and send FIN
     *
     * @return bool True if closed successfully
     */
    public function close(): bool
    {
        $future = $this->inner->close();
        return (bool)Fiber::suspend($future);
    }

    /**
     * Finish writing without fully closing
     *
     * @return bool True if finished successfully
     */
    public function finish(): bool
    {
        $future = $this->inner->finish();
        return (bool)Fiber::suspend($future);
    }

    /**
     * Reset the stream with an error code
     *
     * @param int $errorCode Application error code
     * @return bool True if reset successfully
     */
    public function reset(int $errorCode = 0): bool
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
     * Get the underlying kernel send stream
     *
     * @internal
     * @return KernelQuicSendStream
     */
    public function unwrap(): KernelQuicSendStream
    {
        return $this->inner;
    }

    /**
     * Cast to an IO wrapper based on bitflags
     *
     * @param int $type Bitflags (IO::WRITE)
     * @return mixed Wrapper IO object
     */
    public function castTo(int $type)
    {
        $kernelIo = $this->inner->castTo($type);
        return IO::kernelToWrapper($kernelIo);
    }
}