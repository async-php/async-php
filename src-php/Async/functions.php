<?php

/**
 * This file defines user-land replacements for standard PHP functions.
 * To use this, you must disable the native functions in php.ini using 'disable_functions'.
 * Example: disable_functions = sleep,usleep,file_get_contents,file_put_contents
 */

use Async\Time;
use Async\Kernel\FileSystem;

if (!function_exists('sleep')) {
    function sleep(int $seconds): int {
        Time::sleep($seconds * 1000);
        return 0;
    }
}

if (!function_exists('usleep')) {
    function usleep(int $microseconds): void {
        Time::sleep((int)($microseconds / 1000));
    }
}

if (!function_exists('file_get_contents')) {
    function file_get_contents(string $filename, bool $use_include_path = false, $context = null, int $offset = 0, ?int $length = null): string|false {
        // TODO: Support context and offsets properly
        
        // Check for URLs
        if (preg_match('#^https?://#', $filename)) {
            // TODO: Implement Async HTTP Client call here
            return false;
        }
        
        $future = FileSystem::get_contents($filename);
        return \Fiber::suspend($future);
    }
}

if (!function_exists('file_put_contents')) {
    function file_put_contents(string $filename, mixed $data, int $flags = 0, $context = null): int|false {
        // TODO: Support flags (FILE_APPEND)
        
        $future = FileSystem::put_contents($filename, (string)$data);
        return \Fiber::suspend($future);
    }
}
