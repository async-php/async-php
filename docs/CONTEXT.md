# 协程 Context 上下文功能

类似 Swoole 的协程 Context 实现，为每个协程提供独立的上下文存储空间。

## 功能特性

- ✅ **全局递增的 Fiber ID**：每个协程都有唯一的 ID（从 1 开始递增）
- ✅ **协程隔离的 Context**：每个协程有独立的上下文存储，互不影响
- ✅ **自动生命周期管理**：Context 在协程结束时自动清理
- ✅ **可扩展性**：基于 `ArrayObject`，可以存储任意类型的数据

## API 文档

### Async\Coroutine 类

#### `Coroutine::getCid(): int`

获取当前协程的 ID。如果在协程外调用则返回 0。

```php
use Async\Coroutine;

go(function() {
    $cid = Coroutine::getCid();
    echo "Current coroutine ID: $cid\n";
});
```

#### `Coroutine::getPeakCount(): int`

获取自脚本启动以来创建的协程总数（峰值计数）。

```php
use Async\Coroutine;

$count = Coroutine::getPeakCount();
echo "Total coroutines created: $count\n";
```

### Async\Context 类

#### `Context::get(int $cid = 0): ArrayObject`

获取指定协程的上下文。如果 `$cid` 为 0，则获取当前协程的上下文。

```php
use Async\Context;

go(function() {
    $context = Context::get();
    $context['user_id'] = 123;
    $context['request_id'] = uniqid();

    // 在嵌套调用中访问同一个 context
    someFunction(); // 内部可以通过 Context::get() 获取相同数据
});
```

#### `Context::has(int $cid = 0): bool`

检查指定协程是否有上下文。

```php
if (Context::has()) {
    echo "Current coroutine has context\n";
}
```

#### `Context::clear(int $cid = 0): void`

清除指定协程的上下文。协程结束时会自动清理，一般不需要手动调用。

```php
Context::clear(); // 清除当前协程的 context
```

#### `Context::stats(): array`

获取 Context 的统计信息。

```php
$stats = Context::stats();
echo "Active contexts: {$stats['count']}\n";
echo "Memory usage: {$stats['memory_usage']} bytes\n";
```

## 使用示例

### 示例 1：基本用法

```php
use Async\Context;
use Async\Coroutine;

go(function() {
    $cid = Coroutine::getCid();
    echo "Coroutine ID: $cid\n";

    // 存储数据到 context
    $context = Context::get();
    $context['name'] = 'Alice';
    $context['age'] = 30;

    // 读取数据
    echo "Name: {$context['name']}\n";
    echo "Age: {$context['age']}\n";
});
```

### 示例 2：协程隔离

不同协程的 Context 是完全隔离的：

```php
use Async\Context;

go(function() {
    $context = Context::get();
    $context['data'] = 'Coroutine A';

    // 等待一段时间
    Fiber::suspend(Time::sleep(100));

    $ctx = Context::get();
    echo "Data: {$ctx['data']}\n"; // 输出：Data: Coroutine A
});

go(function() {
    $context = Context::get();
    $context['data'] = 'Coroutine B';

    Fiber::suspend(Time::sleep(100));

    $ctx = Context::get();
    echo "Data: {$ctx['data']}\n"; // 输出：Data: Coroutine B
});
```

### 示例 3：在嵌套函数中访问 Context

Context 在整个协程的调用链中都可以访问：

```php
use Async\Context;
use Async\Coroutine;

function processRequest() {
    $ctx = Context::get();
    echo "Processing request {$ctx['request_id']} for user {$ctx['user_id']}\n";

    validateRequest();
}

function validateRequest() {
    $ctx = Context::get();
    // 在嵌套调用中仍然可以访问同一个 context
    echo "Validating request {$ctx['request_id']}\n";
}

go(function() {
    $context = Context::get();
    $context['user_id'] = 123;
    $context['request_id'] = 'req-' . uniqid();

    processRequest();
});
```

### 示例 4：HTTP 请求追踪

在 HTTP 服务器中使用 Context 追踪请求：

```php
use Async\Context;
use Async\Network\Http\Server;

$server = Server::bind('0.0.0.0:8080');

while ($conn = $server->accept()) {
    go(function() use ($conn) {
        // 为每个请求设置 context
        $context = Context::get();
        $context['request_id'] = uniqid('req_');
        $context['start_time'] = microtime(true);
        $context['client_ip'] = $conn->getPeerAddr();

        // 处理请求
        handleRequest($conn);

        // 记录响应时间
        $duration = microtime(true) - $context['start_time'];
        echo "Request {$context['request_id']} completed in {$duration}s\n";
    });
}

function handleRequest($conn) {
    $ctx = Context::get();

    // 在任何嵌套函数中都能访问请求信息
    logger("Handling request {$ctx['request_id']} from {$ctx['client_ip']}");

    // ... 处理请求逻辑
}

function logger($message) {
    $ctx = Context::get();
    echo "[{$ctx['request_id']}] $message\n";
}
```

## 与 Swoole 的对比

| 特性 | Swoole | Async-PHP |
|------|--------|-----------|
| 获取协程 ID | `Swoole\Coroutine::getCid()` | `Async\Coroutine::getCid()` |
| 获取 Context | `Swoole\Coroutine::getContext()` | `Async\Context::get()` |
| Context 类型 | `Swoole\Coroutine\Context` | `ArrayObject` |
| 自动清理 | ✅ | ✅ |
| 协程隔离 | ✅ | ✅ |

## 实现原理

1. **Fiber ID 生成**：使用 Rust 的 `AtomicU64` 实现线程安全的全局递增计数器
2. **ID 存储**：使用 `tokio::task_local!` ���存储当前协程的 ID
3. **Context 管理**：在 PHP 层使用静态数组 `$contexts[cid]` 存储每个协程的 Context
4. **生命周期**：每个 Fiber 在执行时会被分配一个唯一 ID，Context 由 PHP 层管理生命周期

## 注意事项

1. 在协程外调用 `Coroutine::getCid()` 会返回 0
2. Context 是基于 `ArrayObject` 的，可以像数组一样使用
3. 协程结束后需要手动清理 Context，或者依赖 PHP 的垃圾回收（建议在合适的地方调用 `Context::clear()`）
4. 峰值计数 `getPeakCount()` 只会增加，不会因为协程结束而减少

## 测试

运行测试文件：

```bash
php -d extension=target/release/libasync_php.dylib test_context.php
```

测试覆盖：
- ✅ 基本的 Coroutine ID 分配
- ✅ 多个协程的 ID 唯一性
- ✅ Context 隔离性
- ✅ 嵌套函数调用中的 Context 访问
- ✅ Context 统计信息
- ✅ Context 生命周期管理
