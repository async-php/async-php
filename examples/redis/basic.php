<?php
/**
 * Basic Redis Operations Example
 *
 * This example demonstrates basic Redis operations using the async Redis client.
 */

require_once __DIR__ . '/../../vendor/autoload.php';

use Async\Kernel;
use Async\Kernel\Redis\Client;

echo "Basic Redis Operations Example\n";
echo str_repeat("=", 50) . "\n\n";

Kernel::run(function () {
    // Create a new Redis client
    $redis = new Client();

    try {
        // Connect to Redis
        // For Redis with password, use URL format: redis://:password@host:port
        // For Redis without password, use: redis://host:port or just connect(host, port)
        echo "Connecting to Redis...\n";
        // Change this to match your Redis configuration:
        $result = Fiber::suspend($redis->connect('127.0.0.1', 6379));
        // For Redis with password, use:
        // $result = Fiber::suspend($redis->connect('redis://:your-password@127.0.0.1:6379'));

        if (!$result) {
            throw new Exception("Failed to connect to Redis");
        }
        echo "✓ Connected to Redis\n\n";

        // Ping test
        echo "1. Testing connection with PING\n";
        $pong = Fiber::suspend($redis->ping());
        echo "   Response: $pong\n\n";

        // String operations
        echo "2. String Operations\n";
        echo "   Setting key 'user:1:name' = 'John Doe'\n";
        Fiber::suspend($redis->set('user:1:name', 'John Doe'));

        $name = Fiber::suspend($redis->get('user:1:name'));
        echo "   Getting key 'user:1:name': $name\n\n";

        // String with expiration
        echo "3. String with Expiration\n";
        echo "   Setting key 'session:abc123' with 60s expiration\n";
        Fiber::suspend($redis->set('session:abc123', 'user-data', 60));

        $ttl = Fiber::suspend($redis->ttl('session:abc123'));
        echo "   TTL: $ttl seconds\n\n";

        // Counter operations
        echo "4. Counter Operations\n";
        Fiber::suspend($redis->set('counter', '0'));
        echo "   Initial counter: 0\n";

        $value = Fiber::suspend($redis->incr('counter'));
        echo "   After INCR: $value\n";

        $value = Fiber::suspend($redis->incrBy('counter', 5));
        echo "   After INCRBY 5: $value\n";

        $value = Fiber::suspend($redis->decr('counter'));
        echo "   After DECR: $value\n\n";

        // List operations
        echo "5. List Operations\n";
        Fiber::suspend($redis->del('tasks'));

        Fiber::suspend($redis->rPush('tasks', ['Task 1', 'Task 2', 'Task 3']));
        echo "   Pushed 3 tasks to list\n";

        $len = Fiber::suspend($redis->lLen('tasks'));
        echo "   List length: $len\n";

        $tasks = Fiber::suspend($redis->lRange('tasks', 0, -1));
        echo "   All tasks: " . json_encode($tasks) . "\n";

        $task = Fiber::suspend($redis->lPop('tasks'));
        echo "   Popped task: $task\n\n";

        // Hash operations
        echo "6. Hash Operations\n";
        Fiber::suspend($redis->hSet('user:1', 'name', 'John Doe'));
        Fiber::suspend($redis->hSet('user:1', 'email', 'john@example.com'));
        Fiber::suspend($redis->hSet('user:1', 'age', '30'));
        echo "   Set user hash fields\n";

        $email = Fiber::suspend($redis->hGet('user:1', 'email'));
        echo "   Email: $email\n";

        $user = Fiber::suspend($redis->hGetAll('user:1'));
        echo "   All user data: " . json_encode($user) . "\n\n";

        // Set operations
        echo "7. Set Operations\n";
        Fiber::suspend($redis->sAdd('tags', ['php', 'rust', 'async', 'redis']));
        echo "   Added tags to set\n";

        $isMember = Fiber::suspend($redis->sIsMember('tags', 'php'));
        echo "   Is 'php' a member? " . ($isMember ? 'Yes' : 'No') . "\n";

        $tags = Fiber::suspend($redis->sMembers('tags'));
        echo "   All tags: " . json_encode($tags) . "\n\n";

        // Key existence and deletion
        echo "8. Key Management\n";
        $exists = Fiber::suspend($redis->exists('user:1'));
        echo "   'user:1' exists? " . ($exists ? 'Yes' : 'No') . "\n";

        $deleted = Fiber::suspend($redis->del(['user:1:name', 'user:1', 'tasks']));
        echo "   Deleted $deleted key(s)\n\n";

        // Close connection
        $redis->close();
        echo "✓ Connection closed\n";

    } catch (Exception $e) {
        echo "✗ Error: " . $e->getMessage() . "\n";
        echo "Error details: " . $redis->getLastError() . "\n";
    }
});

echo "\n" . str_repeat("=", 50) . "\n";
echo "Example completed!\n";
