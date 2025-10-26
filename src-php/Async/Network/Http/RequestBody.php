<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\RequestBody as KernelRequestBody;

class RequestBody
{
    protected KernelRequestBody $kernel;

    public function __construct(KernelRequestBody $kernel)
    {
        $this->kernel = $kernel;
    }

    public function read(int $length = 8192): string
    {
        return $this->kernel->read($length);
    }
}
