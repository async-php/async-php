# Async PDO (MySQL / PostgreSQL)

该仓库提供一个用户态的 `PDO`/`PDOStatement` 兼容层（实现位于 `src-php/PDO`，命名空间为 `PDO\*` 并通过 `class_alias` 暴露到全局），底层通过扩展暴露的 `Async\Kernel\PDO\MySql` / `Async\Kernel\PDO\PgSql` 执行 SQL。

## 使用前提

- 必须加载 async-php 扩展（需要 `run()` / `go()` 等函数存在）。
- 需要禁用 PHP 原生 `pdo` 扩展，否则无法声明全局类 `PDO`/`PDOStatement`/`PDOException`（以及 `Pdo\Pgsql` / `Pdo\Mysql`）。

示例（`php.ini`）：

```ini
; 禁用原生 PDO 扩展（避免类名冲突）
; extension=pdo
; extension=pdo_mysql
; extension=pdo_pgsql
```

## 基本用法

```php
use Async\Kernel;

Kernel::run(function () {
    $pdo = new PDO('mysql:host=127.0.0.1;port=3306;dbname=test', 'root', 'pass');
    $stmt = $pdo->prepare('SELECT id, name FROM users WHERE id = ?');
    $stmt->execute([1]);
    $row = $stmt->fetch(PDO::FETCH_ASSOC);
    var_dump($row);
});
```

## 说明

- `mysql:` / `pgsql:` 形式的 PDO DSN 会在 PHP 层转换为 SQLx 的 URI（例如 `mysql://...` / `postgres://...`）。
- `?` 与 `:name` 占位符会在 PHP 层编译为驱动对应的占位符（`pgsql` 会转换为 `$1/$2/...`）。

## 连接池（协程共享）

底层 `Async\Kernel\PDO\MySql` / `Async\Kernel\PDO\PgSql` 基于 SQLx Pool 实现，PDO 层可以通过选项打开“同一个 PDO 实例在多个协程中并发使用”：

```php
$pdo = new PDO(
    'mysql:host=127.0.0.1;port=3306;dbname=test',
    'root',
    'pass',
    [
        PDO::ATTR_CONNECTION_POOL_ENABLED => true,
        PDO::ATTR_CONNECTION_POOL_MIN_SIZE => 2,
        PDO::ATTR_CONNECTION_POOL_MAX_SIZE => 10,
        PDO::ATTR_CONNECTION_POOL_TIMEOUT => 5,      // 创建连接池/首次连接超时（秒）
        PDO::ATTR_CONNECTION_POOL_WAIT_TIMEOUT => 3, // 获取连接等待超时（秒，含健康检查/建连等阶段）
        PDO::ATTR_CONNECTION_POOL_HEARTBEAT => true, // 获取连接前做健康检查
        PDO::ATTR_CONNECTION_POOL_IDLE_TIME => 30,   // 空闲连接回收（秒）
    ]
);
```

说明：

- 事务 / `errorInfo()` / `lastInsertId()` 等状态按协程隔离，避免不同协程互相污染。
- 连接池配置在构造函数中生效（透传到扩展侧 SQLx PoolOptions），`setAttribute()` 当前不会动态重配底层 pool。
