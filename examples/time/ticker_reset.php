<?php
/**
 * Ticker Reset Example
 *
 * This example demonstrates resetting a ticker and practical use cases.
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Time\Ticker;
use Async\Time;
use Fiber;

echo "Ticker Reset Example\n";
echo str_repeat("=", 70) . "\n\n";

Kernel::run(function () {
    // ========================================================================
    // Example 1: Basic Reset
    // ========================================================================
    echo "Example 1: Basic Reset\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(200);

    echo "Phase 1: Ticking 3 times...\n";
    for ($i = 0; $i < 3; $i++) {
        $ticker->tick();
        echo "  Tick #" . $ticker->getTickCount() . "\n";
    }

    echo "\nCurrent tick count: " . $ticker->getTickCount() . "\n";
    echo "Resetting ticker...\n\n";

    $ticker->reset();

    echo "Phase 2: After reset, ticking 3 more times...\n";
    for ($i = 0; $i < 3; $i++) {
        $ticker->tick();
        echo "  Tick #" . $ticker->getTickCount() . "\n";
    }

    echo "\nFinal tick count: " . $ticker->getTickCount() . "\n";
    $ticker->stop();
    echo "\n";

    // ========================================================================
    // Example 2: Game Round Timer
    // ========================================================================
    echo "Example 2: Game Round Timer\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(500);
    $ticker->setMaxTicks(5); // 5 ticks per round

    for ($round = 1; $round <= 3; $round++) {
        echo "\nRound $round - Start!\n";

        foreach ($ticker as $tick) {
            $timeLeft = 5 - $tick;
            echo sprintf(
                "  [Round %d] Tick %d/5 - Time remaining: %d ticks\n",
                $round,
                $tick,
                $timeLeft
            );
        }

        echo "Round $round - Finished!\n";

        if ($round < 3) {
            echo "Preparing next round...\n";
            Fiber::suspend(Time::sleep(0.5));
            $ticker->reset(); // Reset for next round
        }
    }

    echo "\nGame Over! All rounds completed.\n\n";

    // ========================================================================
    // Example 3: Retry Mechanism with Reset
    // ========================================================================
    echo "Example 3: Retry Mechanism with Reset\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(300);
    $ticker->setMaxTicks(3); // 3 retry attempts

    $maxRetries = 2;
    $success = false;

    for ($attempt = 1; $attempt <= $maxRetries && !$success; $attempt++) {
        echo "\nAttempt $attempt:\n";

        $attemptSuccess = rand(0, 1) === 1; // Simulate random success/failure

        foreach ($ticker as $tick) {
            echo sprintf(
                "  [Attempt %d] Retry %d/3 - Trying to connect...\n",
                $attempt,
                $tick
            );
        }

        if ($attemptSuccess) {
            echo "  Success! Connection established.\n";
            $success = true;
        } else {
            echo "  Failed! All retries exhausted for this attempt.\n";

            if ($attempt < $maxRetries) {
                echo "  Resetting retry counter for next attempt...\n";
                Fiber::suspend(Time::sleep(0.5));
                $ticker->reset();
            }
        }
    }

    if (!$success) {
        echo "\nAll attempts failed!\n";
    }

    echo "\n";

    // ========================================================================
    // Example 4: Periodic Task with Dynamic Reset
    // ========================================================================
    echo "Example 4: Periodic Task with Dynamic Reset\n";
    echo str_repeat("-", 70) . "\n";

    $ticker = Ticker::create(250);
    $ticker->setMaxTicks(4);

    echo "\nSimulating a monitoring system...\n";

    $checks = 0;
    $maxChecks = 10;
    $consecutiveFailures = 0;

    while ($checks < $maxChecks) {
        foreach ($ticker as $tick) {
            $checks++;
            $status = rand(0, 2) > 0; // 66% success rate

            $consecutiveFailures = $status ? 0 : $consecutiveFailures + 1;

            echo sprintf(
                "  Check #%d: %s %s\n",
                $checks,
                $status ? 'OK  ' : 'FAIL',
                $consecutiveFailures > 0 ? "(consecutive failures: $consecutiveFailures)" : ''
            );

            if ($consecutiveFailures >= 3) {
                echo "  ALERT: 3 consecutive failures detected! Resetting monitoring...\n";
                $consecutiveFailures = 0;
                break;
            }

            if ($checks >= $maxChecks) {
                break;
            }
        }

        if ($checks < $maxChecks) {
            $ticker->reset();
        }
    }

    echo "\nMonitoring session completed ($checks checks performed).\n\n";

    // ========================================================================
    // Example 5: Multiple Tickers with Different Intervals
    // ========================================================================
    echo "Example 5: Multiple Tickers with Different Intervals\n";
    echo str_repeat("-", 70) . "\n";

    echo "\nRunning fast and slow tickers concurrently...\n\n";

    $fastTicker = Ticker::create(200);
    $slowTicker = Ticker::create(500);

    $fastTicker->setMaxTicks(3);
    $slowTicker->setMaxTicks(2);

    // Fast ticker task
    Kernel::spawn(function () use ($fastTicker) {
        foreach ($fastTicker as $tick) {
            echo sprintf(
                "[%s] Fast ticker: Tick #%d (200ms interval)\n",
                date('H:i:s.v'),
                $tick
            );
        }
    });

    // Slow ticker task
    Kernel::spawn(function () use ($slowTicker) {
        foreach ($slowTicker as $tick) {
            echo sprintf(
                "[%s] Slow ticker: Tick #%d (500ms interval)\n",
                date('H:i:s.v'),
                $tick
            );
        }
    });

    // Wait for both to complete
    Fiber::suspend(Time::sleep(1.5));

    echo "\nBoth tickers completed!\n";
});

echo "\n" . str_repeat("=", 70) . "\n";
echo "All ticker reset examples completed!\n";
