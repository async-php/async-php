<?php

namespace Async\IO\Wrapper;

use Async\IO\Seeker;
use Async\IO\TokioIO;
use Async\Kernel\IO\AsyncSeeker;
use Fiber;

/**
 * SeekerWrapper wraps AsyncSeeker and implements Seeker interface
 */
class SeekerWrapper implements Seeker, TokioIO
{
    private AsyncSeeker $inner;

    public function __construct(AsyncSeeker $inner)
    {
        $this->inner = $inner;
    }

    public function seek(int $offset, int $whence = self::SEEK_START): int
    {
        $future = $this->inner->seek($offset, $whence);
        $result = Fiber::suspend($future);
        return (int)$result;
    }

    /**
     * Get the underlying AsyncSeeker
     */
    public function unwrap(): AsyncSeeker
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
