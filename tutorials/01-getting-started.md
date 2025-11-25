# Getting Started with Async-PHP

This guide will help you write your first async PHP program.

## Basic Concepts

### 1. Kernel and Event Loop

The `Kernel::run()` function starts the async event loop:

```php
use Async\Kernel;

Kernel::run(function () {
    // Your async code here
});
```

### 2. Coroutines with `go()`

Use `go()` to spawn concurrent tasks:

```php
Kernel::run(function () {
    go(function () {
        echo "Task 1\n";
    });

    go(function () {
        echo "Task 2\n";
    });
});
```

### 3. Async Sleep

```php
use Async\Time;

Kernel::run(function () {
    echo "Starting...\n";
    Time::sleep(1000); // Sleep for 1000ms
    echo "Done!\n";
});
```

## Your First Program

Create `hello_async.php`:

```php
<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    echo "Main task started\n";

    // Spawn 3 concurrent tasks
    for ($i = 1; $i <= 3; $i++) {
        go(function() use ($i) {
            echo "Task $i: Starting\n";
            Time::sleep(100 * $i); // Different sleep times
            echo "Task $i: Completed\n";
        });
    }

    // Main task continues
    echo "All tasks spawned\n";
    Time::sleep(500);
    echo "Main task completed\n";
});
```

Run it:
```bash
php -d extension=target/release/libasync_php.so hello_async.php
```

Output:
```
Main task started
All tasks spawned
Task 1: Starting
Task 2: Starting
Task 3: Starting
Task 1: Completed
Task 2: Completed
Task 3: Completed
Main task completed
```

## Timers and Delays

### Sleep
```php
// Sleep for specified milliseconds
Time::sleep(1000);
```

### After
```php
// Get a future that resolves after delay
$future = Time::after(500);
Fiber::suspend($future);
```

### Timer Callback
```php
// Schedule a callback to run after delay
Time::timer(1.5, function() {
    echo "Timer fired!\n";
});
```

### Ticker
```php
// Create a ticker that fires every 100ms
$ticker = Time::createTicker(100);

for ($i = 0; $i < 5; $i++) {
    $ticker->nextTick();
    echo "Tick $i\n";
}

$ticker->stop();
```

## Error Handling

```php
Kernel::run(function () {
    try {
        go(function() {
            throw new Exception("Task failed!");
        });
    } catch (Exception $e) {
        echo "Caught: {$e->getMessage()}\n";
    }
});
```

## Context Management

Store data in coroutine-local storage:

```php
use Async\Context;

Kernel::run(function () {
    // Set a value
    Context::set('user_id', 123);

    go(function() {
        // Each coroutine has its own context
        $userId = Context::get('user_id');
        echo "User ID: $userId\n";
    });
});
```

## Best Practices

### 1. Always Use Kernel::run()
```php
// ✅ Good
Kernel::run(function () {
    // Async code
});

// ❌ Bad - No event loop
go(function () {
    // This won't work!
});
```

### 2. Don't Block the Event Loop
```php
// ❌ Bad - Blocks event loop
sleep(1); // PHP's sleep blocks

// ✅ Good - Non-blocking
Time::sleep(1000); // Async-PHP's sleep
```

### 3. Use go() for Concurrent Tasks
```php
// ✅ Good - Concurrent
go(fn() => fetchUser(1));
go(fn() => fetchUser(2));

// ❌ Bad - Sequential
fetchUser(1);
fetchUser(2);
```

### 4. Handle Errors Properly
```php
go(function() {
    try {
        $result = riskyOperation();
    } catch (Exception $e) {
        logger()->error($e->getMessage());
    }
});
```

## Common Patterns

### Wait for Multiple Tasks
```php
$done = 0;
$total = 3;

for ($i = 0; $i < $total; $i++) {
    go(function() use (&$done) {
        // Do work
        Time::sleep(100);
        $done++;
    });
}

// Wait for all tasks
while ($done < $total) {
    Time::sleep(10);
}

echo "All tasks completed!\n";
```

### Channel Communication
```php
use Async\Channel;

Kernel::run(function () {
    $ch = Channel::make(10);

    // Producer
    go(function() use ($ch) {
        for ($i = 0; $i < 5; $i++) {
            $ch->send($i);
        }
        $ch->close();
    });

    // Consumer
    go(function() use ($ch) {
        while (($val = $ch->receive()) !== null) {
            echo "Received: $val\n";
        }
    });
});
```

## Example: Concurrent HTTP Requests

```php
use Async\Network\Http\Client;

Kernel::run(function () {
    $urls = [
        'https://api.github.com/users/github',
        'https://api.github.com/users/microsoft',
        'https://api.github.com/users/google',
    ];

    $results = [];

    foreach ($urls as $url) {
        go(function() use ($url, &$results) {
            $client = Client::new();
            $response = $client->get($url);
            $results[] = $response->json();
        });
    }

    // Wait for all requests
    Time::sleep(2000);

    foreach ($results as $data) {
        echo "User: {$data['login']}\n";
    }
});
```

## Next Steps

- [HTTP Client Tutorial](02-http-client.md) - Learn async HTTP requests
- [HTTP Server Tutorial](03-http-server.md) - Build async web servers
- [Database Tutorial](04-pdo-database.md) - Use async databases
