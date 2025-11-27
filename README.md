# Async-PHP

[![Version](https://img.shields.io/badge/version-0.4.0-blue.svg)](https://github.com/your-org/async-php)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![PHP](https://img.shields.io/badge/php-%3E%3D8.1-purple.svg)](https://www.php.net/)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

A high-performance async runtime for PHP, built with Rust and powered by Tokio. Brings true async/await capabilities to PHP using Fibers.

English | [简体中文](README.zh-CN.md)

## ✨ Features

### 🚀 **Async I/O**
- Non-blocking file and network operations
- Lightweight coroutines using PHP 8.1+ Fibers
- Built on Tokio async runtime for maximum performance

### 🌐 **HTTP Client & Server**
- **HTTP/1.1, HTTP/2, and HTTP/3 (QUIC)** support
- Zero-copy I/O optimization
- Connection pooling
- Streaming request/response bodies
- PSR-15 middleware support

### 💾 **Async Database (PDO)**
- Full PDO compatibility with async operations
- MySQL and PostgreSQL support
- Connection pooling
- Prepared statements with parameter binding
- Transaction support

### 🔌 **Network Programming**
- TCP, UDP, Unix domain sockets
- TLS/SSL support with custom certificates
- QUIC protocol (for HTTP/3)
- Non-blocking socket operations

### ⚡ **Advanced Features**
- Channels for inter-coroutine communication
- Timers and delays
- Context API for coroutine-local storage
- Concurrent task execution

## 📦 Installation

### Prerequisites

- PHP 8.1 or higher with Fibers support
- Rust 1.70 or higher
- Cargo (Rust package manager)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-org/async-php.git
cd async-php

# Build the extension (release mode)
cargo build --release

# The compiled extension will be at:
# target/release/libasync_php.dylib (macOS)
# target/release/libasync_php.so (Linux)
# target/release/async_php.dll (Windows)
```

### Load the Extension

```bash
# Run PHP with the extension loaded
php -d extension=target/release/libasync_php.dylib your_script.php

# Or add to php.ini
extension=path/to/libasync_php.dylib
```

## 🚀 Quick Start

### Hello World

```php
<?php
use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    echo "Starting...\n";

    // Non-blocking sleep
    Time::sleep(1000); // 1 second

    echo "Done!\n";
});
```

### Concurrent Tasks

```php
<?php
use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    // Spawn concurrent tasks
    go(function () {
        echo "Task 1 started\n";
        Time::sleep(500);
        echo "Task 1 completed\n";
    });

    go(function () {
        echo "Task 2 started\n";
        Time::sleep(300);
        echo "Task 2 completed\n";
    });

    // Wait for tasks to complete
    Time::sleep(1000);
});
```

### HTTP Client

```php
<?php
use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    $client = Client::new();

    // Make async HTTP request
    $response = $client->get('https://api.github.com/users/github');

    echo "Status: " . $response->status() . "\n";
    echo "Body: " . $response->text() . "\n";
});
```

### HTTP Server

```php
<?php
use Async\Kernel;
use Async\Network\Http\Server;
use Async\Network\Tcp\Listener;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

Kernel::run(function () {
    $server = new Server();
    $listener = Listener::bind('127.0.0.1:8080');

    echo "Server listening on http://127.0.0.1:8080\n";

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            $server->serve($conn, function(HttpRequest $req): HttpResponse {
                $resp = new HttpResponse();
                $resp->setStatus(200);
                $resp->setHeader('Content-Type', 'text/plain');
                $resp->setBody('Hello, World!');
                return $resp;
            });
        });
    }
});
```

### HTTP/3 Server (QUIC)

```php
<?php
use Async\Kernel;
use Async\Kernel\Network\Quic\QuicListener;
use Async\Kernel\Network\Http\Http3Server;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

Kernel::run(function () {
    // HTTP/3 requires TLS certificates
    $listener = QuicListener::bind('127.0.0.1:4433', 'cert.pem', 'key.pem');
    $server = new Http3Server();

    echo "HTTP/3 server listening on https://127.0.0.1:4433\n";

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            $server->serve($conn, function(HttpRequest $req): HttpResponse {
                $resp = new HttpResponse();
                $resp->setStatus(200);
                $resp->setHeader('Content-Type', 'text/plain');
                $resp->setBody('Hello from HTTP/3!');
                return $resp;
            });
        });
    }
});
```

### Async Database (PDO)

```php
<?php
use Async\Kernel;

Kernel::run(function () {
    // Create async PDO connection
    $pdo = new \PDO\PDO(
        'mysql:host=localhost;dbname=test',
        'username',
        'password'
    );

    // Execute queries asynchronously
    $stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');
    $stmt->execute([1]);
    $user = $stmt->fetch();

    echo "User: {$user['name']}\n";
});
```

## 📚 Documentation

Comprehensive tutorials and guides are available in the [tutorials/](tutorials/) directory:

- **[Introduction](tutorials/00-introduction.md)** - Overview and architecture
- **[Getting Started](tutorials/01-getting-started.md)** - Your first async program
- **[HTTP Client](tutorials/02-http-client.md)** - Making async HTTP requests
- **[HTTP Server](tutorials/03-http-server.md)** - Building async web servers
- **[PDO Database](tutorials/04-pdo-database.md)** - Async database operations

### Quick Links

- [Examples Directory](examples/) - Working code samples
- [Tutorial README](tutorials/README.md) - Learning paths and quick reference

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│        PHP User Code                │
│  (Fiber-based async/await)          │
├─────────────────────────────────────┤
│     PHP Wrapper Classes             │
│  (Async\Network\Http\Client, etc)   │
├─────────────────────────────────────┤
│      Rust Extension (ext-php-rs)    │
│  - Tokio async runtime              │
│  - SQLx for database                │
│  - Hyper for HTTP/1.1 & HTTP/2      │
│  - Quinn + h3 for HTTP/3            │
│  - Reqwest for HTTP client          │
└─────────────────────────────────────┘
```

## ⚡ Performance

Async-PHP is built with performance in mind:

- **High Throughput**: Handle thousands of concurrent connections
- **Low Latency**: Native Rust performance for I/O operations
- **Memory Efficient**: Lightweight coroutines using PHP Fibers
- **Zero-Copy**: Efficient data transfer without unnecessary copies
- **Connection Pooling**: Reuse connections for databases and HTTP clients

## 🧪 Testing

Run the examples to test the functionality:

```bash
# HTTP client test
php -d extension=target/release/libasync_php.dylib examples/http_client_test.php

# HTTP server test
php -d extension=target/release/libasync_php.dylib examples/http_test_simple.php

# HTTP/3 server test (requires TLS certificates)
php -d extension=target/release/libasync_php.dylib examples/http3_server_test.php

# PDO test (requires database)
php -d extension=target/release/libasync_php.dylib examples/pdo_live_test.php
```

## 🔧 Development

### Project Structure

```
async-php/
├── src/                    # Rust source code
│   ├── http/              # HTTP client & server
│   ├── net/               # Network (TCP, UDP, TLS, QUIC)
│   ├── pdo/               # Async PDO implementation
│   ├── io/                # I/O abstractions
│   └── lib.rs             # Main entry point
├── src-php/               # PHP wrapper classes
│   ├── Async/             # Main namespace
│   ├── PDO/               # PDO namespace
│   └── autoload.php       # Autoloader
├── examples/              # Working examples
├── tutorials/             # Documentation
├── Cargo.toml             # Rust dependencies
└── README.md              # This file
```

### Building

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized runtime)
cargo build --release

# Run tests
cargo test

# Check code without building
cargo check
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Guidelines

1. Follow existing code style
2. Add tests for new features
3. Update documentation
4. Ensure all tests pass

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

This project is built on top of excellent open-source libraries:

- [Tokio](https://tokio.rs/) - Async runtime for Rust
- [ext-php-rs](https://github.com/davidcole1340/ext-php-rs) - PHP extension framework for Rust
- [Hyper](https://hyper.rs/) - HTTP implementation
- [Quinn](https://github.com/quinn-rs/quinn) - QUIC protocol implementation
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/async-php/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/async-php/discussions)

## 🗺️ Roadmap

- [ ] WebSocket support
- [ ] gRPC support
- [ ] More database drivers (SQLite, Redis)
- [ ] Async file I/O improvements
- [ ] Performance benchmarks
- [ ] More comprehensive examples

---

Made with ❤️ using Rust and PHP
