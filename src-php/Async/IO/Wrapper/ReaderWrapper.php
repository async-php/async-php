<?php

namespace Async\IO\Wrapper;

use Async\IO\Reader;
use Async\Kernel\IO\AsyncReader;
use Fiber;

/**
 * ReaderWrapper wraps AsyncReader and implements Reader interface
 */
class ReaderWrapper implements Reader
{
    private AsyncReader $inner;

    public function __construct(AsyncReader $inner)
    {
        $this->inner = $inner;
    }

    public function read(int $length): ?string
    {
        $future = $this->inner->read($length);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    /**
     * Get the underlying AsyncReader
     */
    public function unwrap(): AsyncReader
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
