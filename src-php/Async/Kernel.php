<?php

namespace Async;

use Async\Time;

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
        \Fiber::suspend(Time::sleep($ms));
    }

    public static function spawn(callable $task): void
    {
        \go($task);
    }

    public static function setupLog(array $config): void
    {
        \Async\Logger::init($config);
    }
}
