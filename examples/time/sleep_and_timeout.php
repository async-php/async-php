<?php
/**
 * Sleep, Timer and Timeout Example
 *
 * This example demonstrates async sleep, timer, and timeout functionality.
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Kernel\Time as KernelTime;
use Async\Time;

echo "Sleep, Timer and Timeout Example\n";
echo str_repeat("=", 70) . "\n\n";

Kernel::run(function () {
    // ========================================================================
    // Example 1: Basic Sleep
    // ========================================================================
    echo "Example 1: Basic Sleep\n";
    echo str_repeat("-", 70) . "\n";

    echo "[" . date('H:i:s.v') . "] Starting sleep for 1 second...\n";
    $start = Time::now();

    Time::sleep(1.0);

    $elapsed = Time::now() - $start;
    echo "[" . date('H:i:s.v') . "] Woke up after " . number_format($elapsed, 3) . " seconds\n\n";

    // ========================================================================
    // Example 2: Microsleep (usleep)
    // ========================================================================
    echo "Example 2: Microsleep (usleep)\n";
    echo str_repeat("-", 70) . "\n";

    echo "[" . date('H:i:s.v') . "] Starting usleep for 500,000 microseconds (0.5s)...\n";
    $start = Time::now();

    Time::usleep(500000);

    $elapsed = Time::now() - $start;
    echo "[" . date('H:i:s.v') . "] Woke up after " . number_format($elapsed * 1000, 1) . " milliseconds\n\n";

    // ========================================================================
    // Example 3: Multiple Concurrent Sleeps
    // ========================================================================
    echo "Example 3: Multiple Concurrent Sleeps\n";
    echo str_repeat("-", 70) . "\n";

    echo "Starting multiple sleeps concurrently...\n\n";

    $start = Time::now();

    // These all run concurrently via the async runtime
    Kernel::spawn(function () {
        Time::sleep(0.5);
        echo "[" . date('H:i:s.v') . "] Task A completed (0.5s)\n";
    });

    Kernel::spawn(function () {
        Time::sleep(1.0);
        echo "[" . date('H:i:s.v') . "] Task B completed (1.0s)\n";
    });

    Kernel::spawn(function () {
        Time::sleep(0.3);
        echo "[" . date('H:i:s.v') . "] Task C completed (0.3s)\n";
    });

    // Wait for all tasks to complete
    Time::sleep(1.2);

    $elapsed = Time::now() - $start;
    echo "\nAll tasks completed in " . number_format($elapsed, 3) . " seconds\n";
    echo "(Should be ~1.2s since they ran concurrently)\n\n";

    // ========================================================================
    // Example 4: Timer Callbacks
    // ========================================================================
    echo "Example 4: Timer Callbacks\n";
    echo str_repeat("-", 70) . "\n";

    echo "Scheduling timers...\n\n";

    Time::timer(0.5, function () {
        echo "[" . date('H:i:s.v') . "] Timer 1: 500ms elapsed\n";
    });

    Time::timer(1.0, function () {
        echo "[" . date('H:i:s.v') . "] Timer 2: 1000ms elapsed\n";
    });

    Time::timer(0.2, function () {
        echo "[" . date('H:i:s.v') . "] Timer 3: 200ms elapsed (first!)\n";
    });

    // Wait for all timers to fire
    Time::sleep(1.2);
    echo "\nAll timers completed!\n\n";

    // ========================================================================
    // Example 5: Timeout - Success Case
    // ========================================================================
    echo "Example 5: Timeout - Success Case\n";
    echo str_repeat("-", 70) . "\n";

    echo "Running a task with 2s timeout (task takes 0.5s)...\n";

    try {
        $future = KernelTime::sleep(0.5);
        $result = Time::timeout($future, 2.0);
        echo "[" . date('H:i:s.v') . "] Task completed successfully within timeout!\n";
    } catch (Exception $e) {
        echo "Error: " . $e->getMessage() . "\n";
    }

    echo "\n";

    // ========================================================================
    // Example 6: Timeout - Timeout Case
    // ========================================================================
    echo "Example 6: Timeout - Timeout Case\n";
    echo str_repeat("-", 70) . "\n";

    echo "Running a task with 0.5s timeout (task takes 2s)...\n";

    try {
        $future = KernelTime::sleep(2.0);
        $result = Time::timeout($future, 0.5);
        echo "Task completed successfully within timeout!\n";
    } catch (Exception $e) {
        echo "[" . date('H:i:s.v') . "] Error: " . $e->getMessage() . "\n";
    }

    echo "\n";

    // ========================================================================
    // Example 7: High-Precision Timing
    // ========================================================================
    echo "Example 7: High-Precision Timing\n";
    echo str_repeat("-", 70) . "\n";

    echo "Measuring precise intervals...\n\n";

    for ($i = 1; $i <= 5; $i++) {
        $start = Time::now();
        Time::usleep(100000); // 100ms
        $elapsed = (Time::now() - $start) * 1000; // Convert to ms

        echo sprintf(
            "Iteration %d: Slept for %.3f ms (target: 100ms)\n",
            $i,
            $elapsed
        );
    }

    echo "\n";

    // ========================================================================
    // Example 8: Countdown Timer
    // ========================================================================
    echo "Example 8: Countdown Timer\n";
    echo str_repeat("-", 70) . "\n";

    echo "Starting 5-second countdown...\n\n";

    for ($i = 5; $i > 0; $i--) {
        echo "[" . date('H:i:s') . "] $i...\n";
        Time::sleep(1.0);
    }

    echo "[" . date('H:i:s') . "] Blast off!\n";
});

echo "\n" . str_repeat("=", 70) . "\n";
echo "All sleep and timeout examples completed!\n";
