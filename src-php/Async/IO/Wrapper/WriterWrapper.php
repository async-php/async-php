<?php

namespace Async\IO\Wrapper;

use Async\IO\Writer;
use Async\IO\TokioIO;
use Async\Kernel\IO\AsyncWriter;
use Fiber;

/**
 * WriterWrapper wraps AsyncWriter and implements Writer interface
 */
class WriterWrapper implements Writer, TokioIO
{
    private AsyncWriter $inner;

    public function __construct(AsyncWriter $inner)
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

    /**
     * Get the underlying AsyncWriter
     */
    public function unwrap(): AsyncWriter
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
