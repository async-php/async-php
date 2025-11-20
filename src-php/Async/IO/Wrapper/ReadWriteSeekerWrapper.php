<?php

namespace Async\IO\Wrapper;

use Async\IO\ReadWriteSeeker;
use Async\Kernel\IO\AsyncReadWriteSeeker;
use Fiber;

/**
 * ReadWriteSeekerWrapper wraps AsyncReadWriteSeeker and implements ReadWriteSeeker interface
 */
class ReadWriteSeekerWrapper implements ReadWriteSeeker
{
    private AsyncReadWriteSeeker $inner;

    public function __construct(AsyncReadWriteSeeker $inner)
    {
        $this->inner = $inner;
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
     * Get the underlying AsyncReadWriteSeeker
     */
    public function unwrap(): AsyncReadWriteSeeker
    {
        return $this->inner;
    }
}
