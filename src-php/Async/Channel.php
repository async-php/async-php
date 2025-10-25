<?php

namespace Async;

use Async\Kernel\Channel as KernelChannel;
use Fiber;
use RustFuture;

class Channel
{
    private KernelChannel $inner;

    public function __construct()
    {
        $this->inner = new KernelChannel();
    }

    public function push(mixed $data): bool
    {
        return $this->inner->send($data);
    }

    public function pop(): mixed
    {
        $future = $this->inner->recv();
        return Fiber::suspend($future);
    }
}
