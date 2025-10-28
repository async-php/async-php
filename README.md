# Async PHP

基于 Rust Tokio 的 PHP 异步运行时与网络/数据库工具集，通过 Fiber 驱动异步 Future，让常见 IO 能以同步写法并发执行。

## 环境要求
- PHP 8.2+（启用 Fiber），具备 `php-config`
- Rust stable + Cargo
- Composer（用于自动加载 `src-php` 里的用户态封装）
- 需要对应的系统依赖：MySQL/PostgreSQL 客户端库、OpenSSL/系统证书（HTTPs/TLS）、可选的 `disable_functions` 配置以替换内置阻塞函数

## 构建与安装扩展
1. 编译：`cargo build --release`
2. 找到生成的扩展：`target/release/libasync_php.so`（macOS 可能是 `.dylib`，亦会生成 `.so`）
3. 复制到 PHP 扩展目录：
   ```bash
   cp target/release/libasync_php.so "$(php-config --extension-dir)"
   ```
4. 在 `php.ini` 中启用：
   ```ini
   extension=async_php
   ```
5. 重新加载 PHP-FPM/CLI。验证：`php -m | grep async_php`

> 如需用异步版本的 `sleep/usleep/file_get_contents/file_put_contents`，请在 `php.ini` 中禁用原生实现，例如：
> `disable_functions = sleep,usleep,file_get_contents,file_put_contents`

## PHP 侧准备
```bash
composer install    # 若尚未安装依赖
```
在入口文件中加载：
```php
require __DIR__.'/vendor/autoload.php';
```

## 快速开始
```php
<?php
require __DIR__.'/vendor/autoload.php';

use Async\Kernel;
use Async\Time;

Kernel::run(function () {
    // 启动一个并发任务
    Kernel::spawn(function () {
        Time::sleep(500);
        echo "child done\n";
    });

    // 当前 Fiber 内继续执行
    for ($i = 0; $i < 3; $i++) {
        Time::sleep(200);
        echo "tick {$i}\n";
    }
});
```

## 核心 API
- `Kernel::run(callable $main)`：启动事件循环并执行主 Fiber。
- `Kernel::spawn(callable $task)` / 全局 `go()`：并发执行新的 Fiber。
- `Kernel::setupLog(['level' => 'debug', 'ansi' => 'true'])`：初始化 Rust 侧日志。
- `Time::sleep(int $ms)` / `Time::after(float $seconds, callable $cb)` / `Time::createTicker(int $ms)`：定时工具。
- `Channel::push($data)` / `Channel::pop()`：无锁通道，用于 Fiber 间通信。

## 网络
- TCP：`Async\Network\Tcp\Server::bind($addr)` 接受连接；`Async\Network\Tcp\Socket::connect($addr)` 读写字符串。
- UDP：`Async\Network\Udp\Socket::bind($addr)`，提供 `sendTo/recvFrom`。
- Unix Socket：`Async\Network\Unix\Server/Socket`。
- TLS：`Async\Network\Tcp\TlsSocket::connect($addr)`（基于 rustls）。
- HTTP Server：`Async\Network\Http\Server::listen($addr, callable $handler, array $config = [])`，处理器接收 `Request`，返回或修改 `Response`，支持 `initStream()/write()/end()` 流式响应。
- HTTP Client：`Async\Network\Http\Client::request($method, $url, $headers = null, $body = null)`，返回 `['status' => int, 'body' => string, 'headers' => array]`。

启用协程化的 Stream Wrapper 可让 `fopen/stream_socket_client` 等函数走异步通道：
```php
Kernel::enableCoroutine(
    Kernel::HOOK_TCP | Kernel::HOOK_UDP | Kernel::HOOK_UNIX |
    Kernel::HOOK_SSL | Kernel::HOOK_FILE | Kernel::HOOK_HTTP
);
```

## 文件系统
`Async\FileSystem` 提供常见操作：`getContents/putContents/exists/isFile/isDir/unlink/rename/copy/mkdir/rmdir/size/scandir`，均在 Fiber 中挂起等待。

## 数据库
- MySQL：`Async\Database\MySQL::connect($dsn, $maxConnections = 10)`，提供 `query/execute/beginTransaction`，事务对象支持 `query/execute/commit/rollback`。
- PostgreSQL：`Async\Database\PostgreSQL` 接口同上。
DSN 采用 SQLx 格式，例如 `mysql://user:pass@127.0.0.1:3306/dbname`。

## 示例
目录 `examples/` 覆盖了常见场景：
- `test.php`：Fiber 基础与定时器
- `server.php`：TCP Echo 服务
- `http_server.php`、`http_new_test.php`：HTTP 服务器与流式响应
- `tcp_test.php`、`udp_test.php`、`unix_test.php`、`tls_test.php`：各类网络操作
- `file_test.php`、`test_timer.php` 等

运行示例（需已加载扩展）：`php examples/server.php`

## 调试与日志
Rust 侧日志可通过 `Kernel::setupLog` 配置等级，默认 `info`。建议在开发时设置 `level => debug` 并开启 `ansi => true` 以获得彩色输出。

## 已知注意事项
- 目前 HTTP/3 配置项已预留（`enable_http3`），但实现尚未完成。
- Stream Wrapper Hooks 会全局影响对应协议的处理，建议仅在 CLI 或可控环境启用。
- 扩展在 tokio 当前线程运行，阻塞型用户代码仍会卡住事件循环。
