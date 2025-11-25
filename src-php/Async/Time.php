<?php

namespace Async;

use Async\Kernel\Time as KernelTime;
use Async\Kernel\Ticker as KernelTicker;
use Fiber;

class Time
{
    /**
     * Sleep for the specified number of milliseconds
     */
    public static function sleep(int $ms): void
    {
        $future = KernelTime::sleep($ms);
        Fiber::suspend($future);
    }

    /**
     * Returns a future that resolves after the specified number of milliseconds
     * Use with Fiber::suspend()
     */
    public static function after(int $ms): mixed
    {
        return KernelTime::after($ms);
    }

    /**
     * Schedule a callback to run after the specified number of seconds
     */
    public static function timer(float $seconds, callable $callback): void
    {
        KernelTime::timer($seconds, $callback);
    }

    /**
     * Get the current timestamp in milliseconds
     */
    public static function now(): int
    {
        return KernelTime::now();
    }

    /**
     * Create a ticker that fires at regular intervals
     */
    public static function createTicker(int $intervalMs): Ticker
    {
        $kernelTicker = KernelTime::create_ticker($intervalMs);
        return new Ticker($kernelTicker);
    }
}
