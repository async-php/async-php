<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Channel;
use Async\Kernel;

echo "=== Testing Go-Style Unbuffered Channel (capacity=0) ===\n\n";

// Test 1: Create unbuffered channel (capacity=0)
echo "Test 1: Create unbuffered channel\n";
$chan = new Channel(0);
$stat = $chan->stat();
echo "Capacity: " . $stat['capacity'] . "\n";
echo "isEmpty: " . ($chan->isEmpty() ? 'true' : 'false') . "\n";
echo "length: " . $chan->length() . "\n\n";

Kernel::run(function () {
    // Test 2: Send blocks until receiver is ready (Go-style behavior)
    echo "Test 2: Test blocking send/recv behavior\n";
    $chan2 = new Channel(-1);

    // Spawn sender (will block until receiver is ready)
    Kernel::spawn(function() use ($chan2) {
        echo "Sender: Starting...\n";
        $result = $chan2->push("Hello from sender");
        echo "Sender: Sent message, result=" . ($result ? 'true' : 'false') . "\n";
        return "sender_done";
    });

    // Small delay to ensure sender starts first
    usleep(100000); // 100ms

    // Now receive (will unblock the sender)
    echo "Receiver: Starting to receive...\n";
    $msg = $chan2->pop();
    echo "Receiver: Got message: " . $msg . "\n\n";

    // Test 3: Multiple send/recv pairs
    echo "Test 3: Multiple blocking send/recv pairs\n";
    $chan3 = new Channel(0);

    $senders = [];
    for ($i = 1; $i <= 3; $i++) {
        Kernel::spawn(function() use ($chan3, $i) {
            $result = $chan3->push("Message $i");
            echo "Sender $i: sent\n";
            return $result;
        });
    }

    // Receive all messages
    for ($i = 1; $i <= 3; $i++) {
        $msg = $chan3->pop();
        echo "Receiver: got '$msg'\n";
    }
    echo "\n";

    // Test 4: Timeout on unbuffered channel
    echo "Test 4: Timeout on unbuffered channel\n";
    $chan4 = new Channel(0);

    // Try to receive with timeout (should timeout since no sender)
    $result = $chan4->pop(0.5);
    echo "Pop with timeout (no sender): " . ($result === null ? 'null (timeout)' : $result) . "\n";

    // Try to send with timeout (should timeout since no receiver)
    $result = $chan4->push("test", 0.5);
    echo "Push with timeout (no receiver): " . ($result ? 'true' : 'false (timeout)') . "\n\n";

    // Test 5: Compare with buffered channel
    echo "Test 5: Compare buffered (cap=1) vs unbuffered (cap=0)\n";

    // Buffered channel - send doesn't block immediately
    $buffered = new Channel(1);
    echo "Buffered channel (cap=1):\n";
    $result = $buffered->push("msg1", 0.1);
    echo "  First push (no receiver): " . ($result ? 'success' : 'timeout') . "\n";
    $result = $buffered->push("msg2", 0.1);
    echo "  Second push (buffer full): " . ($result ? 'success' : 'timeout') . "\n";

    // Unbuffered channel - send blocks immediately without receiver
    $unbuffered = new Channel(0);
    echo "Unbuffered channel (cap=0):\n";
    $result = $unbuffered->push("msg", 0.1);
    echo "  First push (no receiver): " . ($result ? 'success' : 'timeout') . "\n\n";

    echo "=== All tests completed! ===\n";
});
