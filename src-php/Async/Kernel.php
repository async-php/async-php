<?php

namespace Async;

use RustFuture;

class Kernel
{
    public static function run(callable $main): void
    {
        if (!function_exists('run')) {
            throw new \RuntimeException("Async extension not loaded.");
        }

        $fiber = new \Fiber($main);
        \run($fiber);
    }

    public static function sleep(int $ms): void
    {
        \Fiber::suspend(RustFuture::sleep($ms));
    }

    public static function spawn(callable $task): void
    {
        RustFuture::spawn($task);
    }
}
