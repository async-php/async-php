<?php

require __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    echo "=== Async Time Features Test ===\n\n";

    // 1. High-precision Sleep (Float seconds)
    echo "[Test 1] Sleep 0.5s (float)...\n";
    $start = Time::now();
    Time::sleep(0.5);
    $diff = Time::now() - $start;
    echo "  Done. Actual time: " . number_format($diff, 4) . "s\n\n";

    // 2. Microsecond Sleep (usleep)
    echo "[Test 2] Sleep 200,000us (0.2s)...\n";
    $start = Time::now();
    Time::usleep(200000);
    $diff = Time::now() - $start;
    echo "  Done. Actual time: " . number_format($diff, 4) . "s\n\n";

    // 3. Timeout Success
    echo "[Test 3] Timeout success (task takes 0.1s, timeout 0.5s)...\n";
    try {
        // We create a sleep future but don't await it immediately
        // In a real app this would be an HTTP request or DB query
        $future = \Async\Kernel\Time::sleep(0.1); 
        $result = Time::timeout($future, 0.5);
        echo "  Success! Task completed in time.\n\n";
    } catch (\Throwable $e) {
        echo "  Failed: " . $e->getMessage() . "\n\n";
    }

    // 4. Timeout Failure
    echo "[Test 4] Timeout failure (task takes 0.5s, timeout 0.1s)...\n";
    try {
        $future = \Async\Kernel\Time::sleep(0.5);
        $result = Time::timeout($future, 0.1);
        echo "  Unexpected success!\n\n";
    } catch (\Throwable $e) {
        echo "  Caught expected error: " . $e->getMessage() . "\n\n";
    }

    // 5. Concurrent Timers
    echo "[Test 5] Concurrent timers...\n";
    $start = Time::now();
    
    go(function() {
        Time::sleep(0.2);
        echo "  [Task A] Finished at " . number_format(Time::now() - $GLOBALS['start'], 4) . "s\n";
    });

    go(function() {
        Time::sleep(0.1);
        echo "  [Task B] Finished at " . number_format(Time::now() - $GLOBALS['start'], 4) . "s\n";
    });

    $GLOBALS['start'] = $start;
    Time::sleep(0.3);
    echo "  Main finished.\n";
});