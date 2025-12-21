<?php
/**
 * Simple Multi-Protocol HTTP Server
 *
 * This example shows how to run HTTP/1, HTTP/2, and HTTP/3 on the SAME port.
 * The server automatically handles protocol negotiation.
 *
 * Run this example:
 * php -d extension=target/release/libasync_php.dylib examples/http3/simple_server.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Fiber;

echo "Multi-Protocol HTTP Server\n";
echo str_repeat("=", 50) . "\n\n";

// Check if certificates exist
$certPath = __DIR__ . '/certs/server.crt';
$keyPath = __DIR__ . '/certs/server.key';

if (!file_exists($certPath) || !file_exists($keyPath)) {
    echo "Certificates not found! Running HTTP only...\n";
    echo "To enable HTTP/3, run:\n";
    echo "  cd examples/http3 && ./generate-certs.sh\n\n";

    $config = '0.0.0.0:8080';  // HTTP only
} else {
    echo "Certificates found! Enabling HTTP/3...\n\n";

    $config = [
        'addr' => '0.0.0.0:8080',  // Same port for both
        'cert' => $certPath,
        'key' => $keyPath,
        'http3' => true  // Enable HTTP/3
    ];
}

// Request handler
$handler = function(Request $req): Response {
    $method = $req->method();
    $path = $req->path();

    echo "[$method] $path\n";

    $resp = new Response();
    $resp->withStatus(200);
    $resp->withHeader('Content-Type', 'text/html; charset=utf-8');
    $resp->withHeader('Server', 'Async-PHP Multi-Protocol');

    $body = <<<HTML
<!DOCTYPE html>
<html>
<head>
    <title>Multi-Protocol Server</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            line-height: 1.6;
        }
        h1 { color: #2563eb; }
        .badge {
            display: inline-block;
            padding: 4px 12px;
            margin: 2px;
            border-radius: 4px;
            font-size: 14px;
            font-weight: 600;
        }
        .http1 { background: #f59e0b; color: white; }
        .http2 { background: #10b981; color: white; }
        .http3 { background: #8b5cf6; color: white; }
        .info {
            background: #f3f4f6;
            padding: 15px;
            border-radius: 8px;
            margin: 20px 0;
        }
        code {
            background: #e5e7eb;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Monaco', monospace;
        }
    </style>
</head>
<body>
    <h1>🚀 Multi-Protocol HTTP Server</h1>

    <p>This server supports multiple HTTP versions on the <strong>same port</strong>:</p>

    <div>
        <span class="badge http1">HTTP/1.1</span>
        <span class="badge http2">HTTP/2</span>
        <span class="badge http3">HTTP/3</span>
    </div>

    <div class="info">
        <strong>Request Details:</strong><br>
        Method: <code>$method</code><br>
        Path: <code>$path</code>
    </div>

    <h2>How It Works</h2>
    <ul>
        <li><strong>HTTP/1.1 & HTTP/2:</strong> TCP listener with TLS (ALPN negotiation)</li>
        <li><strong>HTTP/3:</strong> QUIC listener on the same UDP port</li>
        <li><strong>Protocol Upgrade:</strong> Alt-Svc header advertises HTTP/3 availability</li>
    </ul>

    <h2>Test Commands</h2>
    <pre style="background: #1f2937; color: #e5e7eb; padding: 15px; border-radius: 8px; overflow-x: auto;">
# HTTP/2 (will get Alt-Svc header)
curl -v --http2 -k https://localhost:8443/

# HTTP/3 (direct)
curl --http3 -k https://localhost:8443/

# Check Alt-Svc header
curl -sI --http2 -k https://localhost:8443/ | grep -i alt-svc
</pre>

    <p><small>Note: HTTP/3 requires curl built with HTTP/3 support</small></p>
</body>
</html>
HTML;

    $resp->withBody($body);
    return $resp;
};

// Start the server
$fiber = new Fiber(function() use ($config, $handler) {
    try {
        $server = new Server();
        $server->listenAndServe($config, $handler);
    } catch (Exception $e) {
        echo "Error: " . $e->getMessage() . "\n";
        exit(1);
    }
});

run($fiber);
