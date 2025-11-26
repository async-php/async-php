# Async-PHP Tutorials

Comprehensive tutorials for learning async PHP programming with this extension.

## Getting Started

1. **[Introduction](00-introduction.md)** - Overview of Async-PHP
   - What is Async-PHP?
   - Architecture and features
   - Performance characteristics
   - Installation guide

2. **[Getting Started](01-getting-started.md)** - Your first async program
   - Basic concepts (Kernel, coroutines, event loop)
   - Writing your first program
   - Timers and delays
   - Error handling
   - Common patterns

## Core Features

3. **[HTTP Client](02-http-client.md)** - Making async HTTP requests
   - All HTTP methods
   - Request/response handling
   - Advanced features (headers, cookies, timeouts, TLS)
   - Concurrent requests
   - Streaming
   - Real-world examples

4. **[HTTP Server](03-http-server.md)** - Building async web servers
   - Zero-copy HTTP/1.1, HTTP/2, and HTTP/3 server
   - QUIC protocol support (HTTP/3)
   - Request routing
   - Middleware support (PSR-15)
   - REST API example
   - Best practices

5. **[PDO Database](04-pdo-database.md)** - Async database operations
   - MySQL and PostgreSQL support
   - Connection pooling
   - Prepared statements
   - Transactions
   - Real-world examples

## Learning Path

### For Beginners
1. Read [Introduction](00-introduction.md) to understand the project
2. Follow [Getting Started](01-getting-started.md) to write your first program
3. Try examples from `examples/` directory

### For Web Developers
1. Learn [HTTP Client](02-http-client.md) for API consumption
2. Learn [HTTP Server](03-http-server.md) for building APIs
3. Integrate [PDO Database](04-pdo-database.md) for data persistence

### For Advanced Users
1. Study concurrent patterns in [Getting Started](01-getting-started.md)
2. Review performance tips in all tutorials
3. Explore source code in `src-php/` and `src/`

## Examples Directory

The `examples/` directory contains working code samples:

### Basic Examples
- `test.php` - Basic async/coroutines demo
- `test_timer.php` - Timer examples
- `context_test.php` - Context API usage

### Network Examples
- `tcp_test.php` - TCP client/server
- `udp_test.php` - UDP sockets
- `unix_test.php` - Unix domain sockets
- `tls_test.php` - TLS/SSL connections

### HTTP Client Examples
- `http_client_demo.php` - Basic HTTP client
- `http_client_test.php` - Advanced features
- `http_streaming_client_test.php` - Streaming responses

### HTTP Server Examples
- `http_test_simple.php` - Simple HTTP server
- `test_http_server.php` - Zero-copy server
- `http_server_zero_copy.php` - Performance demo
- `http3_server_test.php` - HTTP/3 server with QUIC
- `psr15_middleware.php` - PSR-15 middleware

### Database Examples
- `pdo_live_test.php` - Basic PDO operations
- `pdo_pool_test.php` - Connection pooling
- `pdo_binding_test.php` - Parameter binding (16+ test cases)

## Quick Reference

### Starting the Event Loop
```php
use Async\Kernel;

Kernel::run(function () {
    // Your async code here
});
```

### Spawning Concurrent Tasks
```php
go(function () {
    // Task code
});
```

### Async Sleep
```php
use Async\Time;

Time::sleep(1000); // 1 second
```

### HTTP Request
```php
use Async\Network\Http\Client;

$client = Client::new();
$response = $client->get('https://example.com');
echo $response->text();
```

### HTTP Server
```php
use Async\Network\Http\Server;
use Async\Network\Tcp\Listener;

$server = new Server();
$listener = Listener::bind('127.0.0.1:8080');

while (true) {
    $conn = $listener->accept();
    go(fn() => $server->serve($conn, $handler));
}
```

### Database Query
```php
$pdo = new \PDO\PDO('mysql:host=localhost;dbname=test', 'user', 'pass');
$stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');
$stmt->execute([1]);
$user = $stmt->fetch();
```

## Contributing

Found an issue in the tutorials? Please open an issue or submit a pull request!

## License

Same as the main project license.
