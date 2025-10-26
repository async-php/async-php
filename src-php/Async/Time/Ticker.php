<?php

namespace Async\Time;

use Async\Kernel\Ticker as KernelTicker;

class Ticker
{
    private KernelTicker $inner;

    public function __construct(KernelTicker $inner)
    {
        $this->inner = $inner;
    }

    public function tick(): void
    {
        $future = $this->inner->next_tick();
        Fiber::suspend($future);
    }

    public function stop(): void
    {
        $this->inner->stop();
    }
}
