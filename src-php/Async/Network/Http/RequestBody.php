<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\RequestBody as KernelRequestBody;
use Fiber;

class RequestBody
{
    protected KernelRequestBody $kernel;

    public function __construct(KernelRequestBody $kernel)
    {
        $this->kernel = $kernel;
    }

    public function read(int $length = 8192): string
    {
        return Fiber::suspend($this->kernel->read($length));
    }
}
