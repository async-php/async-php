<?php

require_once __DIR__ . '/../../vendor/autoload.php';

// Main Fiber
run(function () {
    echo "[Main] Started.\n";

    // Spawn a child fiber using go()
    echo "[Main] Spawning child...\n";
    
    go(function () {
        echo "  [Child] Started. Sleeping 500ms...\n";
        // Child does a long sleep
        Fiber::suspend(\Async\Time::sleep(0.5)); // 500ms = 0.5s
        echo "  [Child] Woke up!\n";
        
        echo "  [Child] Done.\n";
    });
    
    // Demonstrate AsyncTime::after
    go(function() {
        echo "    [After] Started. Will fire after 300ms...\n";
        Fiber::suspend(\Async\Time::sleep(0.3)); // 300ms = 0.3s
        echo "    [After] Fired after 300ms!\n";
    });

    // Demonstrate AsyncTicker (Commented out due to issue)
    /*
    spawn_task(function() {
        echo "      [Ticker] Started. Will tick every 100ms for 3 times...\n";
        $ticker = \Async\Time::createTicker(100);
        for ($i = 0; $i < 3; $i++) {
            Fiber::suspend($ticker->next_tick());
            echo "      [Ticker] Tick " . ($i + 1) . "\n";
        }
        $ticker->stop();
        echo "      [Ticker] Stopped after 3 ticks.\n";
    });
    */

    echo "[Main] Child spawned. Sleeping 200ms...\n";
    
    // Main continues doing something else concurrently
    for ($i = 0; $i < 4; $i++) {
        Fiber::suspend(\Async\Time::sleep(0.2)); // 200ms = 0.2s
        echo "[Main] Tick $i (200ms interval)\n";
    }
    
    echo "[Main] Done.\n";
});

$duration = microtime(true) - $start;
echo "Loop Finished in " . round($duration, 2) . "s\n";