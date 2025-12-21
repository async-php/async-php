<?php
/**
 * HTTP/3 Server Example
 *
 * This example demonstrates how to create an HTTP/3 server using QUIC.
 *
 * HTTP/3 uses QUIC as the transport protocol, which provides:
 * - Built-in TLS 1.3 encryption
 * - Multiplexed streams
 * - 0-RTT connection resumption
 * - Better performance on lossy networks
 *
 * Run this example:
 * php -d extension=target/release/libasync_php.dylib examples/http3/server.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Async\Network\Quic\Listener;

echo "HTTP/3 Server Example\n";
echo str_repeat("=", 50) . "\n\n";

// Check if certificates exist
$certPath = __DIR__ . '/certs/server.crt';
$keyPath = __DIR__ . '/certs/server.key';

if (!file_exists($certPath) || !file_exists($keyPath)) {
    echo "Error: Certificates not found!\n";
    echo "Please run: ./examples/http3/generate-certs.sh\n\n";
    exit(1);
}

echo "Starting HTTP/3 server on 127.0.0.1:4433...\n";
echo "Certificate: $certPath\n";
echo "Private Key: $keyPath\n";
echo str_repeat("-", 50) . "\n\n";

// Create HTTP Server instance
$server = new Server();

// TLS configuration for QUIC
$tlsConfig = [
    'cert_path' => $certPath,
    'key_path' => $keyPath,
    'alpn' => ['h3', 'h3-29'] // HTTP/3 ALPN protocols
];

try {
    echo "Server is ready and listening...\n";
    echo "Try connecting with: curl --http3 -k https://localhost:4433/\n\n";

    // Bind QUIC listener
    $listener = Listener::bind('127.0.0.1:4433', $tlsConfig);

    // Accept connections loop
    while (true) {
        $conn = $listener->accept();
        
        // Handle connection in a fiber
        go(function() use ($server, $conn) {
            $server->serve($conn, function(Request $req): Response {
                $method = $req->method();
                $path = $req->path();
                $query = $req->queryString();

                echo "[HTTP/3] $method $path" . ($query ? "?$query" : "") . "\n";

                $resp = new Response();
                $resp->withStatus(200);
                $resp->withHeader('Content-Type', 'text/html; charset=utf-8');
                $resp->withHeader('Server', 'Async-PHP HTTP/3');

                // Route handling
                switch ($path) {
                    case '/':
                        $body = <<<HTML
<!DOCTYPE html>
<html>
<head>
    <title>HTTP/3 Server - Async PHP</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .info { background: #f0f0f0; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .success { color: #28a745; font-weight: bold; }
    </style>
</head>
<body>
    <h1>🚀 HTTP/3 Server Running!</h1>
    <div class="info">
        <p class="success">✓ Connection successful via HTTP/3</p>
        <p><strong>Protocol:</strong> HTTP/3 over QUIC</p>
        <p><strong>Server:</strong> Async-PHP</p>
        <p><strong>Method:</strong> $method</p>
        <p><strong>Path:</strong> $path</p>
    </div>
    <h2>Test Endpoints:</h2>
    <ul>
        <li><a href="/hello">GET /hello</a> - Hello World</li>
        <li><a href="/json">GET /json</a> - JSON Response</li>
        <li><a href="/info">GET /info</a> - Server Info</li>
    </ul>
</body>
</html>
HTML;
                        $resp->withBody($body);
                        break;

                    case '/hello':
                        $resp->withBody("Hello from HTTP/3! 🚀\n");
                        $resp->withHeader('Content-Type', 'text/plain');
                        break;

                    case '/json':
                        $data = [
                            'protocol' => 'HTTP/3',
                            'transport' => 'QUIC',
                            'server' => 'Async-PHP',
                            'timestamp' => time(),
                            'features' => [
                                'multiplexing' => true,
                                'zero_rtt' => true,
                                'tls_1_3' => true,
                                'connection_migration' => true
                            ]
                        ];
                        $resp->withBody(json_encode($data, JSON_PRETTY_PRINT));
                        $resp->withHeader('Content-Type', 'application/json');
                        break;

                    case '/info':
                        $info = [
                            'Protocol' => 'HTTP/3',
                            'Transport' => 'QUIC',
                            'TLS Version' => '1.3',
                            'Method' => $method,
                            'Path' => $path,
                            'Query' => $query ?: 'none',
                        ];

                        $body = "HTTP/3 Server Information\n";
                        $body .= str_repeat("=", 30) . "\n\n";
                        foreach ($info as $key => $value) {
                            $body .= sprintf("% -15s: %s\n", $key, $value);
                        }

                        $resp->withBody($body);
                        $resp->withHeader('Content-Type', 'text/plain');
                        break;

                    default:
                        $resp->withStatus(404);
                        $resp->withBody("404 Not Found\nPath: $path\n");
                        $resp->withHeader('Content-Type', 'text/plain');
                        break;
                }

                return $resp;
            });
        });
    }

} catch (Exception $e) {
    echo "Error: " . $e->getMessage() . "\n";
    exit(1);
}