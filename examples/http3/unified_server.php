<?php
/**
 * Unified HTTP Server - HTTP/1, HTTP/2, and HTTP/3
 *
 * This example demonstrates how to run a server that supports all HTTP versions:
 * - HTTP/1.1 and HTTP/2 on port 8080 (plain TCP)
 * - HTTP/1.1 and HTTP/2 on port 8443 (TLS)
 * - HTTP/3 on port 4433 (QUIC)
 *
 * Run this example:
 * php -d extension=target/release/libasync_php.dylib examples/http3/unified_server.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;

echo "Unified HTTP Server - HTTP/1 + HTTP/2 + HTTP/3\n";
echo str_repeat("=", 60) . "\n\n";

// Check if certificates exist
$certPath = __DIR__ . '/certs/server.crt';
$keyPath = __DIR__ . '/certs/server.key';

if (!file_exists($certPath) || !file_exists($keyPath)) {
    echo "Warning: Certificates not found!\n";
    echo "Run: ./examples/http3/generate-certs.sh\n";
    echo "HTTPS and HTTP/3 will be disabled.\n\n";
    $hasCerts = false;
} else {
    $hasCerts = true;
}

// Request handler (same handler for all protocols)
$handler = function(Request $req): Response {
    $method = $req->method();
    $path = $req->path();
    $query = $req->queryString();

    // Log the request
    $protocol = $req->header('x-protocol') ?? 'HTTP/?';
    echo "[$protocol] $method $path" . ($query ? "?$query" : "") . "\n";

    $resp = new Response();
    $resp->withStatus(200);
    $resp->withHeader('Server', 'Async-PHP Unified');

    // Route handling
    switch ($path) {
        case '/':
            $resp->withHeader('Content-Type', 'text/html; charset=utf-8');
            $body = <<<HTML
<!DOCTYPE html>
<html>
<head>
    <title>Unified HTTP Server</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 900px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #2c3e50; margin-top: 0; }
        .protocol {
            background: #3498db;
            color: white;
            padding: 5px 10px;
            border-radius: 3px;
            font-weight: bold;
            display: inline-block;
        }
        .info {
            background: #ecf0f1;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }
        .endpoints {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
            margin: 20px 0;
        }
        .endpoint {
            background: #3498db;
            color: white;
            padding: 15px;
            border-radius: 5px;
            text-align: center;
        }
        .endpoint h3 { margin: 0 0 10px 0; font-size: 16px; }
        .endpoint code { background: rgba(255,255,255,0.2); padding: 5px; border-radius: 3px; display: block; margin: 5px 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Unified HTTP Server</h1>
        <p>This server supports <span class="protocol">HTTP/1.1</span>, <span class="protocol">HTTP/2</span>, and <span class="protocol">HTTP/3</span></p>

        <div class="info">
            <strong>Request Info:</strong><br>
            Method: <code>$method</code><br>
            Path: <code>$path</code><br>
            Protocol: <code>$protocol</code>
        </div>

        <h2>Available Endpoints</h2>
        <div class="endpoints">
            <div class="endpoint">
                <h3>HTTP (Plain)</h3>
                <code>http://localhost:8080/</code>
                <p style="font-size: 12px; margin: 10px 0 0 0;">HTTP/1.1, HTTP/2 (h2c)</p>
            </div>
            <div class="endpoint">
                <h3>HTTPS (TLS)</h3>
                <code>https://localhost:8443/</code>
                <p style="font-size: 12px; margin: 10px 0 0 0;">HTTP/1.1, HTTP/2</p>
            </div>
            <div class="endpoint">
                <h3>HTTP/3 (QUIC)</h3>
                <code>https://localhost:4433/</code>
                <p style="font-size: 12px; margin: 10px 0 0 0;">HTTP/3 over QUIC</p>
            </div>
        </div>

        <h2>Test Routes</h2>
        <ul>
            <li><a href="/hello">GET /hello</a> - Simple text response</li>
            <li><a href="/json">GET /json</a> - JSON response</li>
            <li><a href="/info">GET /info</a> - Server information</li>
        </ul>

        <h2>Test Commands</h2>
        <pre style="background: #2c3e50; color: #ecf0f1; padding: 15px; border-radius: 5px; overflow-x: auto;">
# HTTP/1.1 (plain)
curl http://localhost:8080/

# HTTP/2 (TLS)
curl --http2 -k https://localhost:8443/

# HTTP/3 (QUIC) - requires curl with HTTP/3 support
curl --http3 -k https://localhost:4433/</pre>
    </div>
</body>
</html>
HTML;
            $resp->withBody($body);
            break;

        case '/hello':
            $resp->withHeader('Content-Type', 'text/plain');
            $resp->withBody("Hello from $protocol! 👋\n");
            break;

        case '/json':
            $resp->withHeader('Content-Type', 'application/json');
            $data = [
                'server' => 'Async-PHP Unified',
                'protocol' => $protocol,
                'method' => $method,
                'path' => $path,
                'timestamp' => time(),
                'capabilities' => [
                    'http1' => true,
                    'http2' => true,
                    'http3' => true,
                    'tls' => true,
                    'quic' => true
                ]
            ];
            $resp->withBody(json_encode($data, JSON_PRETTY_PRINT));
            break;

        case '/info':
            $resp->withHeader('Content-Type', 'text/plain');
            $info = sprintf(
                "Server Information\n%s\n\nProtocol: %s\nMethod: %s\nPath: %s\nQuery: %s\n\nHeaders:\n%s\n",
                str_repeat("=", 40),
                $protocol,
                $method,
                $path,
                $query ?: 'none',
                json_encode($req->headers(), JSON_PRETTY_PRINT)
            );
            $resp->withBody($info);
            break;

        default:
            $resp->withStatus(404);
            $resp->withHeader('Content-Type', 'text/plain');
            $resp->withBody("404 Not Found\nPath: $path\n");
            break;
    }

    return $resp;
};

// Server configuration
$config = [
    'http' => '0.0.0.0:8080',  // HTTP/1.1 and HTTP/2 (cleartext)
];

if ($hasCerts) {
    $config['https'] = [        // HTTP/1.1 and HTTP/2 (TLS)
        'addr' => '0.0.0.0:8443',
        'cert' => $certPath,
        'key' => $keyPath
    ];
    $config['http3'] = [        // HTTP/3 (QUIC)
        'addr' => '0.0.0.0:4433',
        'cert' => $certPath,
        'key' => $keyPath
    ];
}

echo "Server Configuration:\n";
echo "- HTTP  (port 8080): HTTP/1.1, HTTP/2\n";
if ($hasCerts) {
    echo "- HTTPS (port 8443): HTTP/1.1, HTTP/2 (TLS)\n";
    echo "- HTTP/3 (port 4433): HTTP/3 over QUIC\n";
}
echo "\n";

// Start the unified server
try {
    $server = new Server();
    $server->listenAndServe($config, $handler);
} catch (Exception $e) {
    echo "Error: " . $e->getMessage() . "\n";
    exit(1);
}
