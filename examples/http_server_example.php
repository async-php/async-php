<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;

// Example 1: Simple HTTP server
Server::create('127.0.0.1:8080', function (Request $request): Response {
    $method = $request->getMethod();
    $uri = $request->getUri();
    $version = $request->getVersion();

    return Response::json([
        'message' => 'Hello from async-php!',
        'request' => [
            'method' => $method,
            'uri' => $uri,
            'version' => $version,
            'secure' => $request->isSecure(),
        ],
    ]);
});

// Example 2: HTTPS server with explicit configuration
$server = new Server();

// Load TLS certificates from files
$server->setTlsFromFiles(
    __DIR__ . '/certs/server.crt',
    __DIR__ . '/certs/server.key'
);

// Or set TLS from PEM strings directly
// $certPem = file_get_contents(__DIR__ . '/certs/server.crt');
// $keyPem = file_get_contents(__DIR__ . '/certs/server.key');
// $server->setTls($certPem, $keyPem);

// Enable HTTP/2 and HTTP/3
$server->setEnableHttp1(true)
    ->setEnableHttp2(true)
    ->setEnableHttp3(true);

$server->listen('127.0.0.1:8443', function (Request $request): Response {
    $path = parse_url($request->getUri(), PHP_URL_PATH);

    return match ($path) {
        '/' => Response::html('<h1>Welcome to async-php HTTPS server!</h1>'),
        '/json' => Response::json(['status' => 'ok', 'protocol' => $request->getVersion()]),
        '/text' => Response::text('Plain text response'),
        '/redirect' => Response::redirect('https://github.com/'),
        default => Response::json(['error' => 'Not found'], 404),
    };
});
