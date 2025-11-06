<?php

namespace Async\IO\Wrapper;

use Async\IO\Seeker;
use Async\Kernel\IO\AsyncSeeker;
use Fiber;

/**
 * SeekerWrapper wraps AsyncSeeker and implements Seeker interface
 */
class SeekerWrapper implements Seeker
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
}
