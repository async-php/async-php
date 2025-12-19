<?php

/**
 * Test script for zero-copy HTTP Server
 *
 * This script starts a simple HTTP server and verifies it works correctly.
 */

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Tcp\Listener;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

function testServer(): void
{
    $addr = '127.0.0.1:9001';
    echo "Starting test HTTP server on http://$addr\n";
    echo "Test URLs:\n";
    echo "  - http://$addr/ (Hello World)\n";
    echo "  - http://$addr/echo?message=test (Echo message)\n";
    echo "  - http://$addr/json (JSON response)\n";
    echo "  - http://$addr/headers (Show headers)\n";
    echo "\nPress Ctrl+C to stop\n\n";

    $server = new Server();
    $listener = Listener::bind($addr);

    $requestCount = 0;

    while (true) {
        $conn = $listener->accept();
        $peerAddr = $conn->remoteAddr();

        go(function() use ($server, $conn, $peerAddr, &$requestCount) {
            try {
                $server->serve($conn, function(HttpRequest $req) use ($peerAddr, &$requestCount): HttpResponse {
                    $requestCount++;
                    $method = $req->method();
                    $path = $req->path();
                    $query = $req->queryString();

                    echo "[Request #$requestCount] [$peerAddr] $method $path" . ($query ? "?$query" : "") . "\n";

                    $resp = new HttpResponse();
                    $resp->setHeader('Server', 'Async-PHP/1.0 Zero-Copy');

                    // Route handling
                    if ($path === '/' || $path === '') {
                        $resp->setStatus(200);
                        $resp->setHeader('Content-Type', 'text/plain');
                        $resp->setBody("Hello from Async PHP Zero-Copy HTTP Server!\n\nRequest #$requestCount\nClient: $peerAddr\n");
                    }
                    elseif ($path === '/echo') {
                        $resp->setStatus(200);
                        $resp->setHeader('Content-Type', 'text/plain');
                        $message = $query ? urldecode(str_replace('message=', '', $query)) : 'No message';
                        $resp->setBody("Echo: $message\n");
                    }
                    elseif ($path === '/json') {
                        $resp->setStatus(200);
                        $resp->setHeader('Content-Type', 'application/json');
                        $data = [
                            'status' => 'ok',
                            'server' => 'Async-PHP Zero-Copy',
                            'request_count' => $requestCount,
                            'client' => $peerAddr,
                            'method' => $method,
                            'path' => $path,
                        ];
                        $resp->setBody(json_encode($data, JSON_PRETTY_PRINT) . "\n");
                    }
                    elseif ($path === '/headers') {
                        $resp->setStatus(200);
                        $resp->setHeader('Content-Type', 'text/plain');
                        $body = "Request Headers:\n";
                        $headers = $req->getHeaders();
                        foreach ($headers as $name => $values) {
                            foreach ($values as $value) {
                                $body .= "  $name: $value\n";
                            }
                        }
                        $resp->setBody($body);
                    }
                    else {
                        $resp->setStatus(404);
                        $resp->setHeader('Content-Type', 'text/plain');
                        $resp->setBody("404 Not Found: $path\n");
                    }

                    return $resp;
                });

                echo "[$peerAddr] Connection closed\n";
            } catch (\Throwable $e) {
                echo "[$peerAddr] Error: {$e->getMessage()}\n";
                echo $e->getTraceAsString() . "\n";
            }
        });
    }
}

// Run the test server
$fiber = new Fiber(function() {
    testServer();
});
run($fiber);
