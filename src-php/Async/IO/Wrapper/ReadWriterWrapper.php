<?php

namespace Async\IO\Wrapper;

use Async\IO\ReadWriter;
use Async\IO\AsyncIO;
use Async\Kernel\IO\AsyncReadWriter;
use Fiber;

/**
 * ReadWriterWrapper wraps AsyncReadWriter and implements ReadWriter interface
 */
class ReadWriterWrapper implements ReadWriter, AsyncIO
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
