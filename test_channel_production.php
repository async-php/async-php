<?php

use Async\Channel;
use Async\Kernel;
use Async\Time;

require_once 'vendor/autoload.php';

Kernel::run(function () {
    echo "=== Testing Channel Production Features ===\n\n";

    // Test 1: Unbounded Channel (default)
    echo "Test 1: Unbounded Channel (unlimited capacity)\n";
    $unbounded = new Channel();

    // Push many items without blocking
    for ($i = 0; $i < 1000; $i++) {
        $result = $unbounded->push("item-$i");
        if (!$result && $i < 5) {
            echo "Failed to push item $i\n";
        }
    }
    echo "Pushed 1000 items to unbounded channel\n";
    echo "Stats: " . json_encode($unbounded->stat()) . "\n";
    echo "Is full: " . ($unbounded->isFull() ? 'yes' : 'no') . " (should be 'no' for unbounded)\n\n";

    // Test 2: Bounded Channel
    echo "Test 2: Bounded Channel (capacity=5)\n";
    $bounded = new Channel(5);

    // Fill the channel
    for ($i = 0; $i < 5; $i++) {
        $bounded->push("item-$i");
    }

    echo "Filled channel with 5 items\n";
    echo "Stats: " . json_encode($bounded->stat()) . "\n";
    echo "Is full: " . ($bounded->isFull() ? 'yes' : 'no') . "\n";
    echo "Is empty: " . ($bounded->isEmpty() ? 'yes' : 'no') . "\n";
    echo "Length: " . $bounded->length() . "\n\n";

    // Test 3: Push with timeout
    echo "Test 3: Push with timeout (should fail on full channel)\n";
    $start = microtime(true);
    $result = $bounded->push("extra-item", 1.0); // 1 second timeout
    $elapsed = microtime(true) - $start;
    echo "Push result: " . ($result ? 'success' : 'failed') . "\n";
    echo "Elapsed time: " . round($elapsed, 2) . " seconds\n\n";

    // Test 4: Pop items
    echo "Test 4: Pop items from channel\n";
    $item = $bounded->pop();
    echo "Popped: $item\n";
    echo "After pop - Length: " . $bounded->length() . "\n";
    echo "After pop - Is full: " . ($bounded->isFull() ? 'yes' : 'no') . "\n\n";

    // Test 5: Pop with timeout
    echo "Test 5: Pop with timeout from empty channel\n";
    $empty = new Channel(5);
    $start = microtime(true);
    $result = $empty->pop(0.5); // 0.5 second timeout
    $elapsed = microtime(true) - $start;
    echo "Pop result: " . ($result === null ? 'null (timeout)' : $result) . "\n";
    echo "Elapsed time: " . round($elapsed, 2) . " seconds\n\n";

    // Test 6: Close channel
    echo "Test 6: Close channel\n";
    $ch = new Channel(3);
    $ch->push("item-1");
    $ch->push("item-2");
    echo "Before close - Is closed: " . ($ch->isClosed() ? 'yes' : 'no') . "\n";
    $ch->close();
    echo "After close - Is closed: " . ($ch->isClosed() ? 'yes' : 'no') . "\n";

    // Try to push to closed channel
    $result = $ch->push("item-3", 0.1);
    echo "Push to closed channel: " . ($result ? 'success' : 'failed') . "\n\n";

    // Test 7: Producer-Consumer pattern
    echo "Test 7: Producer-Consumer pattern\n";
    $queue = new Channel(10);

    // Producer
    go(function () use ($queue) {
        for ($i = 0; $i < 5; $i++) {
            $queue->push("task-$i");
            echo "Produced: task-$i\n";
            \Fiber::suspend(Time::sleep(100));
        }
        $queue->close();
        echo "Producer finished\n";
    });

    // Consumer
    go(function () use ($queue) {
        while (!$queue->isClosed() || !$queue->isEmpty()) {
            $item = $queue->pop(0.2);
            if ($item !== null) {
                echo "Consumed: $item\n";
            }
        }
        echo "Consumer finished\n";
    });

    // Wait for completion
    \Fiber::suspend(Time::sleep(2000));

    echo "\nAll tests completed!\n";
});
