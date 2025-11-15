<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Kernel\Channel;

echo "=== Available methods in Async\\Kernel\\Channel ===\n\n";

$reflection = new ReflectionClass('Async\\Kernel\\Channel');
$methods = $reflection->getMethods(ReflectionMethod::IS_PUBLIC);

foreach ($methods as $method) {
    echo "- " . $method->getName() . "\n";
}

echo "\n=== Test direct calls ===\n\n";

$chan = new Channel(5);

echo "Testing stat():\n";
print_r($chan->stat());

echo "\nAttempting to call methods directly:\n";

try {
    echo "isEmpty(): ";
    var_dump($chan->isEmpty());
} catch (Error $e) {
    echo "ERROR: " . $e->getMessage() . "\n";
}

try {
    echo "is_empty(): ";
    var_dump($chan->is_empty());
} catch (Error $e) {
    echo "ERROR: " . $e->getMessage() . "\n";
}

try {
    echo "isFull(): ";
    var_dump($chan->isFull());
} catch (Error $e) {
    echo "ERROR: " . $e->getMessage() . "\n";
}

try {
    echo "is_full(): ";
    var_dump($chan->is_full());
} catch (Error $e) {
    echo "ERROR: " . $e->getMessage() . "\n";
}
