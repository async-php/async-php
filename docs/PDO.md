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
