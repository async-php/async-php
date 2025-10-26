<?php

namespace Async;

use Async\Kernel\Time as KernelTime;
use Async\Kernel\Ticker as KernelTicker;
use Fiber;

class Time
{
    public static function sleep(int $ms): void
    {
        $future = KernelTime::sleep($ms);
        Fiber::suspend($future);
    }
    
    public static function after(int $ms): void
    {
        $future = KernelTime::after($ms);
        Fiber::suspend($future);
    }
    
    public static function now(): int
    {
        return KernelTime::now();
    }
    
    public static function createTicker(int $intervalMs): Ticker
    {
        $kernelTicker = KernelTime::create_ticker($intervalMs);
        return new Ticker($kernelTicker);
    }
}
