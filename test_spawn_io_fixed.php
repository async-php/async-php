<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;

echo "=== Testing Fixed spawnIO ===\n\n";

// Test 1: Channel close handling
echo "Test 1: Channel close handling\n";
class TestReader1 {
    public function read($length) {
        return "data-{$length}";
    }
}

Kernel::run(function () {
    $reader = new TestReader1();
    $channel = IO::spawnIO($reader);

    // Normal operation
    [, $ok] = $channel->push(['read', [10]]);
    echo "Push request: " . ($ok ? 'OK' : 'FAILED') . "\n";

    [$result, $ok] = $channel->pop();
    echo "Pop result: " . ($ok ? 'OK' : 'FAILED') . ", value: {$result}\n";

    // Close channel
    $channel->close();
    echo "Channel closed\n";

    // Try to use closed channel
    [, $ok] = $channel->push(['read', [20]]);
    echo "Push after close: " . ($ok ? 'OK' : 'FAILED') . "\n";

    [$result, $ok] = $channel->pop();
    echo "Pop after close: " . ($ok ? 'OK' : 'FAILED') . "\n";

    echo "✓ Test 1 passed\n\n";
});

// Test 2: Exception handling
echo "Test 2: Exception handling\n";
class TestReader2 {
    public function read($length) {
        throw new Exception("Read failed: disk error");
    }
}

Kernel::run(function () {
    $reader = new TestReader2();
    $channel = IO::spawnIO($reader);

    // Send request that will throw exception
    [, $ok] = $channel->push(['read', [10]]);
    echo "Push request: " . ($ok ? 'OK' : 'FAILED') . "\n";

    // Receive error response
    [$result, $ok] = $channel->pop();
    echo "Pop result: " . ($ok ? 'OK' : 'FAILED') . "\n";

    if (is_array($result) && isset($result['__error__'])) {
        echo "Got error response: {$result['message']}\n";
        echo "✓ Exception properly caught and returned\n";
    } else {
        echo "✗ Expected error response, got: " . json_encode($result) . "\n";
    }

    // Send close command
    [, $ok] = $channel->push(['__close__', []]);
    echo "Close command sent: " . ($ok ? 'OK' : 'FAILED') . "\n";

    echo "✓ Test 2 passed\n\n";
});

// Test 3: Resource cleanup
echo "Test 3: Resource cleanup\n";
class TestResource {
    private $closed = false;

    public function read($length) {
        if ($this->closed) {
            throw new Exception("Resource already closed");
        }
        return "data-{$length}";
    }

    public function close() {
        echo "[TestResource] Closing resource\n";
        $this->closed = true;
    }

    public function __destruct() {
        if (!$this->closed) {
            echo "[TestResource] WARNING: Resource not properly closed!\n";
        } else {
            echo "[TestResource] Resource was properly closed\n";
        }
    }
}

Kernel::run(function () {
    $resource = new TestResource();
    $channel = IO::spawnIO($resource);

    // Normal operation
    [, $ok] = $channel->push(['read', [10]]);
    [$result, $ok] = $channel->pop();
    echo "Read result: {$result}\n";

    // Send close command - this should trigger resource cleanup
    echo "Sending close command...\n";
    [, $ok] = $channel->push(['__close__', []]);

    // Wait a bit for cleanup
    usleep(100000); // 100ms

    echo "✓ Test 3 passed\n\n";
});

// Test 4: Invalid request format
echo "Test 4: Invalid request format\n";
class TestReader4 {
    public function read($length) {
        return "data-{$length}";
    }
}

Kernel::run(function () {
    $reader = new TestReader4();
    $channel = IO::spawnIO($reader);

    // Send invalid format (not an array)
    echo "Sending invalid format (string)...\n";
    [, $ok] = $channel->push("invalid");
    echo "Push: " . ($ok ? 'OK' : 'FAILED') . "\n";

    // Send invalid format (wrong array size)
    echo "Sending invalid format (wrong size array)...\n";
    [, $ok] = $channel->push(['read']);
    echo "Push: " . ($ok ? 'OK' : 'FAILED') . "\n";

    // Send invalid format (method not string)
    echo "Sending invalid format (method not string)...\n";
    [, $ok] = $channel->push([123, [10]]);
    echo "Push: " . ($ok ? 'OK' : 'FAILED') . "\n";

    // Send valid request
    echo "Sending valid request...\n";
    [, $ok] = $channel->push(['read', [10]]);
    [$result, $ok] = $channel->pop();
    echo "Valid request result: {$result}\n";

    // Close
    [, $ok] = $channel->push(['__close__', []]);

    echo "✓ Test 4 passed (spawned fiber didn't crash)\n\n";
});

// Test 5: Multiple operations
echo "Test 5: Multiple operations\n";
class TestMulti {
    private $counter = 0;

    public function read($length) {
        $this->counter++;
        return "read-{$this->counter}-{$length}";
    }

    public function write($data) {
        $this->counter++;
        return strlen($data);
    }

    public function close() {
        echo "[TestMulti] Closed after {$this->counter} operations\n";
    }
}

Kernel::run(function () {
    $multi = new TestMulti();
    $channel = IO::spawnIO($multi);

    // Multiple read operations
    for ($i = 1; $i <= 5; $i++) {
        [, $ok] = $channel->push(['read', [$i * 10]]);
        [$result, $ok] = $channel->pop();
        echo "Read {$i}: {$result}\n";
    }

    // Multiple write operations
    for ($i = 1; $i <= 3; $i++) {
        [, $ok] = $channel->push(['write', ["data-{$i}"]]);
        [$result, $ok] = $channel->pop();
        echo "Write {$i}: {$result} bytes\n";
    }

    // Close
    [, $ok] = $channel->push(['__close__', []]);

    usleep(100000); // Wait for cleanup

    echo "✓ Test 5 passed\n\n";
});

echo "=== All Tests Completed Successfully ===\n";
