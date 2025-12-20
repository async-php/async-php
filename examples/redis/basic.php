<?php
/**
 * Basic Redis Operations Example
 *
 * This example demonstrates basic Redis operations using the async Redis client.
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel\Redis\Client;

echo "Basic Redis Operations Example\n";
echo str_repeat("=", 50) . "\n\n";

// Create a new Redis client
$redis = new Client();

try {
    // Connect to Redis
    echo "Connecting to Redis...\n";
    $result = $redis->connect('127.0.0.1', 6379)->await();

    if (!$result) {
        throw new Exception("Failed to connect to Redis");
    }
    echo "✓ Connected to Redis\n\n";

    // Ping test
    echo "1. Testing connection with PING\n";
    $pong = $redis->ping()->await();
    echo "   Response: $pong\n\n";

    // String operations
    echo "2. String Operations\n";
    echo "   Setting key 'user:1:name' = 'John Doe'\n";
    $redis->set('user:1:name', 'John Doe')->await();

    $name = $redis->get('user:1:name')->await();
    echo "   Getting key 'user:1:name': $name\n\n";

    // String with expiration
    echo "3. String with Expiration\n";
    echo "   Setting key 'session:abc123' with 60s expiration\n";
    $redis->set('session:abc123', 'user-data', 60)->await();

    $ttl = $redis->ttl('session:abc123')->await();
    echo "   TTL: $ttl seconds\n\n";

    // Counter operations
    echo "4. Counter Operations\n";
    $redis->set('counter', '0')->await();
    echo "   Initial counter: 0\n";

    $value = $redis->incr('counter')->await();
    echo "   After INCR: $value\n";

    $value = $redis->incrBy('counter', 5)->await();
    echo "   After INCRBY 5: $value\n";

    $value = $redis->decr('counter')->await();
    echo "   After DECR: $value\n\n";

    // List operations
    echo "5. List Operations\n";
    $redis->del('tasks')->await();

    $redis->rPush('tasks', ['Task 1', 'Task 2', 'Task 3'])->await();
    echo "   Pushed 3 tasks to list\n";

    $len = $redis->lLen('tasks')->await();
    echo "   List length: $len\n";

    $tasks = $redis->lRange('tasks', 0, -1)->await();
    echo "   All tasks: " . json_encode($tasks) . "\n";

    $task = $redis->lPop('tasks')->await();
    echo "   Popped task: $task\n\n";

    // Hash operations
    echo "6. Hash Operations\n";
    $redis->hSet('user:1', 'name', 'John Doe')->await();
    $redis->hSet('user:1', 'email', 'john@example.com')->await();
    $redis->hSet('user:1', 'age', '30')->await();
    echo "   Set user hash fields\n";

    $email = $redis->hGet('user:1', 'email')->await();
    echo "   Email: $email\n";

    $user = $redis->hGetAll('user:1')->await();
    echo "   All user data: " . json_encode($user) . "\n\n";

    // Set operations
    echo "7. Set Operations\n";
    $redis->sAdd('tags', ['php', 'rust', 'async', 'redis'])->await();
    echo "   Added tags to set\n";

    $isMember = $redis->sIsMember('tags', 'php')->await();
    echo "   Is 'php' a member? " . ($isMember ? 'Yes' : 'No') . "\n";

    $tags = $redis->sMembers('tags')->await();
    echo "   All tags: " . json_encode($tags) . "\n\n";

    // Key existence and deletion
    echo "8. Key Management\n";
    $exists = $redis->exists('user:1')->await();
    echo "   'user:1' exists? " . ($exists ? 'Yes' : 'No') . "\n";

    $deleted = $redis->del(['user:1:name', 'user:1', 'tasks'])->await();
    echo "   Deleted $deleted key(s)\n\n";

    // Close connection
    $redis->close();
    echo "✓ Connection closed\n";

} catch (Exception $e) {
    echo "✗ Error: " . $e->getMessage() . "\n";
    echo "Error details: " . $redis->getLastError() . "\n";
}

echo "\n" . str_repeat("=", 50) . "\n";
echo "Example completed!\n";
