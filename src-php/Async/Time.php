<?php

namespace Async;

use Async\Kernel\Time as KernelTime;
use Async\Kernel\Ticker as KernelTicker;
use Fiber;

class Time
{
    /**
     * Sleep for the specified duration (non-blocking).
     *
     * @param float|int $seconds Duration in seconds (e.g., 0.5 for 500ms)
     */
    public static function sleep(float|int $seconds): void
    {
        // KernelTime::sleep expects float seconds
        $future = KernelTime::sleep((float)$seconds);
        Fiber::suspend($future);
    }

    /**
     * Sleep for the specified duration in microseconds (non-blocking).
     *
     * @param int $micros Duration in microseconds
     */
    public static function usleep(int $micros): void
    {
        $future = KernelTime::usleep($micros);
        Fiber::suspend($future);
    }

    /**
     * Execute an asynchronous operation with a timeout.
     *
     * If the operation takes longer than the specified duration,
     * an exception will be thrown.
     *
     * @param object $future The async operation (RustFuture) or Fiber to await
     * @param float $seconds Timeout duration in seconds
     * @return mixed The result of the operation
     * @throws \RuntimeException If the operation times out
     */
    public static function timeout(object $future, float $seconds): mixed
    {
        // For now, we only support RustFuture directly from Kernel
        // In the future, we could support arbitrary callables/fibers
        $timeoutFuture = KernelTime::timeout((float)$seconds, $future);
        return Fiber::suspend($timeoutFuture);
    }

    /**
     * Schedule a callback to run after the specified number of seconds.
     * Note: This runs in a separate background task (fire-and-forget).
     *
     * @param float $seconds Delay in seconds
     * @param callable $callback Function to execute
     */
    public static function timer(float $seconds, callable $callback): void
    {
        KernelTime::timer($seconds, $callback);
    }


    /**
     * Get the current high-resolution timestamp in seconds (float).
     * Similar to microtime(true) but using a monotonic clock if available.
     *
     * @return float
     */
    public static function now(): float
    {
        return KernelTime::now();
    }

    /**
     * Create a ticker that fires at regular intervals.
     *
     * @param int $intervalMs Interval in milliseconds
     * @return Ticker
     */
    public static function createTicker(int $intervalMs): Ticker
    {
        $kernelTicker = KernelTime::create_ticker($intervalMs);
        return new Ticker($kernelTicker);
    }
}