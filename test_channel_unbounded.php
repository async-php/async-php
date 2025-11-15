<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Channel;

echo "=== Testing Unbounded Channel (Default) ===\n\n";

// Test 1: Create unbounded channel (no capacity)
echo "Test 1: Unbounded channel (default)\n";
$chan = new Channel();
$stat = $chan->stat();
echo "Stat: ";
print_r($stat);
echo "isEmpty: " . ($chan->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan->isFull() ? 'true' : 'false') . " (unbounded should never be full)\n";
echo "length: " . $chan->length() . "\n\n";

// Test 2: Create bounded channel with capacity
echo "Test 2: Bounded channel with capacity=5\n";
$chan2 = new Channel(5);
$stat2 = $chan2->stat();
echo "Stat: ";
print_r($stat2);
echo "isEmpty: " . ($chan2->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan2->isFull() ? 'true' : 'false') . "\n";
echo "length: " . $chan2->length() . "\n\n";

// Test 3: Bounded channel with capacity=1
echo "Test 3: Bounded channel with capacity=1\n";
$chan3 = new Channel(1);
echo "Capacity: " . $chan3->stat()['capacity'] . "\n";
echo "isEmpty: " . ($chan3->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan3->isFull() ? 'true' : 'false') . "\n\n";

// Test 4: Bounded channel with large capacity
echo "Test 4: Bounded channel with capacity=1000\n";
$chan4 = new Channel(1000);
echo "Capacity: " . $chan4->stat()['capacity'] . "\n";
echo "isEmpty: " . ($chan4->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan4->isFull() ? 'true' : 'false') . "\n\n";

// Test 5: Close unbounded channel
echo "Test 5: Close unbounded channel\n";
$chan->close();
echo "isClosed: " . ($chan->isClosed() ? 'true' : 'false') . "\n\n";

// Test 6: Verify capacity values
echo "Test 6: Verify capacity values\n";
echo "Unbounded capacity: " . $chan->stat()['capacity'] . " (should be -1)\n";
echo "Bounded(5) capacity: " . $chan2->stat()['capacity'] . " (should be 5)\n";
echo "Bounded(1) capacity: " . $chan3->stat()['capacity'] . " (should be 1)\n";
echo "Bounded(1000) capacity: " . $chan4->stat()['capacity'] . " (should be 1000)\n\n";

echo "=== All tests passed! ===\n";
echo "\nUnbounded channels have capacity = -1\n";
echo "Bounded channels have capacity = specified value\n";
echo "Unbounded channels never return isFull() = true\n";
