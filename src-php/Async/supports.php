<?php

/**
 * This file defines user-land replacements for standard PHP functions.
 * To use this, you must disable the native functions in php.ini using 'disable_functions'.
 * Example: disable_functions = sleep,usleep,file_get_contents,file_put_contents
 */

use Async\Time;

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
        $mode = 'rb';

        $handle = $context !== null
            ? @fopen($filename, $mode, $use_include_path, $context)
            : @fopen($filename, $mode, $use_include_path);

        if ($handle === false) {
            return false;
        }

        if ($offset > 0) {
            if (fseek($handle, $offset) === -1) {
                fclose($handle);
                return false;
            }
        }

        $maxLength = $length ?? -1;
        $data = stream_get_contents($handle, $maxLength);
        fclose($handle);
        
        return $data;
    }
}

if (!function_exists('file_put_contents')) {
    function file_put_contents(string $filename, mixed $data, int $flags = 0, $context = null): int|false {
        $useIncludePath = ($flags & FILE_USE_INCLUDE_PATH) === FILE_USE_INCLUDE_PATH;
        $append = ($flags & FILE_APPEND) === FILE_APPEND;
        $lock = ($flags & LOCK_EX) === LOCK_EX;

        if (is_array($data)) {
            $data = implode('', $data);
        }

        $mode = $append ? 'ab' : 'wb';
        $handle = $context !== null
            ? @fopen($filename, $mode, $useIncludePath, $context)
            : @fopen($filename, $mode, $useIncludePath);

        if ($handle === false) {
            return false;
        }

        if ($lock && !flock($handle, LOCK_EX)) {
            fclose($handle);
            return false;
        }

        $bytesWritten = 0;

        if (is_resource($data) && get_resource_type($data) === 'stream') {
            $bytesWritten = stream_copy_to_stream($data, $handle);
        } else {
            $stringData = (string)$data;
            $length = strlen($stringData);
            while ($bytesWritten < $length) {
                $written = fwrite($handle, substr($stringData, $bytesWritten));
                if ($written === false) {
                    $bytesWritten = false;
                    break;
                }
                $bytesWritten += $written;
            }
        }

        if ($lock) {
            flock($handle, LOCK_UN);
        }
        fclose($handle);

        return $bytesWritten === false ? false : $bytesWritten;
    }
}
