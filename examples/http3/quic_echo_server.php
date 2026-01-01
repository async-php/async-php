<?php
/**
 * QUIC Echo Server Example
 *
 * This is a simple QUIC server that echoes back any data it receives.
 * Useful for testing QUIC client functionality.
 *
 * Run this example:
 * php -d extension=target/release/libasync_php.dylib examples/http3/quic_echo_server.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Network\Quic\Listener;
use Async\Network\Quic\Connection;

echo "QUIC Echo Server Example\n";
echo str_repeat("=", 50) . "\n\n";

// Check if certificates exist
$certPath = __DIR__ . '/certs/server.crt';
$keyPath = __DIR__ . '/certs/server.key';

if (!file_exists($certPath) || !file_exists($keyPath)) {
    echo "Error: Certificates not found!\n";
    echo "Please run: ./examples/http3/generate-certs.sh\n\n";
    exit(1);
}

function main(): void
{
    global $certPath, $keyPath;

    echo "Starting QUIC echo server on 127.0.0.1:4433...\n";
    echo "Certificate: $certPath\n";
    echo "Private Key: $keyPath\n";
    echo str_repeat("-", 50) . "\n\n";

    // TLS configuration
    $tlsConfig = [
        'cert_path' => $certPath,
        'key_path' => $keyPath,
        'alpn' => ['echo', 'h3']
    ];

    try {
        // Create QUIC listener
        $listener = Listener::bind('127.0.0.1:4433', $tlsConfig);
        $localAddr = $listener->localAddr();

        echo "✓ Server listening on $localAddr\n";
        echo "Waiting for connections...\n\n";

        $connCount = 0;

        // Accept connections
        while (true) {
            $conn = $listener->accept();
            $connCount++;

            echo "[$connCount] New connection incoming...\n";

            // Handle each connection in a separate fiber
            go(function() use ($conn, $connCount) {
                try {
                    // Wait for handshake to complete to get remote address
                    $conn->handshake();
                    $remoteAddr = $conn->remoteAddr();
                    echo "[$connCount] Connection established from $remoteAddr\n";
                    handleConnection($conn, $connCount, $remoteAddr);
                } catch (Throwable $e) {
                    echo "[$connCount] Error: " . $e->getMessage() . "\n";
                }
            });
        }

    } catch (Exception $e) {
        echo "Server error: " . $e->getMessage() . "\n";
        exit(1);
    }
}

function handleConnection(Connection $conn, int $connId, string $remoteAddr): void
{
    echo "[$connId] Handling connection from $remoteAddr\n";

    $streamCount = 0;

    // Spawn uni stream acceptor
    go(function() use ($conn, $connId) {
        $uniCount = 0;
        while (true) {
            try {
                $recvStream = $conn->acceptUniStream();
                if (!$recvStream) break;
                
                $uniCount++;
                $streamId = $recvStream->id();
                echo "[$connId:$streamId] New unidirectional stream\n";
                
                go(function() use ($recvStream, $connId, $streamId) {
                    try {
                        while ($data = $recvStream->read(8192)) {
                             echo "[$connId:$streamId] Received " . strlen($data) . " bytes (uni): \"$data\"\n";
                        }
                        echo "[$connId:$streamId] Uni stream closed\n";
                    } catch (Throwable $e) {
                        echo "[$connId:$streamId] Uni stream error: " . $e->getMessage() . "\n";
                    }
                });
            } catch (Throwable $e) {
                break;
            }
        }
    });

    // Accept bidirectional streams in main loop
    while (true) {
        try {
            $stream = $conn->acceptBiStream();

            if (!$stream) {
                echo "[$connId] Connection closed (no more bi streams)\n";
                break;
            }

            $streamCount++;
            $streamId = $stream->id();

            echo "[$connId:$streamId] New bidirectional stream (stream #$streamCount)\n";

            go(function() use ($stream, $connId, $streamId) {
                try {
                    while ($data = $stream->read(8192)) {
                        $len = strlen($data);
                        echo "[$connId:$streamId] Received $len bytes: \"$data\"\n";

                        $echoMsg = "ECHO: $data";
                        $stream->write($echoMsg);
                        echo "[$connId:$streamId] Sent echo response\n";
                    }

                    $stream->finish();
                    $stream->close();
                    echo "[$connId:$streamId] Stream closed\n";

                } catch (Throwable $e) {
                    echo "[$connId:$streamId] Stream error: " . $e->getMessage() . "\n";
                }
            });

        } catch (Throwable $e) {
            echo "[$connId] Error accepting stream: " . $e->getMessage() . "\n";
            break;
        }
    }

    echo "[$connId] Connection handler finished\n\n";
}

// Run logic
if (class_exists('Async\Kernel')) {
    \Async\Kernel::run(fn() => main());
} elseif (function_exists('run')) {
    run(fn() => main());
} else {
    echo "Warning: No event loop detected.\n";
    main();
}
