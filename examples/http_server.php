<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Http\Server;
use Async\Channel;

// Load extension
if (!extension_loaded('async-php')) {
    $extPath = __DIR__ . '/../target/debug/libasync_php.dylib';
    if (file_exists($extPath)) {
        dl($extPath);
    }
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
        /** @var \Async\Http\Request $req */
        $stats->push(1);
        
        $body = "Hello from Async PHP (Hyper)!\n";
        $body .= "Method: {$req->method}\n";
        $body .= "Path: {$req->uri}\n";
        
        $res = new \Async\Http\Response(200, $body);
        $res->withHeader("Content-Type", "text/plain");
        
        return $res;
    });
});
