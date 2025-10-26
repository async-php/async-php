<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Async\Channel;
use Async\Time;

Kernel::run(function () {
    $port = 8081;
    echo "Starting HTTP Server on http://127.0.0.1:$port\n";
    
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

    // Client Test Fiber
    Kernel::spawn(function () use ($port) {
        Time::sleep(200);
        echo "[Client] Sending Request...\n";
        $response = @file_get_contents("http://127.0.0.1:$port/test");
        echo "[Client] Response: " . substr($response, 0, 50) . "...\n";
        
        if (strpos($response, "Hello from Async PHP") !== false) {
            echo "SUCCESS: HTTP Server Test Passed.\n";
            exit(0);
        } else {
            echo "FAILURE: Unexpected response.\n";
            exit(1);
        }
    });

    // Server::listen blocks the fiber.
    Server::listen("127.0.0.1:$port", function (Request $req) use ($stats) {
        $stats->push(1);
        
        $body = "Hello from Async PHP (Hyper)!\n";
        $body .= "Method: " . $req->getMethod() . "\n";
        $body .= "Path: " . $req->getUri() . "\n";
        
        $res = new Response(200, $body);
        $res->withHeader("Content-Type", "text/plain");
        
        return $res;
    });
});
