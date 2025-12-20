<?php

namespace Async\IO\Wrapper;

use Async\IO\ReadSeeker;
use Async\Kernel\IO\AsyncReadSeeker;
use Fiber;

/**
 * ReadSeekerWrapper wraps AsyncReadSeeker and implements ReadSeeker interface
 */
class ReadSeekerWrapper implements ReadSeeker
{
    private AsyncReadSeeker $inner;

    public function __construct(AsyncReadSeeker $inner)
    {
        $this->inner = $inner;
    }

    public function read(int $length): ?string
    {
        $future = $this->inner->read($length);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    public function seek(int $offset, int $whence = self::SEEK_START): int
    {
        $future = $this->inner->seek($offset, $whence);
        $result = Fiber::suspend($future);
        return (int)$result;
    }

    /**
     * Get the underlying AsyncReadSeeker
     */
    public function unwrap(): AsyncReadSeeker
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
