<?php

require __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    echo "Start\n";
    
    $start = microtime(true);
    
    Time::after(0.5, function () use ($start) {
        $diff = microtime(true) - $start;
        echo "Callback executed after " . number_format($diff, 4) . "s\n";
    });
    
    echo "After calling Time::after (should be immediate)\n";
    
    // Keep the main fiber alive long enough for the timer to fire
    Time::sleep(1000);
    
    echo "End\n";
});

