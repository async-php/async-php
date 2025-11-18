# spawnIO 函数存在的问题分析

## 函数位置
`src-php/Async/IO.php:204`

## 当前实现
```php
public static function spawnIO($io): Channel
{
    $channel = new Channel();
    Kernel::spawn(function () use ($io, $channel) {
        while (true) {
            $request = $channel->pop();

            // Terminate if channel closed
            if (is_null($request)) {  // ❌ 问题1: 错误的检查
                break;
            }

            [$method, $args] = $request;  // ❌ 问题2: 未验证格式

            // Terminate on close command
            if ($method === '__close__') {
                $channel->close();
                break;
            }

            $result = $io->$method(...$args);  // ❌ 问题3: 未捕获异常

            // If push fails (channel closed), terminate
            if (!$channel->push($result)) {  // ❌ 问题4: 错误的检查
                break;
            }
        }
    });
    return $channel;
}
```

## 问题详细分析

### 问题1: Channel 返回值检查错误

**问题代码:**
```php
$request = $channel->pop();
if (is_null($request)) {
    break;
}
```

**根本原因:**
- `Channel::pop()` 返回 `[value, bool]` 格式的数组
  - 成功: `[value, true]`
  - 失败(channel关闭或超时): `[null, false]`
- `$request` 永远不会是 `null`，它总是一个数组
- `is_null($request)` 永远为 `false`，导致 spawned fiber 在 channel 关闭后无法正确退出

**时序问题:**
1. 主协程调用 `$channel->close()`
2. spawned fiber 调用 `$channel->pop()` 返回 `[null, false]`
3. spawned fiber 检查 `is_null($request)` → 结果为 `false`
4. spawned fiber 继续执行，尝试解构 `[null, false]`
5. `[$method, $args] = [null, false]` → `$method = null, $args = false`
6. `$io->$method(...)` 崩溃: "Method name must be a string"

**正确做法:**
```php
[$value, $ok] = $channel->pop();
if (!$ok) {
    break;
}
[$method, $args] = $value;
```

### 问题2: 数据格式未验证

**问题代码:**
```php
[$method, $args] = $request;
```

**风险:**
- 如果 channel 接收到的不是 `[$method, $args]` 格式，会导致解构失败
- 如果 `$method` 不是字符串，`$io->$method()` 会崩溃
- 如果 `$args` 不是数组，`...$args` 会崩溃

**正确做法:**
```php
if (!is_array($value) || count($value) !== 2) {
    continue; // 或者 break
}
[$method, $args] = $value;
if (!is_string($method) || !is_array($args)) {
    continue;
}
```

### 问题3: 异常未捕获

**问题代码:**
```php
$result = $io->$method(...$args);
```

**风险:**
- 如果 `$io->$method()` 抛出异常，整个 spawned fiber 会崩溃
- Fiber 崩溃后，主协程无法感知
- Channel 仍然开启，但没有消费者，导致主协程调用 `$channel->pop()` 时永久阻塞

**异常场景示例:**
```php
class FileReader {
    public function read($length) {
        throw new Exception("Disk error");
    }
}

$channel = IO::spawnIO($reader);
$channel->push(['read', [1024]]);
$result = $channel->pop(); // ⚠️ 永久阻塞！spawned fiber 已经崩溃
```

**正确做法:**
```php
try {
    $result = $io->$method(...$args);
    $channel->push($result);
} catch (\Throwable $e) {
    // 将异常信息推送回去，或者关闭 channel
    $channel->push(['error' => $e->getMessage()]);
    // 或者直接 break
}
```

### 问题4: push 返回值检查错误

**问题代码:**
```php
if (!$channel->push($result)) {
    break;
}
```

**根本原因:**
- `Channel::push()` 返回 `[null, bool]` 格式的数组，不是布尔值
- `!$channel->push($result)` 检查数组是否为 falsy
- 非空数组永远是 truthy，所以这个条件永远为 `true`，`break` 永远不会执行

**正确做法:**
```php
[, $ok] = $channel->push($result);
if (!$ok) {
    break;
}
```

### 问题5: 资源泄漏

**问题场景:**
```php
class DatabaseConnection {
    public function query($sql) { /* ... */ }
    public function close() {
        echo "Closing database connection\n";
    }
}

$db = new DatabaseConnection();
$channel = IO::spawnIO($db);

// 外部直接关闭 channel
$channel->close();

// spawned fiber 退出，但 $db->close() 从未被调用
// 数据库连接泄漏！
```

**风险:**
- 如果 `$io` 对象持有文件句柄、网络连接、数据库连接等资源
- spawned fiber 异常退出或正常退出时，都不会调用清理方法
- 导致资源泄漏

**建议:**
```php
try {
    while (true) {
        // ... 主循环 ...
    }
} finally {
    // 确保资源清理
    if (method_exists($io, 'close')) {
        try {
            $io->close();
        } catch (\Throwable $e) {
            // Log error
        }
    }
}
```

### 问题6: 竞态条件

**场景:**
```php
$channel = IO::spawnIO($reader);

// Thread 1: 主协程
$channel->close();

// Thread 2: spawned fiber (可能正在执行)
$result = $io->read(1024); // 可能需要很长时间
$channel->push($result);   // Channel 已关闭，push 失败
```

**时序问题:**
1. spawned fiber 从 channel pop 出请求
2. spawned fiber 开始执行 `$io->method()`（耗时操作）
3. 主协程调用 `$channel->close()`
4. spawned fiber 完成操作，尝试 `$channel->push($result)`
5. push 失败（channel 已关闭），但结果已经丢失

**影响:**
- 执行结果丢失
- spawned fiber 退出，但主协程可能还在等待结果

## 推荐的修复方案

```php
public static function spawnIO($io): Channel
{
    $channel = new Channel();
    Kernel::spawn(function () use ($io, $channel) {
        try {
            while (true) {
                // 修复1: 正确检查 channel 关闭
                [$request, $ok] = $channel->pop();
                if (!$ok) {
                    break;
                }

                // 修复2: 验证数据格式
                if (!is_array($request) || count($request) !== 2) {
                    continue;
                }

                [$method, $args] = $request;

                // 处理关闭命令
                if ($method === '__close__') {
                    break;
                }

                // 修复2: 验证方法和参数
                if (!is_string($method) || !is_array($args)) {
                    continue;
                }

                // 修复3: 捕获异常
                try {
                    $result = $io->$method(...$args);

                    // 修复4: 正确检查 push 返回值
                    [, $pushOk] = $channel->push($result);
                    if (!$pushOk) {
                        // Channel 已关闭，退出
                        break;
                    }
                } catch (\Throwable $e) {
                    // 将异常推送回主协程
                    [, $pushOk] = $channel->push([
                        '__error__' => true,
                        'message' => $e->getMessage(),
                        'trace' => $e->getTraceAsString()
                    ]);
                    if (!$pushOk) {
                        break;
                    }
                }
            }
        } finally {
            // 修复5: 确保资源清理
            if (method_exists($io, 'close')) {
                try {
                    $io->close();
                } catch (\Throwable $e) {
                    // 静默失败或记录日志
                }
            }

            // 确保 channel 关闭
            if (!$channel->isClosed()) {
                $channel->close();
            }
        }
    });
    return $channel;
}
```

## 使用建议

修复后，使用方需要处理错误响应：

```php
$channel = IO::spawnIO($reader);

// 发送请求
$channel->push(['read', [1024]]);

// 接收响应
[$result, $ok] = $channel->pop();
if (!$ok) {
    // Channel 关闭
    throw new Exception("IO channel closed unexpectedly");
}

// 检查是否是错误响应
if (is_array($result) && isset($result['__error__'])) {
    throw new Exception("IO error: " . $result['message']);
}

// 正常处理结果
echo "Read: $result\n";

// 完成后发送关闭命令
$channel->push(['__close__', []]);
```

## 总结

当前 `spawnIO` 实现存在**严重的时序问题和异常处理缺陷**：

1. ❌ Channel 返回值检查错误 → spawned fiber 无法正确退出
2. ❌ 数据格式未验证 → 易崩溃
3. ❌ 异常未捕获 → spawned fiber 静默崩溃，主协程永久阻塞
4. ❌ push 返回值检查错误 → 逻辑错误
5. ❌ 资源未清理 → 资源泄漏
6. ⚠️  竞态条件 → 结果可能丢失

**建议立即修复，否则在生产环境中会导致不可预测的行为和资源泄漏。**
