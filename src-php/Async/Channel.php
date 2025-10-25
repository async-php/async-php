<?php

namespace Async;

use Async\Driver\Channel as DriverChannel;
use Fiber;
use RustFuture;

class Channel
{
    private DriverChannel $inner;

    public function __construct()
    {
        $this->inner = new DriverChannel();
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
