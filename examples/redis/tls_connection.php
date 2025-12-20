<?php
/**
 * Redis TLS Connection Example
 *
 * This example demonstrates how to connect to a Redis server using TLS encryption.
 *
 * Prerequisites:
 * - A Redis server with TLS enabled
 * - Valid TLS certificates (or use insecure mode for testing)
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel\Redis\Client;

echo "Redis TLS Connection Example\n";
echo str_repeat("=", 50) . "\n\n";

// Create a new Redis client
$redis = new Client();

// Example 1: Connect to Redis with TLS (secure mode)
echo "Example 1: Secure TLS Connection\n";
echo str_repeat("-", 50) . "\n";

try {
    // Method 1: Using connectTls() method
    echo "Connecting to Redis with TLS (secure mode)...\n";
    $result = $redis->connectTls('redis.example.com', 6380)->await();

    if ($result) {
        echo "✓ Connected successfully with TLS!\n";

        // Test the connection
        $pong = $redis->ping()->await();
        echo "✓ Ping response: $pong\n";

        $redis->close();
        echo "✓ Connection closed\n";
    }
} catch (Exception $e) {
    echo "✗ Connection failed: " . $e->getMessage() . "\n";
}

echo "\n";

// Example 2: Connect to Redis with TLS (insecure mode - skip certificate verification)
echo "Example 2: TLS Connection with Insecure Mode\n";
echo str_repeat("-", 50) . "\n";

try {
    echo "Connecting to Redis with TLS (insecure mode)...\n";
    // Note: insecure mode skips certificate verification
    // This is useful for testing but NOT recommended for production!
    $result = $redis->connectTls('localhost', 6380, 0.0, null, null, 0.0, true)->await();

    if ($result) {
        echo "✓ Connected successfully with TLS (insecure mode)!\n";

        // Test basic operations
        echo "Testing basic operations...\n";

        // Set a value
        $redis->set('test:tls', 'Hello from TLS!')->await();
        echo "✓ Set key 'test:tls'\n";

        // Get the value
        $value = $redis->get('test:tls')->await();
        echo "✓ Get key 'test:tls': $value\n";

        // Delete the key
        $deleted = $redis->del('test:tls')->await();
        echo "✓ Deleted $deleted key(s)\n";

        $redis->close();
        echo "✓ Connection closed\n";
    }
} catch (Exception $e) {
    echo "✗ Connection failed: " . $e->getMessage() . "\n";
}

echo "\n";

// Example 3: Connect using URL format with connect() method
echo "Example 3: TLS Connection Using URL Format\n";
echo str_repeat("-", 50) . "\n";

try {
    echo "Connecting to Redis using rediss:// URL...\n";

    // Method 2: Using connect() with rediss:// URL
    // This is more flexible and follows Redis URL conventions
    $result = $redis->connect('rediss://localhost:6380#insecure')->await();

    if ($result) {
        echo "✓ Connected successfully using rediss:// URL!\n";

        // Test the connection
        $pong = $redis->ping('Hello TLS')->await();
        echo "✓ Ping with message: $pong\n";

        $redis->close();
        echo "✓ Connection closed\n";
    }
} catch (Exception $e) {
    echo "✗ Connection failed: " . $e->getMessage() . "\n";
}

echo "\n";

// Example 4: Production-ready TLS connection
echo "Example 4: Production TLS Connection with Auth\n";
echo str_repeat("-", 50) . "\n";

try {
    echo "Connecting to Redis with TLS and authentication...\n";

    // For production, use a URL with authentication
    // Format: rediss://[username]:[password]@[host]:[port]
    $result = $redis->connect('rediss://default:your-password@redis.example.com:6380')->await();

    if ($result) {
        echo "✓ Connected successfully with TLS and auth!\n";

        // Production operations...
        $info = $redis->ping()->await();
        echo "✓ Server is responsive: $info\n";

        $redis->close();
        echo "✓ Connection closed\n";
    }
} catch (Exception $e) {
    echo "✗ Connection failed: " . $e->getMessage() . "\n";
    echo "Note: This example requires a properly configured Redis server with auth\n";
}

echo "\n" . str_repeat("=", 50) . "\n";
echo "Examples completed!\n\n";

echo "TLS Connection Tips:\n";
echo "- Use 'rediss://' scheme for TLS connections\n";
echo "- Default TLS port is usually 6380 (vs 6379 for non-TLS)\n";
echo "- Append '#insecure' to skip certificate verification (testing only!)\n";
echo "- For production, always use proper certificates and verification\n";
echo "- You can include auth in the URL: rediss://user:pass@host:port\n";
