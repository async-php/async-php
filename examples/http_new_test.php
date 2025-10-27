<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    $addr = "127.0.0.1:9000";
    echo "Starting server at $addr\n";

    Server::listen($addr, function (Request $req) {
        echo "Received request: " . $req->getMethod() . " " . $req->getUri() . "\n";
        
        $response = $req->getResponse();
        $response->withStatus(200)
                 ->withHeader("Content-Type", "text/plain");
                 
        // Test Streaming Response
        $response->initStream();
        
        // Write in background (async)
        Kernel::spawn(function () use ($response) {
            for ($i = 0; $i < 5; $i++) {
                $response->write("Chunk $i\n");
                Time::sleep(100);
            }
            $response->end();
        });
        
        return $response;
    }, ['enable_http3' => 'true']);
});