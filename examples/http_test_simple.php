<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;

echo "=== HTTP Server Test ===\n";
echo "Starting server on 127.0.0.1:9001\n";
echo "Test with: curl http://127.0.0.1:9001/\n";
echo "Press Ctrl+C to stop\n\n";

Kernel::run(function () {
    Server::create('127.0.0.1:9001', function (Request $request): Response {
        $method = $request->getMethod();
        $uri = $request->getUri();
        $version = $request->getVersion();

        echo "[{$method}] {$uri} (HTTP/{$version})\n";

        $path = parse_url($uri, PHP_URL_PATH);

        return match ($path) {
            '/' => Response::html('<h1>Hello from async-php!</h1>
                <p>HTTP Version: ' . htmlspecialchars($version) . '</p>
                <ul>
                    <li><a href="/json">JSON response</a></li>
                    <li><a href="/text">Text response</a></li>
                    <li><a href="/redirect">Redirect test</a></li>
                </ul>'),

            '/json' => Response::json([
                'message' => 'Hello, World!',
                'timestamp' => time(),
                'request' => [
                    'method' => $method,
                    'uri' => $uri,
                    'version' => $version,
                ],
            ]),

            '/text' => Response::text("Plain text response\nHTTP/{$version}"),

            '/redirect' => Response::redirect('/'),

            default => Response::json(['error' => 'Not Found'], 404),
        };
    }, [
        'http1' => true,
        'http2' => false,
        'http3' => false,  // Disable HTTP/3 (requires TLS)
    ]);
});
