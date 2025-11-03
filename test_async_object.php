<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Kernel;
use Async\Network\TcpListener;

echo "=== Test Async Object Creation ===\n";

Kernel::run(function () {
    echo "Creating TcpListener...\n";
    $listener = TcpListener::bind('127.0.0.1:0');
    echo "TcpListener created: " . $listener->localAddr() . "\n";

    echo "Test passed!\n";
});
