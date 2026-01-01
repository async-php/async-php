<?php
/**
 * QUIC Client Example
 *
 * This example demonstrates basic QUIC client functionality:
 * - Connecting to a QUIC server (Echo Server)
 * - Opening bidirectional streams
 * - Sending and receiving data
 *
 * Run this example:
 * php -d extension=target/release/libasync_php.dylib examples/http3/quic_client.php
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Network\Quic\Connection;

echo "QUIC Client Example\n";
echo str_repeat("=", 50) . "\n\n";

// This client sends raw text, so it connects to quic_echo_server.php.
// It does NOT connect to server.php (HTTP/3) because it doesn't speak HTTP/3 frames.

function main(): void
{
    try {
        echo "Connecting to QUIC Echo Server at 127.0.0.1:4433...\n";

        // Connect to QUIC server (with insecure mode for self-signed cert)
        // ALPN 'echo' matches quic_echo_server.php
        $config = [
            'alpn' => ['echo'], 
            'verify_cert' => false
        ];

        $conn = Connection::connect('127.0.0.1:4433', 'localhost', $config);
        echo "✓ Connected successfully via QUIC\n\n";

        // Get connection info
        $remoteAddr = $conn->remoteAddr();
        // localAddr not always available on client depending on impl
        echo "Remote address: $remoteAddr\n\n";

        // Example 1: Send a simple message
        echo "Example 1: Bidirectional Stream\n";
        echo str_repeat("-", 50) . "\n";

        $stream = $conn->openBiStream();
        $streamId = $stream->id();
        echo "Opened bidirectional stream (ID: $streamId)\n";

        // Send data
        $message = "Hello from QUIC client!";
        echo "Sending: \"$message\"\n";
        $stream->write($message);
        $stream->finish(); // Signal end of writing

        // Receive response
        $response = $stream->read(8192);
        if ($response) {
            echo "Received: \"$response\"\n";
        } else {
            echo "No response received\n";
        }

        $stream->close();
        echo "Stream closed\n\n";

        // Example 2: Multiple streams
        echo "Example 2: Multiple Concurrent Streams\n";
        echo str_repeat("-", 50) . "\n";

        $streams = [];
        for ($i = 1; $i <= 3; $i++) {
            $stream = $conn->openBiStream();
            $streams[] = $stream;

            $msg = "Stream $i message";
            echo "Stream {$stream->id()}: Sending \"$msg\"\n";
            $stream->write($msg);
            $stream->finish();
        }

        echo "\nReceiving responses...\n";
        foreach ($streams as $i => $stream) {
            $response = $stream->read(8192);
            echo "Stream {$stream->id()}: Received \"$response\"\n";
            $stream->close();
        }

        echo "\n";

        // Example 3: Unidirectional stream (send only)
        echo "Example 3: Unidirectional Stream (Send)\n";
        echo str_repeat("-", 50) . "\n";

        $sendStream = $conn->openUniStream();
        echo "Opened unidirectional stream (ID: {$sendStream->id()})\n";

        $sendStream->write("One-way message from client");
        $sendStream->finish();
        $sendStream->close();
        echo "Sent one-way message\n\n";

        // Close connection
        echo "Closing QUIC connection...\n";
        $conn->close(0, "Client closing");
        echo "✓ Connection closed\n";

    } catch (Exception $e) {
        echo "Error: " . $e->getMessage() . "\n";
        exit(1);
    }
}

// Run the main function
// Assuming environment supports top-level execution or Kernel::run
// Using basic execution since Fiber creation is handled inside logic if needed?
// But main uses blocking/suspending calls, so it must be in a Fiber.
if (class_exists('Async\Kernel')) {
    \Async\Kernel::run(fn() => main());
} elseif (function_exists('run')) {
    run(fn() => main());
} else {
    // Fallback if no run loop (likely will fail on suspend)
    echo "Warning: No event loop detected. Attempting to run directly...\n";
    main();
}