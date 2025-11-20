<?php

namespace Async\IO\Wrapper;

use Async\IO\ReadWriter;
use Async\Kernel\IO\AsyncReadWriter;
use Fiber;

/**
 * ReadWriterWrapper wraps AsyncReadWriter and implements ReadWriter interface
 */
class ReadWriterWrapper implements ReadWriter
{
    private AsyncReadWriter $inner;

    public function __construct(AsyncReadWriter $inner)
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

    /**
     * Get the underlying AsyncReadWriter
     */
    public function unwrap(): AsyncReadWriter
    {
        return $this->inner;
    }
}
