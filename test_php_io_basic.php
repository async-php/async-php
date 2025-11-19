<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;

echo "=== Testing PHP IO Wrappers - Basic ===\n\n";

// Test 1: PhpReader creation
echo "Test 1: Creating PhpReader\n";
class MyReader {
    public function read($length) {
        return "Hello";
    }
}

Kernel::run(function () {
    $reader = new MyReader();
    echo "Creating PhpReader...\n";
    $phpReader = IO::wrapPhpReader($reader);
    echo "PhpReader created: " . get_class($phpReader) . "\n";

    echo "Getting AsyncReader...\n";
    $asyncReader = $phpReader->asReader();
    echo "AsyncReader created: " . get_class($asyncReader) . "\n";

    echo "Reading data...\n";
    $data = \Fiber::suspend($asyncReader->read(5));
    echo "Read: '{$data}'\n";

    echo "✓ Test 1 passed\n\n";
});

echo "=== Test Completed ===\n";
