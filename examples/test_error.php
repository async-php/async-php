<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Kernel\FileSystem;

function test_exception_handling() {
    echo ">> Testing Exception Handling for Async Operations\n";

    // 1. Test Success Case
    Kernel::spawn(function () {
        echo "[1] Creating a test file...\n";
        $file = 'test_ok.txt';
        $content = 'Hello Async World';
        
        \Fiber::suspend(FileSystem::putContents($file, $content)); 
        
        echo "[1] Reading file (should succeed)...\n";
        try {
            $read = \Fiber::suspend(FileSystem::getContents($file));
            echo "[1] SUCCESS: Read content: " . $read . "\n";
        } catch (Throwable $e) {
            echo "[1] FAILED: Unexpected exception: " . $e->getMessage() . "\n";
        }
        
        \Fiber::suspend(FileSystem::unlink($file));
    });

    // 2. Test Failure Case (get_contents on non-existent file)
    Kernel::spawn(function () {
        echo "[2] Reading non-existent file (should throw)...\n";
        try {
            $res = \Fiber::suspend(FileSystem::getContents("non_existent_file_12345.txt"));
            echo "[2] FAILED: Should have thrown exception, but got: " . var_export($res, true) . "\n";
        } catch (Exception $e) {
            echo "[2] SUCCESS: Caught expected exception: " . $e->getMessage() . "\n";
        } catch (Throwable $t) {
            echo "[2] PARTIAL: Caught Throwable but expected Exception: " . get_class($t) . ": " . $t->getMessage() . "\n";
        }
    });
}

// Run the event loop
Kernel::run(function () {
    test_exception_handling();
    // Keep main loop alive to allow spawned fibers to run
    Kernel::sleep(500);
});
