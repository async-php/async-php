<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Async\Network\Tcp\Listener;

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
            $server->serve($conn, function (Request $request): Response {
                $method = $request->method();
                $path = $request->path();

                echo "[$method] $path\n";

                $resp = new Response();

                $body = match ($path) {
                    '/', '' => '<h1>Hello from async-php!</h1>
                        <ul>
                            <li><a href="/json">JSON response</a></li>
                            <li><a href="/text">Text response</a></li>
                        </ul>',

                    '/json' => [
                        'message' => 'Hello, World!',
                        'timestamp' => time(),
                        'request' => [
                            'method' => $method,
                            'path' => $path,
                        ],
                    ],

                    '/text' => 'Plain text response',

                    default => '404 Not Found',
                };

                if ($path === '/json') {
                    $resp->setStatus(200)->setJson($body);
                } elseif ($path === '/text') {
                    $resp->setStatus(200)->setText($body);
                } elseif (in_array($path, ['/', ''])) {
                    $resp->setStatus(200)->setHtml($body);
                } else {
                    $resp->setStatus(404)->setText($body);
                }

                return $resp;
            });
        });
    }
});
