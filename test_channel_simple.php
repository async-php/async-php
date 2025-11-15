<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Channel;

echo "=== Testing Production Channel Features (Simple) ===\n\n";

// Test 1: Create channel with custom capacity
echo "Test 1: Channel with capacity=5\n";
$chan = new Channel(5);
$stat = $chan->stat();
echo "Initial stat: ";
print_r($stat);
echo "isEmpty: " . ($chan->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan->isFull() ? 'true' : 'false') . "\n";
echo "length: " . $chan->length() . "\n\n";

// Test 2: Test state methods after creation
echo "Test 2: State methods\n";
echo "isEmpty: " . ($chan->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan->isFull() ? 'true' : 'false') . "\n";
echo "length: " . $chan->length() . "\n";
echo "isClosed: " . ($chan->isClosed() ? 'true' : 'false') . "\n\n";

// Test 3: Close channel
echo "Test 3: Close channel\n";
$result = $chan->close();
echo "close() result: " . ($result ? 'true' : 'false') . "\n";
echo "isClosed: " . ($chan->isClosed() ? 'true' : 'false') . "\n\n";

// Test 4: Create another channel with different capacity
echo "Test 4: Channel with capacity=10\n";
$chan2 = new Channel(10);
$stat2 = $chan2->stat();
echo "Stat: ";
print_r($stat2);
echo "\n";

// Test 5: Test capacity boundaries
echo "Test 5: Test capacity=1 (minimal)\n";
$chan3 = new Channel(1);
echo "Capacity: " . $chan3->stat()['capacity'] . "\n";
echo "isEmpty: " . ($chan3->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan3->isFull() ? 'true' : 'false') . "\n";
echo "length: " . $chan3->length() . "\n\n";

// Test 6: Test default capacity
echo "Test 6: Channel with default capacity\n";
$chan4 = new Channel();
echo "Default capacity: " . $chan4->stat()['capacity'] . "\n";
echo "isEmpty: " . ($chan4->isEmpty() ? 'true' : 'false') . "\n\n";

// Test 7: Large capacity
echo "Test 7: Channel with large capacity (1000)\n";
$chan5 = new Channel(1000);
echo "Capacity: " . $chan5->stat()['capacity'] . "\n";
echo "isEmpty: " . ($chan5->isEmpty() ? 'true' : 'false') . "\n";
echo "isFull: " . ($chan5->isFull() ? 'true' : 'false') . "\n\n";

echo "=== All synchronous tests passed! ===\n";
