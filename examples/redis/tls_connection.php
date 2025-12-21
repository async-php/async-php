<?php
/**
 * Redis TLS Connection Example
 *
 * This example demonstrates comprehensive Redis operations over TLS connection.
 *
 * Prerequisites:
 * - A Redis server with TLS enabled (see start-redis-tls.sh)
 * - TLS certificates (see redis-cert.pem and redis-key.pem)
 *
 * To start the TLS Redis server:
 *   ./start-redis-tls.sh
 *
 * Server info:
 * - Host: localhost
 * - Port: 6380 (TLS)
 * - Password: tls123
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;

echo "Redis TLS Connection Example\n";
echo str_repeat("=", 70) . "\n\n";

Kernel::run(function () {
    // Create a new Redis client using the hooked Redis class
    $redis = new Redis();

    // ========================================================================
    // Example 1: TLS Connection with Password Authentication
    // ========================================================================
    echo "Example 1: TLS Connection with Password Authentication\n";
    echo str_repeat("-", 70) . "\n";

    try {
        echo "Connecting to Redis with TLS (insecure mode for self-signed cert)...\n";

        // Connect using rediss:// URL with password
        // #insecure flag skips certificate verification (for self-signed certs)
        $result = $redis->connect('rediss://:tls123@localhost:6380#insecure');

        if (!$result) {
            throw new Exception("Failed to connect to Redis");
        }
        echo "✓ Connected successfully with TLS!\n\n";

        // Test the connection
        $pong = $redis->ping('TLS Test');
        echo "PING Response: $pong\n\n";

    } catch (Exception $e) {
        echo "✗ Connection failed: " . $e->getMessage() . "\n";
        echo "Make sure Redis TLS server is running: ./start-redis-tls.sh\n\n";
        exit(1);
    }

    // ========================================================================
    // Example 2: String Operations over TLS
    // ========================================================================
    echo "Example 2: String Operations over TLS\n";
    echo str_repeat("-", 70) . "\n";

    // Basic SET/GET
    $redis->set('tls:message', 'Hello from TLS connection!');
    echo "SET tls:message = 'Hello from TLS connection!'\n";

    $value = $redis->get('tls:message');
    echo "GET tls:message = '$value'\n\n";

    // SET with expiration
    $redis->set('tls:temp', 'Temporary data', 10);
    echo "SET tls:temp with 10s expiration\n";

    $ttl = $redis->ttl('tls:temp');
    echo "TTL tls:temp = $ttl seconds\n\n";

    // SETEX
    $redis->setex('tls:session', 30, 'session-data-12345');
    echo "SETEX tls:session 30 'session-data-12345'\n";

    $sessionData = $redis->get('tls:session');
    echo "GET tls:session = '$sessionData'\n\n";

    // ========================================================================
    // Example 3: Counter Operations over TLS
    // ========================================================================
    echo "Example 3: Counter Operations over TLS\n";
    echo str_repeat("-", 70) . "\n";

    $redis->set('tls:counter', '100');
    echo "Initial counter: 100\n";

    $val = $redis->incr('tls:counter');
    echo "INCR tls:counter = $val\n";

    $val = $redis->incrBy('tls:counter', 50);
    echo "INCRBY tls:counter 50 = $val\n";

    $val = $redis->decr('tls:counter');
    echo "DECR tls:counter = $val\n";

    $val = $redis->decrBy('tls:counter', 10);
    echo "DECRBY tls:counter 10 = $val\n\n";

    // ========================================================================
    // Example 4: List Operations over TLS
    // ========================================================================
    echo "Example 4: List Operations over TLS\n";
    echo str_repeat("-", 70) . "\n";

    $redis->del('tls:queue');

    $redis->rPush('tls:queue', 'job1', 'job2', 'job3');
    echo "RPUSH tls:queue job1 job2 job3\n";

    $redis->lPush('tls:queue', 'urgent-job');
    echo "LPUSH tls:queue urgent-job\n";

    $len = $redis->lLen('tls:queue');
    echo "LLEN tls:queue = $len\n";

    $items = $redis->lRange('tls:queue', 0, -1);
    echo "LRANGE tls:queue 0 -1 = " . json_encode($items) . "\n";

    $first = $redis->lPop('tls:queue');
    echo "LPOP tls:queue = '$first'\n";

    $last = $redis->rPop('tls:queue');
    echo "RPOP tls:queue = '$last'\n\n";

    // ========================================================================
    // Example 5: Hash Operations over TLS
    // ========================================================================
    echo "Example 5: Hash Operations over TLS\n";
    echo str_repeat("-", 70) . "\n";

    $redis->hSet('tls:user:1001', 'name', 'Alice');
    $redis->hSet('tls:user:1001', 'email', 'alice@example.com');
    $redis->hSet('tls:user:1001', 'age', '28');
    $redis->hSet('tls:user:1001', 'role', 'admin');
    echo "HSET tls:user:1001 (4 fields)\n";

    $name = $redis->hGet('tls:user:1001', 'name');
    echo "HGET tls:user:1001 name = '$name'\n";

    $exists = $redis->hExists('tls:user:1001', 'email');
    echo "HEXISTS tls:user:1001 email = " . ($exists ? 'true' : 'false') . "\n";

    $user = $redis->hGetAll('tls:user:1001');
    echo "HGETALL tls:user:1001 = " . json_encode($user) . "\n";

    $deleted = $redis->hDel('tls:user:1001', 'age');
    echo "HDEL tls:user:1001 age = $deleted field(s) deleted\n\n";

    // ========================================================================
    // Example 6: Set Operations over TLS
    // ========================================================================
    echo "Example 6: Set Operations over TLS\n";
    echo str_repeat("-", 70) . "\n";

    $redis->sAdd('tls:tags', 'php', 'rust', 'async', 'tls', 'redis');
    echo "SADD tls:tags php rust async tls redis\n";

    $isMember = $redis->sIsMember('tls:tags', 'tls');
    echo "SISMEMBER tls:tags tls = " . ($isMember ? 'true' : 'false') . "\n";

    $members = $redis->sMembers('tls:tags');
    echo "SMEMBERS tls:tags = " . json_encode($members) . "\n";

    $removed = $redis->sRem('tls:tags', 'rust');
    echo "SREM tls:tags rust = $removed member(s) removed\n\n";

    // ========================================================================
    // Example 7: Key Management over TLS
    // ========================================================================
    echo "Example 7: Key Management over TLS\n";
    echo str_repeat("-", 70) . "\n";

    $exists = $redis->exists('tls:message');
    echo "EXISTS tls:message = $exists\n";

    $exists = $redis->exists(['tls:message', 'tls:counter', 'tls:queue']);
    echo "EXISTS tls:message tls:counter tls:queue = $exists keys exist\n";

    $redis->expire('tls:message', 60);
    echo "EXPIRE tls:message 60\n";

    $ttl = $redis->ttl('tls:message');
    echo "TTL tls:message = $ttl seconds\n\n";

    // ========================================================================
    // Example 8: Bulk Operations and Cleanup
    // ========================================================================
    echo "Example 8: Bulk Operations and Cleanup\n";
    echo str_repeat("-", 70) . "\n";

    // Delete multiple keys at once
    $deleted = $redis->del([
        'tls:message',
        'tls:temp',
        'tls:session',
        'tls:counter',
        'tls:queue',
        'tls:user:1001',
        'tls:tags'
    ]);
    echo "DEL (bulk) = $deleted keys deleted\n\n";

    // ========================================================================
    // Close connection
    // ========================================================================
    $redis->close();
    echo str_repeat("=", 70) . "\n";
    echo "✓ All TLS operations completed successfully!\n";
    echo "✓ Connection closed\n";
});

echo "\n" . str_repeat("=", 70) . "\n";
echo "TLS Connection Tips:\n";
echo "- Use 'rediss://' scheme for TLS connections\n";
echo "- Default TLS port is usually 6380 (vs 6379 for non-TLS)\n";
echo "- Append '#insecure' to skip certificate verification (testing only!)\n";
echo "- For production, use valid certificates and proper verification\n";
echo "- Include auth in URL: rediss://[user]:[password]@host:port\n";
echo "- TLS provides encryption for data in transit\n";
