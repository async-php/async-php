<?php
/**
 * Ticker ForEach Example
 *
 * This example demonstrates using Ticker with foreach loops and callbacks.
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Time\Ticker;

echo "Ticker ForEach Example\n";
echo str_repeat("=", 70) . "\n\n";

Kernel::run(function () {
    // ========================================================================
    // Example 1: Using Ticker with foreach Loop
    // ========================================================================
    echo "Example 1: Using Ticker with foreach Loop\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(300);
    $ticker->setMaxTicks(5);

    echo "Iterating over ticker with foreach (max 5 ticks)...\n\n";

    foreach ($ticker as $tickNumber) {
        echo sprintf(
            "[%s] Iteration: Tick #%d\n",
            date('H:i:s.v'),
            $tickNumber
        );
    }

    echo "\nForEach loop completed!\n\n";

    // ========================================================================
    // Example 2: Using forEach() Callback
    // ========================================================================
    echo "Example 2: Using forEach() Callback\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(250);
    $ticker->setMaxTicks(4);

    echo "Using forEach callback (max 4 ticks)...\n\n";

    $ticker->forEach(function ($tickNumber) {
        echo sprintf(
            "[%s] Callback: Tick #%d - Counter value: %d\n",
            date('H:i:s.v'),
            $tickNumber,
            $tickNumber * 10
        );
    });

    echo "\nforEach callback completed!\n\n";

    // ========================================================================
    // Example 3: Combining with Business Logic
    // ========================================================================
    echo "Example 3: Combining with Business Logic\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(200);
    $ticker->setMaxTicks(6);

    echo "Simulating a heartbeat monitor...\n\n";

    $heartRate = 70; // BPM

    foreach ($ticker as $tick) {
        // Simulate heartbeat variation
        $variation = rand(-5, 5);
        $currentRate = $heartRate + $variation;

        $status = match (true) {
            $currentRate < 60 => 'LOW',
            $currentRate > 80 => 'HIGH',
            default => 'NORMAL'
        };

        echo sprintf(
            "[%s] Heartbeat #%d: %d BPM [%s]\n",
            date('H:i:s.v'),
            $tick,
            $currentRate,
            $status
        );
    }

    echo "\nHeartbeat monitoring completed!\n\n";

    // ========================================================================
    // Example 4: Progress Bar with Ticker
    // ========================================================================
    echo "Example 4: Progress Bar with Ticker\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(150);
    $ticker->setMaxTicks(10);

    echo "Simulating a download progress bar...\n\n";

    $totalSize = 100; // MB

    foreach ($ticker as $tick) {
        $downloaded = ($tick / 10) * $totalSize;
        $progress = ($tick / 10) * 100;
        $bar = str_repeat('=', (int)($progress / 10)) . str_repeat('-', 10 - (int)($progress / 10));

        echo sprintf(
            "\r[%s] [%s] %.1f%% (%.1f MB / %d MB)",
            date('H:i:s.v'),
            $bar,
            $progress,
            $downloaded,
            $totalSize
        );
    }

    echo "\n\nDownload completed!\n";
});

echo "\n" . str_repeat("=", 70) . "\n";
echo "All forEach examples completed!\n";
