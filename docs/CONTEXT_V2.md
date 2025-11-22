# 协程 Context 上下文功能（基于 Async\Fiber）

类似 Swoole 的协程 Context 实现，每个 `Async\Fiber` 都有独立的上下文存储空间。

## 功能特性

- ✅ **全局递增的 Fiber ID**：每个 AsyncFiber 都有唯一的 ID（从 1 开始递增）
- ✅ **Fiber 内置 Context**：每个 Async\Fiber 对象持有一个私有的 Context（ArrayObject）
- ✅ **自动隔离**：每个 Fiber 的 Context 完全独立，互不影响
- ✅ **引用传递**：Context 使用 ArrayObject 实现，修改会持久化
- ✅ **便捷访问**：通过 `Context::get()` 快速访问当前 Fiber 的 Context

## 核心类

### `Async\Fiber`

扩展的 Fiber 类，每个实例包含：
- 内部 PHP `\Fiber` 对象
- 唯一的协程 ID (CID)
- 私有的 Context 存储（ArrayObject）

### `Async\Context`

Context 管理类，提供静态方法访问当前 Fiber 的上下文。

## API 文档

### Async\Fiber 类

#### `new Async\Fiber(callable $callback)`

创建一个新的 AsyncFiber。

```php
$fiber = new Async\Fiber(function() {
    echo "Hello from Async Fiber!\n";
});

go($fiber); // 使用 go() 函数启动
```

#### `Fiber::getCurrentCid(): int`

获取当前协程的 ID。

```php
go(new Async\Fiber(function() {
    $cid = Async\Fiber::getCurrentCid();
    echo "Current CID: $cid\n";
}));
```

#### `Fiber::getCurrentContext(): ?ArrayObject`

获取当前 Fiber 的 Context。

```php
go(new Async\Fiber(function() {
    $ctx = Async\Fiber::getCurrentContext();
    $ctx['user_id'] = 123;
}));
```

#### `Fiber::getPeakCount(): int`

获取自脚本启动以来创建的 Fiber 总数。

```php
$count = Async\Fiber::getPeakCount();
```

### Async\Context 类

#### `Context::get(): ?ArrayObject`

获取当前 Fiber 的 Context（便捷方法）。

```php
use Async\Context;

go(new Async\Fiber(function() {
    $ctx = Context::get();
    $ctx['user'] = 'alice';
    $ctx['request_id'] = uniqid();
}));
```

#### `Context::getCid(): int`

获取当前 Fiber 的 CID。

```php
$cid = Context::getCid();
```

#### `Context::stats(): array`

获取统计信息。

```php
$stats = Context::stats();
echo "Total fibers created: {$stats['peak_count']}\n";
```

## 使用示例

### 示例 1：基本用法

```php
use Async\Context;

run(new Fiber(function() {
    $fiber = new Async\Fiber(function() {
        $cid = Context::getCid();
        echo "Fiber CID: $cid\n";

        $ctx = Context::get();
        $ctx['name'] = 'Alice';
        $ctx['age'] = 30;

        // 模拟异步操作
        Fiber::suspend(Time::sleep(100));

        // 数据仍然保持
        $ctx2 = Context::get();
        echo "Name: {$ctx2['name']}, Age: {$ctx2['age']}\n";
    });

    go($fiber);
    Fiber::suspend(Time::sleep(200));
}));
```

### 示例 2：多个 Fiber 隔离

```php
use Async\Context;

run(new Fiber(function() {
    for ($i = 1; $i <= 3; $i++) {
        $fiber = new Async\Fiber(function() use ($i) {
            $ctx = Context::get();
            $ctx['id'] = $i;
            $ctx['name'] = "Fiber-$i";

            Fiber::suspend(Time::sleep(10));

            $ctx2 = Context::get();
            echo "Fiber #{$i}: id={$ctx2['id']}, name={$ctx2['name']}\n";
        });

        go($fiber);
    }

    Fiber::suspend(Time::sleep(100));
}));
```

### 示例 3：嵌套函数调用

```php
use Async\Context;

function processRequest() {
    $ctx = Context::get();
    echo "Processing request {$ctx['request_id']} for user {$ctx['user']}\n";
    validateRequest();
}

function validateRequest() {
    $ctx = Context::get();
    echo "Validating request {$ctx['request_id']}\n";
}

run(new Fiber(function() {
    $fiber = new Async\Fiber(function() {
        $ctx = Context::get();
        $ctx['user'] = 'bob';
        $ctx['request_id'] = 'req-' . uniqid();

        processRequest(); // 嵌套调用可以访问同一个 Context
    });

    go($fiber);
    Fiber::suspend(Time::sleep(100));
}));
```

### 示例 4：HTTP 请求追踪

```php
use Async\Context;
use Async\Network\Http\Server;

run(new Fiber(function() {
    $server = Server::bind('0.0.0.0:8080');

    while ($conn = $server->accept()) {
        $fiber = new Async\Fiber(function() use ($conn) {
            // 为每个请求设置 Context
            $ctx = Context::get();
            $ctx['request_id'] = uniqid('req_');
            $ctx['start_time'] = microtime(true);

            handleRequest($conn);

            $duration = microtime(true) - $ctx['start_time'];
            echo "Request {$ctx['request_id']} completed in {$duration}s\n";
        });

        go($fiber);
    }
}));

function handleRequest($conn) {
    $ctx = Context::get();
    logger("Handling request {$ctx['request_id']}");
    // ... 处理请求
}

function logger($message) {
    $ctx = Context::get();
    echo "[{$ctx['request_id']}] $message\n";
}
```

## 实现原理

### 架构设计

1. **Async\Fiber 类**（Rust 端）：
   - 包装 PHP 的 `\Fiber` 对象
   - 创建时分配唯一的 CID
   - 持有一个 `ArrayObject` 作为 Context 存储

2. **Thread-Local 存储**（Rust 端）：
   - 使用 `tokio::task_local!` 存储当前 Fiber ID
   - 使用 `thread_local!` 存储 CID → Context 的映射

3. **Context 类**（PHP 端）：
   - 提供便捷的静态方法访问 API
   - 调用 `Async\Fiber` 的静态方法获取数据

### 关键实现细节

1. **CID 生成**：使用 `AtomicU64` 全局计数器，线程安全
2. **Context 存储**：ArrayObject 对象，引用传递，修改会持久化
3. **Fiber 驱动**：通过 `go()` 函数启动，自动设置 `CURRENT_FIBER_ID`
4. **生命周期**：Fiber 销毁时自动清理 Context（Drop trait）

## 与 Swoole 的对比

| 特性 | Swoole | Async-PHP |
|------|--------|-----------|
| 获取协程 ID | `Swoole\Coroutine::getCid()` | `Async\Fiber::getCurrentCid()` 或 `Context::getCid()` |
| 获取 Context | `Swoole\Coroutine::getContext()` | `Context::get()` 或 `Async\Fiber::getCurrentContext()` |
| Context 类型 | `Swoole\Coroutine\Context` | `ArrayObject` |
| 使用方式 | 直接创建协程 | 使用 `Async\Fiber` + `go()` |
| 自动清理 | ✅ | ✅ |
| 协程隔离 | ✅ | ✅ |

## 注意事项

1. **必须使用 Async\Fiber**：只有 `Async\Fiber` 才有 Context，普通 `\Fiber` 没有
2. **需要通过 go() 启动**：必须用 `go($fiber)` 启动才能正确设置 CID
3. **Context 是 ArrayObject**：不是原生数组，但可以像数组一样使用
4. **在 Fiber 外调用**：`getCid()` 返回 0，`get()` 返回 null
5. **数据持久化**：修改 Context 会立即持久化（对象引用）

## 最佳实践

1. **统一使用 Context::get()**：比直接调用 `Fiber::getCurrentContext()` 更简洁
2. **在 Fiber 入口设置数据**：尽早初始化 Context 数据
3. **避免存储大对象**：Context 在 Fiber 生命周期内一直存在
4. **使用 request_id 追踪**：便于调试和日志记录

## 测试

运行测试：

```bash
php -d extension=target/release/libasync_php.dylib test_final_context.php
```

测试覆盖：
- ✅ 基本的 CID 分配和 Context 访问
- ✅ Context 数据设置和持久化
- ✅ 嵌套函数调用中的 Context 访问
- ✅ 统计信息
