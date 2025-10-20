<?php

require __DIR__ . '/src-php/bootstrap.php';

use Async\Kernel;
use Async\Http\Server;
use Async\Channel;

// Load extension
if (!extension_loaded('async-php')) {
    $extPath = __DIR__ . '/target/release/libasync_php.dylib';
    if (file_exists($extPath)) dl($extPath);
}

Kernel::run(function () {
    echo "Starting HTTP Server on http://127.0.0.1:8081\n";
    
    // Create a channel for statistics
    $stats = new Channel();
    
    // Stats Aggregator Fiber
    Kernel::spawn(function () use ($stats) {
        $count = 0;
        while (true) {
            $data = $stats->pop();
            $count++;
            if ($count % 10 === 0) {
                echo "[Stats] Handled $count requests.\n";
            }
        }
    });

    $server = new Server('127.0.0.1', 8081);
    
    $server->handle(function ($req) use ($stats) {
        // $req is HttpReq object from Rust
        $stats->push(1);
        
        return "Hello from Async PHP!\nMethod: {$req->method}\nPath: {$req->path}\n";
    });
});
