<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpBody as KernelHttpBody;

class RequestBody extends Body
{
    public function __construct(KernelHttpBody $kernel)
    {
        parent::__construct($kernel);
    }
}
