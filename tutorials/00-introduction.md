# Async-PHP Introduction

## What is Async-PHP?

Async-PHP is a high-performance async PHP extension built with Rust. It brings async/await capabilities to PHP using Fibers, providing:

- **Async I/O** - Non-blocking network and file operations
- **HTTP Client** - Full-featured HTTP/1.1, HTTP/2 client with connection pooling
- **HTTP Server** - Zero-copy HTTP server with HTTP/1.1, HTTP/2, and HTTP/3 (QUIC) support
- **PDO Database** - Async PDO implementation for MySQL and PostgreSQL
- **Network** - TCP, UDP, Unix sockets with TLS support
- **Concurrency** - Lightweight coroutines using PHP Fibers

## Architecture

```
┌─────────────────────────────────────┐
│        PHP User Code                │
│  (Fiber-based async/await)          │
├─────────────────────────────────────┤
│     PHP Wrapper Classes             │
│  (Async\Network\Http\Client, etc)   │
├─────────────────────────────────────┤
│      Rust Extension                 │
│  (Tokio async runtime + SQLx)       │
└─────────────────────────────────────┘
```

## Key Features

### 1. Async I/O with Fibers
```php
use Async\Kernel;

Kernel::run(function () {
    // Multiple concurrent operations
    go(function () {
        echo "Task 1 started\n";
        Async\Time::sleep(100);
        echo "Task 1 completed\n";
    });

    go(function () {
        echo "Task 2 started\n";
        Async\Time::sleep(50);
        echo "Task 2 completed\n";
    });
});
```

### 2. HTTP Client
```php
$client = Client::new();
$response = $client->get('https://api.github.com/users/github');
$data = $response->json();
```

### 3. HTTP Server
```php
$server = new Server();
$listener = Listener::bind('127.0.0.1:8080');

while (true) {
    $conn = $listener->accept();
    go(fn() => $server->serve($conn, $handler));
}
```

### 4. Async PDO
```php
$pdo = new \PDO\PDO('mysql:host=localhost;dbname=test', 'user', 'pass');
$stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');
$stmt->execute([1]);
$user = $stmt->fetch();
```

### 5. Network Programming
```php
// TCP Server
$listener = TcpListener::bind('127.0.0.1:8080');
while (true) {
    $conn = $listener->accept();
    go(function() use ($conn) {
        $data = $conn->read(1024);
        $conn->write("Echo: $data");
    });
}
```

## Performance

Async-PHP is built with Rust and Tokio, providing:

- **High throughput** - Handles thousands of concurrent connections
- **Low latency** - Native Rust performance for I/O operations
- **Memory efficient** - Lightweight coroutines using PHP Fibers
- **Zero-copy** - Efficient data transfer without unnecessary copies

## Installation

### Requirements
- PHP 8.1+ with Fiber support
- Rust toolchain (for building from source)
- macOS or Linux

### Build from Source
```bash
git clone https://github.com/your-org/async-php.git
cd async-php
cargo build --release
composer install
```

### Load Extension
```bash
php -d extension=target/release/libasync_php.so your_script.php
```

## Next Steps

- [Getting Started](01-getting-started.md) - Write your first async PHP program
- [HTTP Client](02-http-client.md) - Learn to make async HTTP requests
- [HTTP Server](03-http-server.md) - Build async HTTP servers
- [PDO Database](04-pdo-database.md) - Use async database connections
- [Network Programming](05-network.md) - TCP, UDP, and TLS
- [Async Patterns](06-async-patterns.md) - Advanced async programming patterns
