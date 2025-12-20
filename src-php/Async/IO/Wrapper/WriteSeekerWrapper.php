<?php

namespace Async\IO\Wrapper;

use Async\IO\WriteSeeker;
use Async\IO\AsyncIO;
use Async\Kernel\IO\AsyncWriteSeeker;
use Fiber;

/**
 * WriteSeekerWrapper wraps AsyncWriteSeeker and implements WriteSeeker interface
 */
class WriteSeekerWrapper implements WriteSeeker, AsyncIO
{
    private AsyncWriteSeeker $inner;

    public function __construct(AsyncWriteSeeker $inner)
    {
        $this->inner = $inner;
    }

    public function write(string $data): int
    {
        $future = $this->inner->write($data);
        $result = Fiber::suspend($future);
        return $result ?: 0;
    }

    public function flush(): void
    {
        $future = $this->inner->flush();
        Fiber::suspend($future);
    }

    public function seek(int $offset, int $whence = self::SEEK_START): int
    {
        $future = $this->inner->seek($offset, $whence);
        $result = Fiber::suspend($future);
        return (int)$result;
    }

    /**
     * Get the underlying AsyncWriteSeeker
     */
    public function unwrap(): AsyncWriteSeeker
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
