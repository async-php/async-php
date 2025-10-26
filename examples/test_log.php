<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Http\Response;
use Async\Time;

// Setup Logger
Kernel::setupLog(['level' => 'info', 'ansi' => 'true']);

Kernel::run(function () {
    echo "Main Fiber Started. Logger configured to INFO.\n";
    
    $port = 8082;
    $addr = "127.0.0.1:$port";
    
    // Spawn server in a background fiber
    Kernel::spawn(function() use ($addr) {
        Server::listen($addr, function ($req) {
            return new Response(200, "ok");
        });
    });

    // Main fiber acts as client
    Time::sleep(500); // Wait for server to start
    echo "Sending request to trigger logs...\n";
    $ctx = stream_context_create(['http' => ['timeout' => 1]]);
    @file_get_contents("http://$addr", false, $ctx);
    echo "Request sent.\n";
    exit(0);
});