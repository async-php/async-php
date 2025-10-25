<?php

namespace Async\Database;

use Async\Kernel\DB\MySql\Transaction as MySqlTransaction;
use Fiber;

class MySQL implements DriverInterface
{
    private MySql $driver;

    private function __construct(MySql $driver)
    {
        $this->driver = $driver;
    }

    public static function connect(string $dsn, int $maxConnections = 10): ?static
    {
        $driver = Fiber::suspend(MySql::connect($dsn, $maxConnections));
        if (!$driver) return null;
        return new static($driver);
    }

    public function query(string $sql, ?array $params = null): array|false
    {
        return Fiber::suspend($this->driver->query($sql, $params));
    }

    public function execute(string $sql, ?array $params = null): int|false
    {
        return Fiber::suspend($this->driver->execute($sql, $params));
    }

    public function beginTransaction(): TransactionInterface|false
    {
        $tx = Fiber::suspend($this->driver->beginTransaction());
        if (!$tx) return false;
        return new MySQLTransaction($tx);
    }
}

class MySQLTransaction implements TransactionInterface
{
    private MySqlTransaction $tx;

    public function __construct(MySqlTransaction $tx)
    {
        $this->tx = $tx;
    }

    public function query(string $sql, ?array $params = null): array|false
    {
        return Fiber::suspend($this->tx->query($sql, $params));
    }

    public function execute(string $sql, ?array $params = null): int|false
    {
        return Fiber::suspend($this->tx->execute($sql, $params));
    }

    public function commit(): bool
    {
        return Fiber::suspend($this->tx->commit());
    }

    public function rollback(): bool
    {
        return Fiber::suspend($this->tx->rollback());
    }
}