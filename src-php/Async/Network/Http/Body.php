<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpBody as KernelHttpBody;
use Fiber;

class Body
{
    public function __construct(protected KernelHttpBody $kernel)
    {
    }

    public static function fromReadCloser(object $readCloser): self
    {
        return new self(new KernelHttpBody($readCloser));
    }

    public static function fromString(string $data): self
    {
        return new self(KernelHttpBody::from_string($data));
    }

    public function read(int $length = 8192): ?string
    {
        return Fiber::suspend($this->kernel->read($length));
    }

    public function close(): bool
    {
        return Fiber::suspend($this->kernel->close());
    }

    public function getKernel(): KernelHttpBody
    {
        return $this->kernel;
    }
}
