<?php

namespace Async;

use AsyncChannel;
use Fiber;
use RustFuture;

class Channel
{
    private AsyncChannel $inner;

    public function __construct()
    {
        $this->inner = new AsyncChannel();
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
