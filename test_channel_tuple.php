<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Channel;
use Async\Kernel;

echo "=== Testing Go-Style Channel Tuple API ===\n\n";

Kernel::run(function () {
    echo "Test 1: Basic push/pop with tuple returns\n";
    $chan = new Channel(1);

    // Push
    [, $ok] = $chan->push("Hello World");
    echo "Push result: " . ($ok ? 'success' : 'failed') . "\n";

    // Pop
    [$value, $ok] = $chan->pop();
    if ($ok) {
        echo "Pop result: '$value'\n";
    } else {
        echo "Pop failed\n";
    }
    echo "\n";

    echo "Test 2: Pop from empty channel with timeout\n";
    $chan2 = new Channel(0);
    [$value, $ok] = $chan2->pop(0.1);
    echo "Pop with timeout (empty): " . ($ok ? 'success' : 'timeout') . "\n";
    echo "Value: " . ($value === null ? 'null' : $value) . "\n\n";

    echo "Test 3: Push to full channel with timeout\n";
    $chan3 = new Channel(1);
    [, $ok] = $chan3->push("first");
    echo "First push: " . ($ok ? 'success' : 'failed') . "\n";
    [, $ok] = $chan3->push("second", 0.1);
    echo "Second push (timeout): " . ($ok ? 'success' : 'timeout') . "\n\n";

    echo "Test 4: Close channel and try operations\n";
    $chan4 = new Channel(1);
    $chan4->close();

    [, $ok] = $chan4->push("test");
    echo "Push to closed channel: " . ($ok ? 'success' : 'failed') . "\n";

    [$value, $ok] = $chan4->pop();
    echo "Pop from closed channel: " . ($ok ? 'success' : 'failed') . "\n\n";

    echo "Test 5: Go-style usage pattern\n";
    $chan5 = new Channel(3);

    // Producer
    Kernel::spawn(function() use ($chan5) {
        foreach (['apple', 'banana', 'cherry'] as $fruit) {
            [, $ok] = $chan5->push($fruit);
            if ($ok) {
                echo "Sent: $fruit\n";
            }
        }
        $chan5->close();
    });

    // Consumer
    while (true) {
        [$value, $ok] = $chan5->pop();
        if (!$ok) {
            echo "Channel closed, stopping consumer\n";
            break;
        }
        echo "Received: $value\n";
    }
    echo "\n";

    echo "Test 6: Error handling without exceptions\n";
    $chan6 = new Channel(1);
    [, $ok] = $chan6->push("data");

    if ($ok) {
        [$value, $ok] = $chan6->pop();
        if ($ok) {
            echo "Successfully got: $value\n";
        } else {
            echo "Failed to pop (but no exception thrown!)\n";
        }
    } else {
        echo "Failed to push (but no exception thrown!)\n";
    }
});

echo "\n=== All tests completed! ===\n";
echo "No exceptions, clean error handling with tuple returns!\n";
