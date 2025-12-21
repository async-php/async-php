<?php
/**
 * Basic Ticker Example
 *
 * This example demonstrates how to use a Ticker for periodic operations.
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Time\Ticker;

echo "Basic Ticker Example\n";
echo str_repeat("=", 70) . "\n\n";

Kernel::run(function () {
    // ========================================================================
    // Example 1: Basic Ticker Usage
    // ========================================================================
    echo "Example 1: Basic Ticker Usage\n";
    echo str_repeat("-", 70) . "\n";

    // Create a ticker that ticks every 500ms
    $ticker = Ticker::create(500);

    echo "Starting ticker (500ms interval)...\n";
    echo "Ticker interval: " . $ticker->getInterval() . "ms\n\n";

    // Manually tick 5 times
    for ($i = 0; $i < 5; $i++) {
        $ticker->tick();
        echo sprintf(
            "[%s] Tick #%d\n",
            date('H:i:s.v'),
            $ticker->getTickCount()
        );
    }

    echo "\nTotal ticks: " . $ticker->getTickCount() . "\n";
    $ticker->stop();
    echo "Ticker stopped.\n\n";

    // ========================================================================
    // Example 2: Ticker with Max Ticks (Auto-stop)
    // ========================================================================
    echo "Example 2: Ticker with Max Ticks (Auto-stop)\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(300);
    $ticker->setMaxTicks(3);

    echo "Ticker will auto-stop after 3 ticks\n";
    echo "Max ticks: " . $ticker->getMaxTicks() . "\n\n";

    while ($ticker->isRunning()) {
        $ticker->tick();
        echo sprintf(
            "[%s] Tick #%d (running: %s)\n",
            date('H:i:s.v'),
            $ticker->getTickCount(),
            $ticker->isRunning() ? 'yes' : 'no'
        );
    }

    echo "\nTicker auto-stopped after reaching max ticks\n";
    echo "Is stopped: " . ($ticker->isStopped() ? 'yes' : 'no') . "\n\n";

    // ========================================================================
    // Example 3: Using Ticker Status
    // ========================================================================
    echo "Example 3: Using Ticker Status\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(200);

    echo "Before starting:\n";
    echo "  Is running: " . ($ticker->isRunning() ? 'yes' : 'no') . "\n";
    echo "  Is stopped: " . ($ticker->isStopped() ? 'yes' : 'no') . "\n";
    echo "  Tick count: " . $ticker->getTickCount() . "\n\n";

    // Tick a few times
    for ($i = 0; $i < 3; $i++) {
        $ticker->tick();
    }

    echo "After 3 ticks:\n";
    echo "  Is running: " . ($ticker->isRunning() ? 'yes' : 'no') . "\n";
    echo "  Tick count: " . $ticker->getTickCount() . "\n\n";

    $ticker->stop();

    echo "After stop:\n";
    echo "  Is running: " . ($ticker->isRunning() ? 'yes' : 'no') . "\n";
    echo "  Is stopped: " . ($ticker->isStopped() ? 'yes' : 'no') . "\n";
    echo "  Tick count: " . $ticker->getTickCount() . "\n";
});

echo "\n" . str_repeat("=", 70) . "\n";
echo "All ticker examples completed!\n";
