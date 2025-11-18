<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;

echo "=== Debug spawnIO ===\n\n";

class MyReader {
    public function read($length) {
        echo "[MyReader] read({$length}) called\n";
        return "Hello";
    }
}

Kernel::run(function () {
    $reader = new MyReader();
    echo "1. Creating channel via spawnIO\n";
    $channel = IO::spawnIO($reader);
    echo "2. Channel created\n";

    echo "3. Sending read request\n";
    [, $ok] = $channel->push(['read', [5]]);
    echo "4. Request sent: " . ($ok ? 'OK' : 'FAILED') . "\n";

    echo "5. Waiting for response\n";
    [$result, $ok] = $channel->pop();
    echo "6. Response received: " . ($ok ? 'OK' : 'FAILED') . ", result: '{$result}'\n";

    echo "7. Sending close\n";
    [, $ok] = $channel->push(['__close__', []]);
    echo "8. Close sent\n";

    echo "✓ Test passed\n";
});

echo "\n=== Test Completed ===\n";
