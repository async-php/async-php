# Async-PHP

[![版本](https://img.shields.io/badge/版本-0.4.0-blue.svg)](https://github.com/your-org/async-php)
[![许可证](https://img.shields.io/badge/许可证-MIT-green.svg)](LICENSE)
[![PHP](https://img.shields.io/badge/php-%3E%3D8.1-purple.svg)](https://www.php.net/)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

基于 Rust 和 Tokio 构建的高性能 PHP 异步运行时。使用 Fibers 为 PHP 带来真正的 async/await 能力。

[English](README.md) | 简体中文

## ✨ 特性

### 🚀 **异步 I/O**
- 非阻塞文件和网络操作
- 使用 PHP 8.1+ Fibers 的轻量级协程
- 基于 Tokio 异步运行时，性能强劲

### 🌐 **HTTP 客户端和服务器**
- **支持 HTTP/1.1、HTTP/2 和 HTTP/3 (QUIC)**
- 零拷贝 I/O 优化
- 连接池
- 流式请求/响应体
- PSR-15 中间件支持

### 💾 **异步数据库 (PDO)**
- 完全兼容 PDO，支持异步操作
- 支持 MySQL 和 PostgreSQL
- 连接池
- 参数绑定的预处理语句
- 事务支持

### 🔌 **网络编程**
- TCP、UDP、Unix 域套接字
- 支持 TLS/SSL 和自定义证书
- QUIC 协议（用于 HTTP/3）
- 非阻塞套接字操作

### ⚡ **高级特性**
- 协程间通信的 Channel
- 定时器和延迟
- 协程本地存储的 Context API
- 并发任务执行

## 📦 安装

### 前置要求

- PHP 8.1 或更高版本，支持 Fibers
- Rust 1.70 或更高版本
- Cargo（Rust 包管理器）

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-org/async-php.git
cd async-php

# 构建扩展（发布模式）
cargo build --release

# 编译后的扩展位置：
# target/release/libasync_php.dylib (macOS)
# target/release/libasync_php.so (Linux)
# target/release/async_php.dll (Windows)
```

### 加载扩展

```bash
# 运行 PHP 时加载扩展
php -d extension=target/release/libasync_php.dylib your_script.php

# 或添加到 php.ini
extension=path/to/libasync_php.dylib
```

## 🚀 快速开始

### Hello World

```php
<?php
use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    echo "开始...\n";

    // 非阻塞睡眠
    Time::sleep(1000); // 1 秒

    echo "完成！\n";
});
```

### 并发任务

```php
<?php
use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    // 启动并发任务
    go(function () {
        echo "任务 1 开始\n";
        Time::sleep(500);
        echo "任务 1 完成\n";
    });

    go(function () {
        echo "任务 2 开始\n";
        Time::sleep(300);
        echo "任务 2 完成\n";
    });

    // 等待任务完成
    Time::sleep(1000);
});
```

### HTTP 客户端

```php
<?php
use Async\Kernel;
use Async\Network\Http\Client;

Kernel::run(function () {
    $client = Client::new();

    // 发起异步 HTTP 请求
    $response = $client->get('https://api.github.com/users/github');

    echo "状态码: " . $response->status() . "\n";
    echo "响应体: " . $response->text() . "\n";
});
```

### HTTP 服务器

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

    echo "服务器监听 http://127.0.0.1:8080\n";

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            $server->serve($conn, function(HttpRequest $req): HttpResponse {
                $resp = new HttpResponse();
                $resp->setStatus(200);
                $resp->setHeader('Content-Type', 'text/plain');
                $resp->setBody('你好，世界！');
                return $resp;
            });
        });
    }
});
```

### HTTP/3 服务器 (QUIC)

```php
<?php
use Async\Kernel;
use Async\Kernel\Network\Quic\QuicListener;
use Async\Kernel\Network\Http\Http3Server;
use Async\Kernel\Network\Http\HttpRequest;
use Async\Kernel\Network\Http\HttpResponse;

Kernel::run(function () {
    // HTTP/3 需要 TLS 证书
    $listener = QuicListener::bind('127.0.0.1:4433', 'cert.pem', 'key.pem');
    $server = new Http3Server();

    echo "HTTP/3 服务器监听 https://127.0.0.1:4433\n";

    while (true) {
        $conn = $listener->accept();

        go(function() use ($server, $conn) {
            $server->serve($conn, function(HttpRequest $req): HttpResponse {
                $resp = new HttpResponse();
                $resp->setStatus(200);
                $resp->setHeader('Content-Type', 'text/plain');
                $resp->setBody('来自 HTTP/3 的问候！');
                return $resp;
            });
        });
    }
});
```

### 异步数据库 (PDO)

```php
<?php
use Async\Kernel;

Kernel::run(function () {
    // 创建异步 PDO 连接
    $pdo = new \PDO\PDO(
        'mysql:host=localhost;dbname=test',
        'username',
        'password'
    );

    // 异步执行查询
    $stmt = $pdo->prepare('SELECT * FROM users WHERE id = ?');
    $stmt->execute([1]);
    $user = $stmt->fetch();

    echo "用户: {$user['name']}\n";
});
```

## 📚 文档

[tutorials/](tutorials/) 目录中提供了全面的教程和指南：

- **[简介](tutorials/00-introduction.md)** - 概述和架构
- **[入门指南](tutorials/01-getting-started.md)** - 第一个异步程序
- **[HTTP 客户端](tutorials/02-http-client.md)** - 发起异步 HTTP 请求
- **[HTTP 服务器](tutorials/03-http-server.md)** - 构建异步 Web 服务器
- **[PDO 数据库](tutorials/04-pdo-database.md)** - 异步数据库操作

### 快速链接

- [示例目录](examples/) - 可运行的代码示例
- [教程 README](tutorials/README.md) - 学习路径和快速参考

## 🏗️ 架构

```
┌─────────────────────────────────────┐
│        PHP 用户代码                  │
│  (基于 Fiber 的 async/await)        │
├─────────────────────────────────────┤
│     PHP 包装类                       │
│  (Async\Network\Http\Client 等)     │
├─────────────────────────────────────┤
│      Rust 扩展 (ext-php-rs)         │
│  - Tokio 异步运行时                  │
│  - SQLx 数据库                       │
│  - Hyper (HTTP/1.1 & HTTP/2)        │
│  - Quinn + h3 (HTTP/3)              │
│  - Reqwest (HTTP 客户端)            │
└─────────────────────────────────────┘
```

## ⚡ 性能

Async-PHP 专注性能设计：

- **高吞吐量**：处理数千个并发连接
- **低延迟**：原生 Rust 性能的 I/O 操作
- **内存高效**：使用 PHP Fibers 的轻量级协程
- **零拷贝**：无需不必要复制的高效数据传输
- **连接池**：复用数据库和 HTTP 客户端连接

## 🧪 测试

运行示例来测试功能：

```bash
# HTTP 客户端测试
php -d extension=target/release/libasync_php.dylib examples/http_client_test.php

# HTTP 服务器测试
php -d extension=target/release/libasync_php.dylib examples/http_test_simple.php

# HTTP/3 服务器测试（需要 TLS 证书）
php -d extension=target/release/libasync_php.dylib examples/http3_server_test.php

# PDO 测试（需要数据库）
php -d extension=target/release/libasync_php.dylib examples/pdo_live_test.php
```

## 🔧 开发

### 项目结构

```
async-php/
├── src/                    # Rust 源代码
│   ├── http/              # HTTP 客户端和服务器
│   ├── net/               # 网络 (TCP, UDP, TLS, QUIC)
│   ├── pdo/               # 异步 PDO 实现
│   ├── io/                # I/O 抽象
│   └── lib.rs             # 主入口点
├── src-php/               # PHP 包装类
│   ├── Async/             # 主命名空间
│   ├── PDO/               # PDO 命名空间
│   └── autoload.php       # 自动加载器
├── examples/              # 工作示例
├── tutorials/             # 文档
├── Cargo.toml             # Rust 依赖
└── README.md              # 说明文件
```

### 构建

```bash
# 调试构建（编译快，运行慢）
cargo build

# 发布构建（编译慢，运行优化）
cargo build --release

# 运行测试
cargo test

# 检查代码但不构建
cargo check
```

## 🤝 贡献

欢迎贡献！请随时提交问题和拉取请求。

### 指南

1. 遵循现有代码风格
2. 为新功能添加测试
3. 更新文档
4. 确保所有测试通过

## 📄 许可证

本项目采用 MIT 许可证 - 详见 LICENSE 文件。

## 🙏 致谢

本项目基于优秀的开源库构建：

- [Tokio](https://tokio.rs/) - Rust 异步运行时
- [ext-php-rs](https://github.com/davidcole1340/ext-php-rs) - 用于 Rust 的 PHP 扩展框架
- [Hyper](https://hyper.rs/) - HTTP 实现
- [Quinn](https://github.com/quinn-rs/quinn) - QUIC 协议实现
- [SQLx](https://github.com/launchbadge/sqlx) - 异步 SQL 工具包
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端

## 📞 支持

- **问题**：[GitHub Issues](https://github.com/your-org/async-php/issues)
- **讨论**：[GitHub Discussions](https://github.com/your-org/async-php/discussions)

## 🗺️ 路线图

- [ ] WebSocket 支持
- [ ] gRPC 支持
- [ ] 更多数据库驱动（SQLite、Redis）
- [ ] 异步文件 I/O 改进
- [ ] 性能基准测试
- [ ] 更全面的示例

---

使用 ❤️ 以 Rust 和 PHP 构建
