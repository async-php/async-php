<?php

/**
 * Zero-Copy HTTP Server Example
 *
 * This example demonstrates the zero-copy HTTP server implementation
 * that directly uses tokio's IO without memory copies between Rust and PHP.
 *
 * Features:
 * - Direct tokio AsyncRead/AsyncWrite integration
 * - No memory copies for request/response bodies
 * - Supports HTTP/1.1 and HTTP/2
 * - Go-style concurrent connection handling
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Tcp\Listener;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

// Main server function
function main(): void
{
    $addr = '127.0.0.1:9001';
    echo "Starting HTTP server on http://$addr\n";
    echo "Zero-copy mode: TcpStream -> tokio IO (no memory copies)\n";
    echo "Press Ctrl+C to stop\n\n";

    // Create HTTP server
    $server = new Server();

    // Optional: Configure HTTP/1.1 or HTTP/2 only
    // $server->http1Only();
    // $server->http2Only();

    // Optional: Configure HTTP settings
    // $server->http1(['keep_alive' => true, 'max_headers' => 100]);
    // $server->http2(['max_concurrent_streams' => 200]);

    // Bind TCP listener
    $listener = Listener::bind($addr);

    echo "Server ready, accepting connections...\n\n";

    // Accept and serve connections (Go-style)
    while (true) {
        // Accept new connection
        $conn = $listener->accept();
        $peerAddr = $conn->remoteAddr();

        echo "New connection from: $peerAddr\n";

        // Spawn a fiber to handle this connection
        // This allows concurrent connection handling
        go(function() use ($server, $conn, $peerAddr) {
            try {
                // Serve HTTP on this connection using zero-copy IO
                // The connection's native tokio TcpStream is used directly
                $server->serve($conn, function(HttpRequest $req) use ($peerAddr): HttpResponse {
                    $method = $req->method();
                    $path = $req->path();
                    $query = $req->queryString();

                    echo "[$peerAddr] $method $path" . ($query ? "?$query" : "") . "\n";

                    // Create response
                    $resp = new HttpResponse();
                    $resp->setStatus(200);
                    $resp->setHeader('Content-Type', 'text/plain');
                    $resp->setHeader('Server', 'Async-PHP/1.0');

                    // Build response body
                    $body = "Hello from Async PHP!\n\n";
                    $body .= "Request Details:\n";
                    $body .= "  Method: $method\n";
                    $body .= "  Path: $path\n";
                    $body .= "  Query: " . ($query ?: "(none)") . "\n";
                    $body .= "  Client: $peerAddr\n\n";
                    $body .= "This response was generated with ZERO memory copies!\n";
                    $body .= "The connection uses native tokio IO directly.\n";

                    $resp->setBody($body);

                    return $resp;
                });

                echo "[$peerAddr] Connection closed\n";
            } catch (\Throwable $e) {
                echo "[$peerAddr] Error: {$e->getMessage()}\n";
            }
        });
    }
}

// Alternative: Use the convenience method
function mainConvenience(): void
{
    $server = new Server();

    // This does the same as main() but in one call
    $server->listenAndServe('127.0.0.1:9001', function(HttpRequest $req): HttpResponse {
        $resp = new HttpResponse();
        $resp->setStatus(200);
        $resp->setHeader('Content-Type', 'text/html');
        $resp->setBody('<h1>Hello from Async PHP!</h1>');
        return $resp;
    });
}

// Run the server
main();
// Or use the convenience method:
// mainConvenience();
