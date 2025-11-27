<?php
/**
 * Async Redis Example
 *
 * This example demonstrates async Redis operations using the Redis extension wrapper.
 * Make sure you have Redis server running on localhost:6379 before running this example.
 *
 * Usage:
 *   php -d extension=target/release/libasync_php.dylib examples/redis_test.php
 */

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\Time;
use Redis\Redis;

Kernel::run(function () {
    echo "=== Async Redis Example ===\n\n";

    // Create Redis instance
    $redis = new Redis();

    // Connect to Redis server
    echo "1. Connecting to Redis...\n";
    if (!$redis->connect('127.0.0.1', 6379, 5.0)) {
        echo "Failed to connect to Redis server!\n";
        echo "Error: " . $redis->getLastError() . "\n";
        echo "Make sure Redis is running on localhost:6379\n";
        return;
    }
    echo "   ✓ Connected successfully\n\n";

    // Test ping
    echo "2. Testing ping...\n";
    $pong = $redis->ping();
    echo "   Response: {$pong}\n";

    $customPong = $redis->ping("Hello Redis!");
    echo "   Custom ping: {$customPong}\n\n";

    // String operations
    echo "3. String operations...\n";
    $redis->set('name', 'Alice');
    $redis->set('age', '30');
    $redis->set('city', 'New York');

    echo "   name = " . $redis->get('name') . "\n";
    echo "   age = " . $redis->get('age') . "\n";
    echo "   city = " . $redis->get('city') . "\n\n";

    // Set with expiration
    echo "4. Set with expiration...\n";
    $redis->set('session:123', 'user_data', 5); // Expires in 5 seconds
    echo "   session:123 = " . $redis->get('session:123') . "\n";
    echo "   TTL: " . $redis->ttl('session:123') . " seconds\n\n";

    // Counter operations
    echo "5. Counter operations...\n";
    $redis->set('counter', '0');
    $count1 = $redis->incr('counter');
    $count2 = $redis->incrBy('counter', 5);
    $count3 = $redis->decr('counter');
    echo "   After incr: {$count1}\n";
    echo "   After incrBy(5): {$count2}\n";
    echo "   After decr: {$count3}\n\n";

    // List operations
    echo "6. List operations...\n";
    $redis->del('tasks');
    $redis->rPush('tasks', 'Task 1', 'Task 2', 'Task 3');
    $redis->lPush('tasks', 'Urgent Task');

    $allTasks = $redis->lRange('tasks', 0, -1);
    echo "   All tasks: " . implode(', ', $allTasks) . "\n";
    echo "   List length: " . $redis->lLen('tasks') . "\n";

    $firstTask = $redis->lPop('tasks');
    echo "   Popped first task: {$firstTask}\n";

    $lastTask = $redis->rPop('tasks');
    echo "   Popped last task: {$lastTask}\n\n";

    // Hash operations
    echo "7. Hash operations...\n";
    $redis->hSet('user:1', 'name', 'Bob');
    $redis->hSet('user:1', 'email', 'bob@example.com');
    $redis->hSet('user:1', 'age', '25');

    $userName = $redis->hGet('user:1', 'name');
    echo "   user:1 name: {$userName}\n";

    $user = $redis->hGetAll('user:1');
    echo "   user:1 data: " . json_encode($user) . "\n";

    $exists = $redis->hExists('user:1', 'email');
    echo "   Email field exists: " . ($exists ? 'yes' : 'no') . "\n\n";

    // Set operations
    echo "8. Set operations...\n";
    $redis->del('tags');
    $redis->sAdd('tags', 'php', 'rust', 'async');
    $redis->sAdd('tags', 'redis', 'tokio');

    $allTags = $redis->sMembers('tags');
    echo "   All tags: " . implode(', ', $allTags) . "\n";

    $isPhp = $redis->sIsMember('tags', 'php');
    $isPython = $redis->sIsMember('tags', 'python');
    echo "   Has 'php': " . ($isPhp ? 'yes' : 'no') . "\n";
    echo "   Has 'python': " . ($isPython ? 'yes' : 'no') . "\n";

    $removed = $redis->sRem('tags', 'rust');
    echo "   Removed 'rust': {$removed} member(s)\n\n";

    // Key operations
    echo "9. Key operations...\n";
    $exists = $redis->exists('name');
    echo "   'name' exists: {$exists}\n";

    $redis->expire('name', 10);
    $ttl = $redis->ttl('name');
    echo "   'name' TTL: {$ttl} seconds\n";

    $deleted = $redis->del(['age', 'city']);
    echo "   Deleted {$deleted} key(s)\n\n";

    // Concurrent operations with go()
    echo "10. Concurrent Redis operations...\n";

    // Spawn multiple concurrent tasks
    go(function () use ($redis) {
        for ($i = 1; $i <= 5; $i++) {
            $redis->set("task1:item{$i}", "value{$i}");
            Time::sleep(10); // Small delay
        }
        echo "   Task 1 completed\n";
    });

    go(function () use ($redis) {
        for ($i = 1; $i <= 5; $i++) {
            $redis->set("task2:item{$i}", "value{$i}");
            Time::sleep(10);
        }
        echo "   Task 2 completed\n";
    });

    go(function () use ($redis) {
        for ($i = 1; $i <= 5; $i++) {
            $redis->incr("concurrent:counter");
            Time::sleep(10);
        }
        $final = $redis->get("concurrent:counter");
        echo "   Task 3 completed (counter: {$final})\n";
    });

    // Wait for concurrent tasks
    Time::sleep(100);

    // Verify concurrent operations
    $counter = $redis->get("concurrent:counter");
    echo "   Final counter value: {$counter}\n\n";

    // Check expiration
    echo "11. Checking expiration (after 5+ seconds)...\n";
    Time::sleep(6000); // Wait 6 seconds
    $sessionData = $redis->get('session:123');
    echo "   session:123 = " . ($sessionData === false ? 'expired (false)' : $sessionData) . "\n\n";

    // Cleanup
    echo "12. Cleanup...\n";
    $redis->del([
        'name', 'age', 'city', 'counter', 'tasks', 'user:1', 'tags',
        'task1:item1', 'task1:item2', 'task1:item3', 'task1:item4', 'task1:item5',
        'task2:item1', 'task2:item2', 'task2:item3', 'task2:item4', 'task2:item5',
        'concurrent:counter'
    ]);
    echo "   ✓ Cleanup completed\n\n";

    // Close connection
    $redis->close();
    echo "=== Example completed successfully ===\n";
});
