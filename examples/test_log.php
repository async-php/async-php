<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Http\Server;
use Async\Time;

// Setup Logger
// Level 'debug' should show detailed logs if any libraries use debug.
// 'info' will show our "Listening on..." message.
Kernel::setupLog(['level' => 'info', 'ansi' => 'true']);

Kernel::run(function () {
    echo "Main Fiber Started. Logger configured to INFO.\n";
    
    $server = new Server('127.0.0.1', 8082);
    
    // Spawn server in a background fiber
    Kernel::spawn(function() use ($server) {
        $server->handle(function ($req) {
            return new \Async\HttpResponse(200, "ok");
        });
    });

    // Main fiber acts as client
    Time::sleep(500);
    echo "Sending request to trigger logs...\n";
    $ctx = stream_context_create(['http' => ['timeout' => 1]]);
    @file_get_contents("http://127.0.0.1:8082", false, $ctx);
    echo "Request sent.\n";
    exit(0);
});

