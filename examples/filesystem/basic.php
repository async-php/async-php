<?php

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;

Kernel::run(function () {
    Kernel::enableCoroutine(Kernel::HOOK_FILE);

    $file = 'test_file.txt';
    $content = "Hello Async File World!\n";

    echo "Writing to file...\n";
    // This uses Async\Stream\FileStreamWrapper
    file_put_contents($file, $content);
    
    echo "Reading from file...\n";
    // This uses Async\Stream\FileStreamWrapper
    $read = file_get_contents($file);
    
    echo "Read Content: " . trim($read) . "\n";
    
    if ($read === $content) {
        echo "SUCCESS: File I/O works.\n";
    } else {
        echo "FAILURE: Content mismatch.\n";
    }
    
    unlink($file);
});

