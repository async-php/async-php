<?php

/**
 * HTTP/3 Server Test
 *
 * This example demonstrates a simple HTTP/3 server using QUIC protocol.
 *
 * Prerequisites:
 * - Self-signed TLS certificate and key (see below for how to generate)
 * - PHP 8.1+ with Fibers support
 *
 * To generate a self-signed certificate for testing:
 *
 *   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
 *
 * Run this example:
 *
 *   php -d extension=target/release/libasync_php.dylib examples/http3_server_test.php
 *
 * Test with an HTTP/3 client:
 *
 *   curl --http3 https://localhost:4433/ --insecure
 *
 * Note: Most browsers support HTTP/3, but you'll need to accept the self-signed certificate.
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Kernel\Network\Quic\QuicListener;
use Async\Kernel\Network\Http\Http3Server;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

// Check if certificate files exist
$certFile = 'cert.pem';
$keyFile = 'key.pem';

if (!file_exists($certFile) || !file_exists($keyFile)) {
    // Try checking in the script directory
    $certFile = __DIR__ . '/cert.pem';
    $keyFile = __DIR__ . '/key.pem';
}

if (!file_exists($certFile) || !file_exists($keyFile)) {
    echo "Error: TLS certificate files not found.\n";
    echo "\n";
    echo "Please generate a self-signed certificate:\n";
    echo "  openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes\n";
    echo "\n";
    exit(1);
}

Kernel::run(function () use ($certFile, $keyFile) {
    // Create QUIC listener (similar to TCP listener)
    $listener = QuicListener::bind('127.0.0.1:4433', $certFile, $keyFile);

    // Create HTTP/3 server
    $server = new Http3Server();

    echo "HTTP/3 server listening on https://127.0.0.1:4433\n";
    echo "Press Ctrl+C to stop\n\n";
    echo "Test with: curl --http3 https://localhost:4433/ --insecure\n\n";

    echo "Test with: curl --http3 https://localhost:4433/ --insecure\n\n";

    $handler = function (HttpRequest $request): HttpResponse {
        $method = $request->method();
        $path = $request->path();
        $query = $request->queryString();

        echo "[$method] $path";
        if ($query) {
            echo "?$query";
        }
        echo "\n";

        $response = new HttpResponse();

        // Simple routing
        if ($path === '/') {
            $response->setStatus(200);
            $response->setHeader('Content-Type', 'text/html; charset=utf-8');
            $response->setBody(<<<HTML
<!DOCTYPE html>
<html>
<head>
    <title>HTTP/3 Server</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .card {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #333; }
        .badge {
            display: inline-block;
            padding: 5px 10px;
            background: #4CAF50;
            color: white;
            border-radius: 3px;
            font-size: 12px;
        }
        code {
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <div class="card">
        <h1>🚀 HTTP/3 Server <span class="badge">QUIC</span></h1>
        <p>Congratulations! You've successfully connected to an HTTP/3 server running on Async-PHP!</p>

        <h2>Features</h2>
        <ul>
            <li>HTTP/3 protocol over QUIC transport</li>
            <li>TLS 1.3 encryption</li>
            <li>Low latency connections</li>
            <li>Connection migration support</li>
            <li>Improved packet loss recovery</li>
        </ul>

        <h2>Try These Endpoints</h2>
        <ul>
            <li><a href="/api/info">/api/info</a> - Server information (JSON)</li>
            <li><a href="/api/echo?message=Hello">/api/echo?message=Hello</a> - Echo service</li>
            <li><a href="/404">/404</a> - Not found page</li>
        </ul>
    </div>
</body>
</html>
HTML);
        } elseif ($path === '/api/info') {
            $response->setStatus(200);
            $response->setHeader('Content-Type', 'application/json');
            $response->setBody(json_encode([
                'server' => 'Async-PHP HTTP/3',
                'protocol' => 'HTTP/3',
                'transport' => 'QUIC',
                'tls' => '1.3',
                'timestamp' => time(),
                'features' => [
                    '0-RTT connection establishment',
                    'Connection migration',
                    'Improved congestion control',
                    'Stream multiplexing',
                ]
            ], JSON_PRETTY_PRINT));
        } elseif ($path === '/api/echo') {
            $params = [];
            if ($query) {
                parse_str($query, $params);
            }

            $message = $params['message'] ?? 'No message provided';

            $response->setStatus(200);
            $response->setHeader('Content-Type', 'application/json');
            $response->setBody(json_encode([
                'echo' => $message,
                'method' => $method,
                'path' => $path,
                'query' => $params
            ], JSON_PRETTY_PRINT));
        } else {
            $response->setStatus(404);
            $response->setHeader('Content-Type', 'application/json');
            $response->setBody(json_encode([
                'error' => 'Not Found',
                'path' => $path
            ]));
        }

        return $response;
    };

    // Accept loop (similar to TCP server pattern)
    while (true) {
        try {
            // accept() returns a RustFuture, we must suspend to await the result
            $future = $listener->accept();
            $conn = \Fiber::suspend($future);
        } catch (\Throwable $e) {
            echo "Accept failed: " . $e->getMessage() . "\n";
            // Prevent infinite loop on error
            \Async\Time::sleep(1);
            continue;
        }

        // Handle each connection concurrently
        go(function () use ($server, $conn, $handler) {
            // serve() also returns a RustFuture
            $future = $server->serve($conn, $handler);
            \Fiber::suspend($future);
        });
    }
});
