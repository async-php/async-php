<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Tcp\Listener;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

echo "=== HTTP Server Test ===\n";
echo "Starting server on 127.0.0.1:9001\n";
echo "Test with: curl http://127.0.0.1:9001/\n";
echo "Press Ctrl+C to stop\n\n";

Kernel::run(function () {
    $server = new Server();
    $listener = Listener::bind('127.0.0.1:9001');

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            $server->serve($conn, function (HttpRequest $request): HttpResponse {
                $method = $request->method();
                $path = $request->path();

                echo "[$method] $path\n";

                $resp = new HttpResponse();
                $resp->setHeader('Content-Type', 'text/html; charset=utf-8');

                $body = match ($path) {
                    '/', '' => '<h1>Hello from async-php!</h1>
                        <ul>
                            <li><a href="/json">JSON response</a></li>
                            <li><a href="/text">Text response</a></li>
                        </ul>',

                    '/json' => json_encode([
                        'message' => 'Hello, World!',
                        'timestamp' => time(),
                        'request' => [
                            'method' => $method,
                            'path' => $path,
                        ],
                    ], JSON_PRETTY_PRINT),

                    '/text' => 'Plain text response',

                    default => '404 Not Found',
                };

                if ($path === '/json') {
                    $resp->setHeader('Content-Type', 'application/json');
                } elseif ($path === '/text') {
                    $resp->setHeader('Content-Type', 'text/plain');
                }

                if (!in_array($path, ['/', '', '/json', '/text'])) {
                    $resp->setStatus(404);
                } else {
                    $resp->setStatus(200);
                }

                $resp->setBody($body);
                return $resp;
            });
        });
    }
});
