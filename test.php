<?php

// run_on_tokio acts as the entry point for our async runtime.
// It takes a Fiber, runs it, and allows it to spawn concurrent fibers.

if (!function_exists('run_on_tokio')) {
    die("Extension 'async-php' not loaded.\n");
}

// Main Fiber
$main = new Fiber(function () {
    echo "[Main] Started.\n";

    // Spawn a child fiber
    echo "[Main] Spawning child...\n";
    
    RustFuture::spawn(function () {
        echo "  [Child] Started. Sleeping 500ms...\n";
        // Child does a long sleep
        Fiber::suspend(RustFuture::nativeSleep(500));
        echo "  [Child] Woke up!\n";
        
        echo "  [Child] Fetching FFI...\n";
        $res = Fiber::suspend(RustFuture::ffiFetchData(99));
        echo "  [Child] FFI Result: $res\n";
        
        echo "  [Child] Done.\n";
    });
    
    echo "[Main] Child spawned. Sleeping 200ms...\n";
    
    // Main continues doing something else concurrently
    for ($i = 0; $i < 3; $i++) {
        Fiber::suspend(RustFuture::nativeSleep(200));
        echo "[Main] Tick $i (200ms interval)\n";
    }
    
    echo "[Main] Done.\n";
});

echo "Starting Event Loop...\n";
$start = microtime(true);

run_on_tokio($main);

$duration = microtime(true) - $start;
echo "Loop Finished in " . round($duration, 2) . "s\n";