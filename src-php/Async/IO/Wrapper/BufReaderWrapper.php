<?php

namespace Async\IO\Wrapper;

use Async\IO\BufReader;
use Async\Kernel\IO\AsyncBufReader;
use Fiber;

/**
 * BufReaderWrapper wraps AsyncBufReader and implements BufReader interface
 */
class BufReaderWrapper implements BufReader
{
    private AsyncBufReader $inner;

    public function __construct(AsyncBufReader $inner)
    {
        $this->inner = $inner;
    }

    public function read(int $length): ?string
    {
        // AsyncBufReader doesn't have a read method directly
        // We'll need to implement this differently or use readLine/readUntil
        throw new \RuntimeException("Use readLine() or readUntil() for buffered reading");
    }

    public function readLine(): ?string
    {
        $future = $this->inner->readLine();
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    public function readUntil(int $delim): ?string
    {
        $future = $this->inner->readUntil($delim);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    /**
     * Get the underlying AsyncBufReader
     */
    public function unwrap(): AsyncBufReader
    {
        return $this->inner;
    }

    /**
     * Cast to a different IO wrapper based on bitflags.
     *
     * @param int $type Bitflags (IO::READ | IO::WRITE | IO::SEEK | IO::BUF)
     * @return mixed Wrapper IO object
     */
    public function castTo(int $type)
    {
        $kernelIo = $this->inner->castTo($type);
        return \Async\IO::kernelToWrapper($kernelIo);
    }
}
